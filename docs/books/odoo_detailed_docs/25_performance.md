# 25 — Performance & N+1 queries

## The N+1 problem

```python
# BAD: 1 query for search + 1 query per lead = N+1
leads = self.search([('state', '=', 'new')])
for lead in leads:
    print(lead.partner_id.name)   # DB query on EACH iteration

# GOOD: ORM batches the related field access
# Access partner_id on the full recordset first → 1 query for all partners
partners = leads.mapped('partner_id')
for partner in partners:
    print(partner.name)           # reads from cache, no extra queries
```

## How ORM prefetching works

```python
# Odoo batches reads when you access the SAME field on a recordset
leads = self.search([('state', '=', 'new')])
for lead in leads:
    _ = lead.name    # first access on the loop triggers 1 batch fetch for all
    # subsequent leads read from cache
```

## Force prefetch with read()

```python
# Explicitly load fields into cache before the loop
leads.read(['name', 'partner_id', 'state', 'expected_revenue'])
# Now all field accesses below are from cache — zero extra queries
for lead in leads:
    print(lead.name, lead.state)
```

## Bulk write vs loop write

```python
# BAD: N queries
for lead in leads:
    lead.write({'state': 'qualified'})

# GOOD: 1 query
leads.write({'state': 'qualified'})

# BAD: N creates
for vals in vals_list:
    Lead.create(vals)

# GOOD: 1 query (batch create)
Lead.create(vals_list)
```

## Computed fields: store=True vs store=False

```python
# store=False (default): recomputed on EVERY read — bad for large datasets
call_count = fields.Integer(compute='_compute_call_count')

# store=True: computed once, saved to DB, recomputed only when deps change
call_count = fields.Integer(compute='_compute_call_count', store=True)
# Use store=True when:
# - Field is used in list views (sorted/filtered)
# - Field is searched in domains
# - Computation is expensive
```

## read_group() instead of looping

```python
# BAD: loop + len() for each category
result = {}
for category in categories:
    count = self.search_count([('category_id', '=', category.id)])
    result[category.name] = count

# GOOD: single read_group query
groups = self.read_group(
    domain=[],
    fields=['category_id'],
    groupby=['category_id'],
)
result = {g['category_id'][1]: g['category_id_count'] for g in groups}
```

## Logging slow queries

```python
import logging
_logger = logging.getLogger(__name__)

# Add to odoo.conf for development:
# log_level = debug_sql   ← shows all SQL queries in logs
```

## General rules

1. **Write on recordsets, not in loops** — `leads.write(...)` not `for l in leads: l.write(...)`
2. **Create in batch** — pass a list to `create()`, not one dict at a time
3. **Use `read_group()`** for aggregations instead of Python loops
4. **Use `search_read()`** when you need dicts, not full ORM objects
5. **Use `store=True`** on computed fields that appear in list views
6. **Use raw SQL** only for cross-model reports or bulk updates without hooks
