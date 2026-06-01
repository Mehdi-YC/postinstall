+++
title = "PostgreSQL Reference for Developers"
description = "Queries, joins, indexes, transactions, and the things that actually matter when building on Postgres"
[extra]
date = 2024-01-15
+++

# PostgreSQL Reference for Developers

*Queries, joins, indexes, transactions, and the things that actually matter when building on Postgres.*

## Data Definition: Tables

```sql,linenos
    -- Create a table
    CREATE TABLE users (
        id          BIGSERIAL PRIMARY KEY,        -- auto-incrementing integer PK
        email       TEXT      NOT NULL UNIQUE,     -- enforced at DB level
        name        TEXT      NOT NULL,
        age         INTEGER   CHECK (age >= 0),
        role        TEXT      NOT NULL DEFAULT 'user',
        created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
        deleted_at  TIMESTAMPTZ                   -- NULL = not deleted (soft delete)
    );

    -- Prefer BIGSERIAL or BIGINT GENERATED ALWAYS AS IDENTITY over SERIAL
    id  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY

    -- Or use UUID as primary key (requires pgcrypto or gen_random_uuid())
    id  UUID PRIMARY KEY DEFAULT gen_random_uuid()

    -- Alter an existing table
    ALTER TABLE users ADD COLUMN bio TEXT;
    ALTER TABLE users ALTER COLUMN name SET NOT NULL;
    ALTER TABLE users DROP COLUMN bio;
    ALTER TABLE users RENAME COLUMN name TO full_name;
    ALTER TABLE users RENAME TO accounts;

    -- Drop
    DROP TABLE users;
    DROP TABLE IF EXISTS users CASCADE; -- CASCADE drops dependent views/FKs
```

## Foreign Keys

```sql,linenos
    CREATE TABLE posts (
        id         BIGSERIAL PRIMARY KEY,
        user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        title      TEXT NOT NULL,
        body       TEXT,
        published  BOOLEAN NOT NULL DEFAULT false,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now()
    );

    -- ON DELETE options:
    -- CASCADE     → delete child rows when parent is deleted
    -- SET NULL    → set FK column to NULL when parent is deleted
    -- SET DEFAULT → set FK column to its default value
    -- RESTRICT    → prevent parent deletion if children exist (default)
    -- NO ACTION   → like RESTRICT but deferred until end of transaction

    -- Always index foreign key columns (Postgres does NOT do this automatically)
    CREATE INDEX ON posts(user_id);
```

<blockquote class="warn">
<p><strong>Postgres does not automatically create indexes on foreign key columns.</strong> Every FK column that you JOIN or filter on needs a manual <code>CREATE INDEX</code>. Missing FK indexes cause sequential scans and are one of the most common Postgres performance mistakes.</p>
</blockquote>

## INSERT, UPDATE, DELETE

```sql,linenos
    -- Insert a single row
    INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice');

    -- Insert and return the generated id
    INSERT INTO users (email, name) VALUES ('bob@example.com', 'Bob') RETURNING id;

    -- Insert multiple rows
    INSERT INTO users (email, name) VALUES
        ('carol@example.com', 'Carol'),
        ('dave@example.com',  'Dave');

    -- Upsert: insert or update on conflict
    INSERT INTO users (email, name)
    VALUES ('alice@example.com', 'Alice Updated')
    ON CONFLICT (email)
    DO UPDATE SET name = EXCLUDED.name, updated_at = now();

    -- Ignore on conflict (do nothing)
    INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice')
    ON CONFLICT DO NOTHING;

    -- Update
    UPDATE users SET name = 'Alice Smith', role = 'admin' WHERE id = 1;

    -- Update and return modified rows
    UPDATE users SET role = 'admin' WHERE email = 'alice@example.com' RETURNING *;

    -- Delete
    DELETE FROM users WHERE id = 1;
    DELETE FROM users WHERE deleted_at < now() - INTERVAL '30 days' RETURNING id;
```

## SELECT Fundamentals

```sql,linenos
    -- Basic select
    SELECT id, name, email FROM users;
    SELECT * FROM users;                     -- avoid * in production queries

    -- Filter
    SELECT * FROM users WHERE role = 'admin' AND age > 18;
    SELECT * FROM users WHERE id IN (1, 2, 3);
    SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM banned);
    SELECT * FROM users WHERE deleted_at IS NULL;       -- NULL check
    SELECT * FROM users WHERE deleted_at IS NOT NULL;

    -- LIKE and ILIKE (case-insensitive)
    SELECT * FROM users WHERE name LIKE 'A%';       -- starts with A
    SELECT * FROM users WHERE email ILIKE '%@gmail.com';

    -- Sort
    SELECT * FROM users ORDER BY created_at DESC;
    SELECT * FROM users ORDER BY role ASC, name ASC;
    SELECT * FROM users ORDER BY name NULLS LAST;    -- push NULLs to the end

    -- Limit and offset (pagination)
    SELECT * FROM users ORDER BY id LIMIT 20 OFFSET 40; -- page 3 of 20
```

<blockquote class="warn">
<p><strong>OFFSET pagination is slow on large tables.</strong> <code>OFFSET 10000 LIMIT 20</code> requires scanning 10020 rows to return 20. Use keyset (cursor) pagination instead: <code>WHERE id > :last_id ORDER BY id LIMIT 20</code>. Much faster and consistent under inserts.</p>
</blockquote>

## Aggregates and GROUP BY

```sql,linenos
    -- Aggregate functions
    SELECT COUNT(*) FROM users;
    SELECT COUNT(*) FROM users WHERE role = 'admin';
    SELECT AVG(age), MIN(age), MAX(age), SUM(age) FROM users;

    -- GROUP BY
    SELECT role, COUNT(*) AS total
    FROM users
    GROUP BY role
    ORDER BY total DESC;

    -- HAVING: filter on aggregated values (WHERE runs before aggregation, HAVING after)
    SELECT user_id, COUNT(*) AS post_count
    FROM posts
    GROUP BY user_id
    HAVING COUNT(*) > 5
    ORDER BY post_count DESC;

    -- Count distinct values
    SELECT COUNT(DISTINCT user_id) FROM posts;

    -- Conditional aggregation
    SELECT
        COUNT(*) AS total,
        COUNT(*) FILTER (WHERE role = 'admin') AS admins,
        COUNT(*) FILTER (WHERE deleted_at IS NOT NULL) AS deleted
    FROM users;
```

## Subqueries and CTEs

```sql,linenos
    -- Subquery in WHERE
    SELECT * FROM posts
    WHERE user_id IN (
        SELECT id FROM users WHERE role = 'admin'
    );

    -- EXISTS: often faster than IN for large subqueries
    SELECT * FROM users u
    WHERE EXISTS (
        SELECT 1 FROM posts p WHERE p.user_id = u.id
    );

    -- CTE (Common Table Expression): named subquery, run once, referenced multiple times
    WITH active_users AS (
        SELECT id, name FROM users WHERE deleted_at IS NULL
    ),
    prolific AS (
        SELECT user_id, COUNT(*) AS post_count
        FROM posts
        GROUP BY user_id
        HAVING COUNT(*) > 10
    )
    SELECT u.name, p.post_count
    FROM active_users u
    JOIN prolific p ON p.user_id = u.id
    ORDER BY p.post_count DESC;

    -- Recursive CTE: for trees and hierarchies
    WITH RECURSIVE org_tree AS (
        SELECT id, name, manager_id, 0 AS depth
        FROM employees WHERE manager_id IS NULL     -- base case: root
        UNION ALL
        SELECT e.id, e.name, e.manager_id, t.depth + 1
        FROM employees e
        JOIN org_tree t ON e.manager_id = t.id          -- recursive case
    )
    SELECT * FROM org_tree ORDER BY depth, name;
```

## JOIN Types

| Join Type | Returns | When to Use |
|-----------|---------|-------------|
| `INNER JOIN` | Rows where the condition matches in both tables | You only want rows that have a matching partner |
| `LEFT JOIN` | All rows from left table; NULLs where no match in right | Optional relationship — keep left rows even if no match |
| `RIGHT JOIN` | All rows from right table; NULLs where no match in left | Rarely used; just flip the tables and use LEFT JOIN |
| `FULL OUTER JOIN` | All rows from both tables; NULLs where no match | Find rows that exist in one table but not the other |
| `CROSS JOIN` | Cartesian product — every combination | Generating test data; matrix comparisons |

```sql,linenos
    -- INNER JOIN: only posts that have a matching user
    SELECT p.title, u.name
    FROM posts p
    JOIN users u ON u.id = p.user_id;  -- JOIN = INNER JOIN

    -- LEFT JOIN: all users, even those with no posts
    SELECT u.name, COUNT(p.id) AS post_count
    FROM users u
    LEFT JOIN posts p ON p.user_id = u.id
    GROUP BY u.id, u.name
    ORDER BY post_count DESC;

    -- LEFT JOIN to find rows with NO match (anti-join pattern)
    SELECT u.* FROM users u
    LEFT JOIN posts p ON p.user_id = u.id
    WHERE p.id IS NULL;  -- users who have never posted

    -- Multiple joins
    SELECT p.title, u.name AS author, c.body AS comment
    FROM posts p
    JOIN users u ON u.id = p.user_id
    LEFT JOIN comments c ON c.post_id = p.id
    WHERE p.published = true
    ORDER BY p.created_at DESC;

    -- Self-join: join a table to itself (e.g., employee → manager)
    SELECT e.name AS employee, m.name AS manager
    FROM employees e
    LEFT JOIN employees m ON m.id = e.manager_id;
```

<blockquote class="tip">
<p><strong>JOIN order doesn't change results, but it hints to the planner.</strong> Put the smaller/more-filtered table first for readability. Postgres's query planner will reorder joins for efficiency regardless — but explicit <code>JOIN</code> instead of comma-separated <code>FROM</code> makes intent clear and avoids accidental cartesian products.</p>
</blockquote>

## UNION, INTERSECT, EXCEPT

```sql,linenos
    -- UNION: combine results, remove duplicates
    SELECT email FROM users
    UNION
    SELECT email FROM pending_users;

    -- UNION ALL: combine results, keep duplicates (faster — no dedup step)
    SELECT id, name, 'user' AS source FROM users
    UNION ALL
    SELECT id, name, 'admin' AS source FROM admins;

    -- INTERSECT: rows in both result sets
    SELECT email FROM newsletter_subscribers
    INTERSECT
    SELECT email FROM paying_customers;

    -- EXCEPT: rows in first set but not second
    SELECT email FROM users
    EXCEPT
    SELECT email FROM unsubscribed;
```

## Index Basics

Indexes speed up reads by creating a separate data structure the planner can use instead of scanning the whole table. They cost write overhead and storage. Add them where you filter, sort, or join — not everywhere.

```sql,linenos
    -- Basic B-tree index (default, best for equality and range queries)
    CREATE INDEX idx_users_email ON users(email);
    CREATE INDEX ON posts(user_id);    -- auto-named
    CREATE INDEX ON posts(created_at DESC);

    -- Unique index (enforces uniqueness, equivalent to UNIQUE constraint)
    CREATE UNIQUE INDEX ON users(email);

    -- Composite index: column order matters
    -- Useful when you filter on (status, created_at) together
    -- Also usable for queries on just `status` (leftmost prefix rule)
    CREATE INDEX ON posts(status, created_at DESC);

    -- Partial index: only indexes rows matching a condition
    -- Smaller, faster for queries that always include that condition
    CREATE INDEX ON users(email) WHERE deleted_at IS NULL;
    CREATE INDEX ON orders(created_at) WHERE status = 'pending';

    -- Concurrent index creation: doesn't lock the table (use in production)
    CREATE INDEX CONCURRENTLY idx_posts_title ON posts(title);

    -- Drop
    DROP INDEX idx_users_email;
    DROP INDEX CONCURRENTLY idx_posts_title; -- non-blocking drop

    -- List indexes on a table
    \d users  -- psql: shows table structure and indexes
```

## Index Types

| Type | Best For | Notes |
|------|----------|-------|
| `B-tree` | Equality, range, sorting, LIKE 'prefix%' | Default. Use for almost everything. |
| `Hash` | Equality only (`=`) | Slightly faster than B-tree for pure equality, but B-tree is usually fine. |
| `GIN` | Arrays, JSONB, full-text search, `@>`, `?` | Required for indexing JSONB keys/values and array containment. |
| `GiST` | Geometric types, full-text, ranges | Used by PostGIS. Also for `tsvector` full-text search. |
| `BRIN` | Very large tables with naturally ordered data (time-series, logs) | Tiny. Fast to build. Poor selectivity. Good for append-only partitioned tables. |

```sql,linenos
    -- GIN index for JSONB (required for @>, ?, ?|, ?& operators)
    CREATE INDEX ON events USING gin(data);

    -- GIN index for full-text search
    CREATE INDEX ON articles USING gin(to_tsvector('english', title || ' ' || body));

    -- BRIN index for a large log table (created_at is always increasing)
    CREATE INDEX ON logs USING brin(created_at);
```

## EXPLAIN: Reading the Query Plan

```sql,linenos
    -- Show the query plan (no execution)
    EXPLAIN SELECT * FROM users WHERE email = 'alice@example.com';

    -- Show plan WITH actual execution stats (run the query)
    EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM users WHERE email = 'alice@example.com';

    -- Key things to look for in EXPLAIN output:
    -- Seq Scan     → full table scan. BAD on large tables. Missing index?
    -- Index Scan   → uses index to find rows, fetches from heap. GOOD.
    -- Index Only Scan → all data in index, no heap access. BEST.
    -- Bitmap Scan  → index scan for many rows, batch heap access. OK.
    -- Nested Loop  → for each row in outer, scan inner. Fast if inner is small.
    -- Hash Join    → build hash table from smaller side. Fast for large joins.
    -- Merge Join   → merge two sorted inputs. Fast when both sides are sorted.
    -- cost=X..Y    → estimated startup cost .. total cost (arbitrary planner units)
    -- rows=N       → estimated row count (inaccurate = stale statistics → ANALYZE)
    -- actual time=X..Y rows=N loops=N → real execution data (with ANALYZE)

    -- Update statistics if estimates are way off
    ANALYZE users;
    ANALYZE;  -- analyze all tables
```

<blockquote class="tip">
<p><strong>Use explain.dalibo.com.</strong> Paste your <code>EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)</code> output into explain.dalibo.com for a visual tree with highlighted bottlenecks. Far easier to read than raw text output for complex plans.</p>
</blockquote>

## Transactions

A transaction groups statements into an all-or-nothing unit. Either all statements succeed and commit, or any failure rolls everything back. Postgres transactions are fully ACID compliant.

```sql,linenos
    -- Basic transaction
    BEGIN;
        UPDATE accounts SET balance = balance - 100 WHERE id = 1;
        UPDATE accounts SET balance = balance + 100 WHERE id = 2;
    COMMIT;  -- both updates committed atomically

    -- Rollback on error
    BEGIN;
        UPDATE accounts SET balance = balance - 1000 WHERE id = 1;
        -- something goes wrong in application code...
    ROLLBACK;  -- undoes the UPDATE, balance unchanged

    -- Savepoints: partial rollback within a transaction
    BEGIN;
        INSERT INTO orders (...) VALUES (...);
        SAVEPOINT after_order;
        INSERT INTO order_items (...) VALUES (...);
        -- if items insert fails:
        ROLLBACK TO SAVEPOINT after_order;
        -- order still exists, items rolled back
    COMMIT;
```

## Isolation Levels

Isolation levels control what concurrent transactions can see of each other's changes. Higher isolation = fewer anomalies but more contention.

| Level | Dirty Read | Non-repeatable Read | Phantom Read | Use Case |
|-------|------------|---------------------|--------------|----------|
| `READ COMMITTED` | No | Yes | Yes | Default. Fine for most apps. |
| `REPEATABLE READ` | No | No | No* | Reports, consistent snapshots. |
| `SERIALIZABLE` | No | No | No | Financial ops, strict correctness. |

```sql,linenos
    -- Set isolation level for a transaction
    BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ;
        SELECT SUM(balance) FROM accounts;  -- consistent snapshot for this tx
        -- other transactions can commit changes, but we don't see them
    COMMIT;
```

## Locking

```sql,linenos
    -- SELECT FOR UPDATE: lock rows for update, block other writers
    -- Use when you read a row and intend to update it (prevents lost updates)
    BEGIN;
    SELECT balance FROM accounts WHERE id = 1 FOR UPDATE;
    -- now we own a lock: other transactions block on this row
    UPDATE accounts SET balance = balance - 50 WHERE id = 1;
    COMMIT;

    -- SELECT FOR UPDATE SKIP LOCKED: skip rows locked by others
    -- Pattern: job queue — workers grab available jobs without blocking each other
    BEGIN;
    SELECT * FROM jobs
    WHERE status = 'pending'
    ORDER BY created_at
    LIMIT 1
    FOR UPDATE SKIP LOCKED;
    -- process the job...
    UPDATE jobs SET status = 'done' WHERE id = :job_id;
    COMMIT;

    -- Advisory locks: application-level locks (no table/row needed)
    SELECT pg_try_advisory_lock(12345);  -- returns true if acquired, false if not
    SELECT pg_advisory_unlock(12345);
```

<blockquote class="warn">
<p><strong>Deadlocks happen when two transactions each hold a lock the other needs.</strong> Postgres detects them and kills one transaction. Prevent them by always acquiring locks in the same order across transactions, and keeping transactions short.</p>
</blockquote>

## VACUUM and Autovacuum

```sql,linenos
    -- Why VACUUM exists:
    -- Postgres uses MVCC: UPDATE and DELETE don't remove old row versions.
    -- They mark them dead. Dead rows pile up ("table bloat").
    -- VACUUM reclaims dead row space and updates the visibility map.

    -- Manual vacuum (autovacuum usually handles this)
    VACUUM users;
    VACUUM ANALYZE users;   -- vacuum + update statistics
    VACUUM FULL users;       -- rewrite table, reclaim OS space — locks table, use sparingly

    -- Check autovacuum health
    SELECT relname, n_dead_tup, n_live_tup, last_autovacuum, last_autoanalyze
    FROM pg_stat_user_tables
    ORDER BY n_dead_tup DESC;
```

## Data Types

| Type | Use For | Notes |
|------|---------|-------|
| `TEXT` | Variable-length strings | Preferred over VARCHAR(n) — no performance difference, less hassle. |
| `VARCHAR(n)` | Strings with length limit | Only use if you need DB-level length enforcement. |
| `INTEGER / INT` | 32-bit integer | Max ~2.1 billion. Use BIGINT if IDs may exceed that. |
| `BIGINT` | 64-bit integer | Preferred for IDs on tables that will grow. |
| `NUMERIC(p,s)` | Exact decimals (money) | No floating-point errors. Use for currency. Slower than float. |
| `FLOAT8 / DOUBLE PRECISION` | Approximate decimals | Fast. Imprecise. Not for money. |
| `BOOLEAN` | true/false | Accepts true/false, 't'/'f', 'yes'/'no', 1/0. |
| `TIMESTAMPTZ` | Timestamps | Always use TIMESTAMPTZ (with time zone) — stores UTC internally. |
| `DATE` | Calendar date only | No time component. |
| `INTERVAL` | Duration | `INTERVAL '3 days'`, `INTERVAL '1 hour 30 minutes'`. |
| `UUID` | Universally unique IDs | Use `gen_random_uuid()` (built-in since Pg 13). |
| `JSONB` | JSON data | Binary JSON — indexable, queryable. Prefer over `JSON`. |
| `ARRAY` | Arrays of any type | `TEXT[]`, `INTEGER[]`. Indexable with GIN. |
| `ENUM` | Fixed set of string values | Enforced at DB level. Adding values requires ALTER TYPE. |

<blockquote class="warn">
<p><strong>Always use TIMESTAMPTZ, never TIMESTAMP.</strong> <code>TIMESTAMP</code> (without time zone) stores no timezone info. When you insert a value, Postgres strips the offset. You lose the ability to reason about time correctly across timezones. <code>TIMESTAMPTZ</code> always stores UTC and converts on display.</p>
</blockquote>

## JSONB

```sql,linenos
    -- Store and query semi-structured data
    CREATE TABLE events (
        id    BIGSERIAL PRIMARY KEY,
        data  JSONB NOT NULL
    );

    -- Insert JSON
    INSERT INTO events (data) VALUES
        ('{"type": "click", "user_id": 42, "tags": ["mobile", "nav"]}');

    -- Access operators
    data->'user_id'           -- returns JSON value: 42
    data->>'user_id'          -- returns TEXT value: '42'  (use for WHERE comparisons)
    data->'address'->'city'  -- nested access
    data#>'{address,city}'     -- path access (array of keys)
    data#>>'{address,city}'  -- path access returning TEXT

    -- Query
    SELECT * FROM events WHERE data->>'type' = 'click';
    SELECT * FROM events WHERE (data->>'user_id')::INT = 42;
    SELECT * FROM events WHERE data @> '{"type": "click"}';  -- containment (use GIN index)
    SELECT * FROM events WHERE data ? 'user_id';             -- key exists
    SELECT * FROM events WHERE data->'tags' @> '["mobile"]'; -- array contains value

    -- Update a key in JSONB
    UPDATE events SET data = data || '{"processed": true}' WHERE id = 1;
    UPDATE events SET data = jsonb_set(data, '{user_id}', '99') WHERE id = 1;

    -- Index for fast JSONB queries
    CREATE INDEX ON events USING gin(data);
```

## ENUMs and Custom Types

```sql,linenos
    -- Create an ENUM type
    CREATE TYPE order_status AS ENUM ('pending', 'processing', 'shipped', 'delivered', 'cancelled');

    CREATE TABLE orders (
        id     BIGSERIAL PRIMARY KEY,
        status order_status NOT NULL DEFAULT 'pending'
    );

    -- Add a value to an existing ENUM (can only add, not remove)
    ALTER TYPE order_status ADD VALUE 'refunded' AFTER 'delivered';
```

## Window Functions

Window functions perform a calculation across a set of rows related to the current row — without collapsing them into a single group like `GROUP BY` would.

```sql,linenos
    -- ROW_NUMBER: rank each row within a partition
    SELECT
        name,
        department,
        salary,
        ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) AS rank
    FROM employees;

    -- RANK and DENSE_RANK: same as ROW_NUMBER but ties get same rank
    -- RANK skips numbers after a tie (1,1,3), DENSE_RANK doesn't (1,1,2)
    RANK() OVER (ORDER BY score DESC)
    DENSE_RANK() OVER (ORDER BY score DESC)

    -- LAG / LEAD: access previous / next row's value
    SELECT
        date,
        revenue,
        LAG(revenue) OVER (ORDER BY date) AS prev_revenue,
        revenue - LAG(revenue) OVER (ORDER BY date) AS change
    FROM daily_revenue;

    -- Running total with SUM OVER
    SELECT
        date,
        amount,
        SUM(amount) OVER (ORDER BY date) AS running_total
    FROM transactions;

    -- Top N per group (e.g., top 3 posts per user)
    SELECT * FROM (
        SELECT *,
            ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) AS rn
        FROM posts
    ) sub
    WHERE rn <= 3;
```

## Full-Text Search

```sql,linenos
    -- tsvector: processed text for searching
    -- tsquery: a search query

    -- Basic full-text search
    SELECT title FROM articles
    WHERE to_tsvector('english', title || ' ' || body) @@ to_tsquery('english', 'postgres & index');

    -- Ranking results by relevance
    SELECT title,
        ts_rank(to_tsvector('english', body), to_tsquery('postgres')) AS rank
    FROM articles
    WHERE to_tsvector('english', body) @@ to_tsquery('postgres')
    ORDER BY rank DESC;

    -- Store tsvector in a column for performance (update with trigger or generated column)
    ALTER TABLE articles ADD COLUMN search_vector TSVECTOR
        GENERATED ALWAYS AS (to_tsvector('english', coalesce(title, '') || ' ' || coalesce(body, ''))) STORED;

    CREATE INDEX ON articles USING gin(search_vector);

    -- plainto_tsquery: user input (no operators, just words)
    WHERE search_vector @@ plainto_tsquery('english', user_input)

    -- websearch_to_tsquery: supports "phrases", -exclusions (Pg 11+)
    WHERE search_vector @@ websearch_to_tsquery('english', '"exact phrase" -exclude')
```

## Generated Columns and Constraints

```sql,linenos
    -- Generated (computed) column: value is always derived from other columns
    ALTER TABLE products
        ADD COLUMN price_with_tax NUMERIC
        GENERATED ALWAYS AS (price * 1.20) STORED;

    -- Table-level constraints
    CREATE TABLE bookings (
        id         BIGSERIAL PRIMARY KEY,
        start_date DATE NOT NULL,
        end_date   DATE NOT NULL,
        CONSTRAINT valid_dates CHECK (end_date > start_date),
        CONSTRAINT unique_booking UNIQUE (user_id, start_date)
    );

    -- Exclusion constraint: no overlapping date ranges (requires btree_gist extension)
    CREATE EXTENSION IF NOT EXISTS btree_gist;
    ALTER TABLE bookings
        ADD CONSTRAINT no_overlap
        EXCLUDE USING gist (room_id WITH =, daterange(start_date, end_date) WITH &&);
```

## psql CLI

```bash,linenos
    # Connect
    psql -U username -d dbname
    psql -U username -h hostname -p 5432 -d dbname
    psql "postgresql://user:password@host:5432/dbname"

    # Meta-commands (no semicolon needed)
    \l          -- list databases
    \c dbname   -- connect to database
    \dt         -- list tables in current schema
    \dt *.*     -- list tables in all schemas
    \d tablename -- describe table (columns, indexes, constraints)
    \di         -- list indexes
    \df         -- list functions
    \dv         -- list views
    \dn         -- list schemas
    \du         -- list users/roles
    \timing     -- toggle query timing
    \x          -- toggle expanded output (useful for wide tables)
    \e          -- open query in $EDITOR
    \i file.sql -- run SQL file
    \copy       -- client-side COPY (works over remote connections)
    \q          -- quit

    # Run a query from the shell
    psql -U postgres -d mydb -c "SELECT COUNT(*) FROM users;"

    # Run a SQL file
    psql -U postgres -d mydb -f schema.sql
```

## Users, Roles, and Permissions

```sql,linenos
    -- Create a role (roles can login, groups cannot — but the distinction is just flags)
    CREATE ROLE app_user LOGIN PASSWORD 'securepassword';
    CREATE ROLE readonly NOLOGIN;  -- group role

    -- Grant connect and usage
    GRANT CONNECT ON DATABASE mydb TO app_user;
    GRANT USAGE ON SCHEMA public TO app_user;

    -- Grant table permissions
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_user;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO app_user;

    -- Grant on future tables automatically
    ALTER DEFAULT PRIVILEGES IN SCHEMA public
        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO app_user;

    -- Read-only role pattern
    GRANT USAGE ON SCHEMA public TO readonly;
    GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly;
    GRANT readonly TO reporting_user;  -- assign group role to a user

    -- Revoke
    REVOKE INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public FROM readonly;
```

## Backup and Restore

```bash,linenos
    # pg_dump: logical backup (SQL or custom format)
    pg_dump -U postgres mydb > mydb.sql                         # plain SQL
    pg_dump -U postgres -Fc mydb > mydb.dump                   # custom format (preferred)
    pg_dump -U postgres -Fc -t users mydb > users.dump         # single table
    pg_dump -U postgres -Fc --schema-only mydb > schema.dump   # schema only
    pg_dump -U postgres -Fc --data-only mydb > data.dump       # data only

    # Restore custom format
    pg_restore -U postgres -d mydb mydb.dump
    pg_restore -U postgres -d mydb --clean mydb.dump           # drop objects first
    pg_restore -U postgres -d mydb -j 4 mydb.dump              # parallel restore (4 workers)

    # Restore plain SQL
    psql -U postgres -d mydb < mydb.sql

    # pg_dumpall: dump all databases + roles + tablespaces
    pg_dumpall -U postgres > all.sql

    # COPY: fast bulk import/export
    # Server-side (must be superuser, reads/writes server filesystem)
    COPY users TO '/tmp/users.csv' CSV HEADER;
    COPY users FROM '/tmp/users.csv' CSV HEADER;

    # Client-side (works over remote connections)
    \copy users TO 'users.csv' CSV HEADER
    \copy users FROM 'users.csv' CSV HEADER
```

## Useful Diagnostic Queries

```sql,linenos
    -- Show running queries
    SELECT pid, now() - pg_stat_activity.query_start AS duration,
           query, state
    FROM pg_stat_activity
    WHERE state != 'idle'
    ORDER BY duration DESC;

    -- Kill a query
    SELECT pg_cancel_backend(pid);   -- graceful cancel
    SELECT pg_terminate_backend(pid); -- force kill

    -- Show locks and what's blocking what
    SELECT blocked.pid, blocked.query,
           blocking.pid AS blocking_pid, blocking.query AS blocking_query
    FROM pg_stat_activity blocked
    JOIN pg_stat_activity blocking
        ON blocking.pid = ANY(pg_blocking_pids(blocked.pid));

    -- Slowest queries (requires pg_stat_statements extension)
    CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
    SELECT query, round(mean_exec_time::numeric, 2) AS avg_ms,
           calls, round(total_exec_time::numeric, 2) AS total_ms
    FROM pg_stat_statements
    ORDER BY mean_exec_time DESC
    LIMIT 20;


    -- Table sizes
    SELECT relname AS table,
        pg_size_pretty(pg_total_relation_size(relid)) AS total_size,
        pg_size_pretty(pg_relation_size(relid)) AS table_size,
        pg_size_pretty(pg_total_relation_size(relid) - pg_relation_size(relid)) AS index_size
    FROM pg_stat_user_tables
    ORDER BY pg_total_relation_size(relid) DESC;

    -- Unused indexes (wasting write overhead)
    SELECT indexrelname, idx_scan, pg_size_pretty(pg_relation_size(indexrelid)) AS size
    FROM pg_stat_user_indexes
    WHERE idx_scan = 0
    ORDER BY pg_relation_size(indexrelid) DESC;
```
<blockquote class="success">
<p><strong>pg_stat_statements is essential.</strong> Enable it in <code>postgresql.conf</code> with <code>shared_preload_libraries = 'pg_stat_statements'</code> and restart. It tracks query statistics across all executions — the single most useful tool for finding slow queries in production.</p>
</blockquote>