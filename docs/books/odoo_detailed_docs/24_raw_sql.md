# 24 — Raw SQL

Use raw SQL for performance-critical aggregations, complex joins, or bulk updates.

## Basic usage

```python
def _get_revenue_by_category(self):
    query = """
        SELECT
            c.name AS category,
            SUM(l.expected_revenue) AS total,
            COUNT(l.id) AS lead_count
        FROM crm_lead l
        LEFT JOIN crm_lead_category c ON c.id = l.category_id
        WHERE l.state NOT IN ('lost')
        GROUP BY c.name
        ORDER BY total DESC
    """
    self.env.cr.execute(query)
    return self.env.cr.dictfetchall()
    # [{'category': 'Technology', 'total': 450000, 'lead_count': 5}]
```

## Parameters — ALWAYS use %s, never f-strings

```python
# CORRECT — parameterized query
def _get_leads_for_state(self, state):
    self.env.cr.execute(
        "SELECT id FROM crm_lead WHERE state = %s AND company_id = %s",
        (state, self.env.company.id)   # always a tuple
    )
    ids = [row['id'] for row in self.env.cr.dictfetchall()]
    return self.env['crm.lead'].browse(ids)

# WRONG — SQL injection risk
def _get_leads_bad(self, state):
    self.env.cr.execute(f"SELECT id FROM crm_lead WHERE state = '{state}'")
```

## Cursor methods

| Method | Returns |
|---|---|
| `cr.execute(sql, params)` | None (runs the query) |
| `cr.fetchone()` | Single tuple or None |
| `cr.fetchall()` | List of tuples |
| `cr.dictfetchone()` | Single dict or None |
| `cr.dictfetchall()` | List of dicts |
| `cr.rowcount` | Number of affected rows |

## Invalidate ORM cache after raw writes

```python
# If you write to DB with raw SQL, the ORM's in-memory cache is stale
self.env.cr.execute(
    "UPDATE crm_lead SET state = 'new' WHERE create_date < %s",
    (cutoff_date,)
)
# Tell ORM to forget cached values for this model
self.env['crm.lead'].invalidate_model()
```

## When to use raw SQL vs ORM

| Situation | Use |
|---|---|
| Standard CRUD | ORM always |
| Complex GROUP BY / aggregations | `read_group()` first, raw SQL if it can't do it |
| Bulk update without triggering `write()` hooks | Raw SQL + `invalidate_model()` |
| Cross-model joins for reporting | Raw SQL |
| Anything security-sensitive | ORM (respects access rules, raw SQL doesn't) |
