# 16 — search / read / browse

## search()

```python
leads = self.env['crm.lead'].search(
    [('state', '=', 'new')],
    order='create_date desc',
    limit=10,
    offset=0,
)

# Count without fetching
count = self.env['crm.lead'].search_count([('state', '=', 'new')])

# Empty domain = all records
all_leads = self.env['crm.lead'].search([])
```

## browse() — when you already have IDs

```python
# No search query — directly wraps known IDs
leads = self.env['crm.lead'].browse([1, 2, 3])
lead  = self.env['crm.lead'].browse(42)
```

## read() — returns list of dicts (faster than ORM fields)

```python
data = leads.read(['name', 'state', 'expected_revenue'])
# [{'id': 1, 'name': 'Lead A', 'state': 'new', 'expected_revenue': 50000.0}]

# Great for: passing data to an API response, report generation
```

## search_read() — one DB call

```python
data = self.env['crm.lead'].search_read(
    domain=[('state', '=', 'new')],
    fields=['name', 'phone', 'email'],
    limit=50,
    order='name asc',
)
# Returns list of dicts directly
```

## read_group() — SQL GROUP BY

```python
groups = self.env['crm.lead'].read_group(
    domain=[],
    fields=['expected_revenue:sum', 'state'],
    groupby=['state'],
)
# [{'state': 'new', 'expected_revenue': 250000, 'state_count': 5}, ...]

# Multiple groupby (nested)
groups = self.env['crm.lead'].read_group(
    domain=[],
    fields=['expected_revenue:sum'],
    groupby=['category_id', 'state'],
    lazy=False,   # flatten all groups into one list
)
```

## When to use what

| Method | Use when |
|---|---|
| `search()` | You need ORM recordsets and will access fields via dot notation |
| `browse()` | You already have IDs (e.g. from context or another query) |
| `read()` | You need plain dicts, e.g. for JSON responses or reports |
| `search_read()` | You need filtered dicts in one call (API endpoints, wizards) |
| `read_group()` | Aggregations, counts, sums grouped by a field |

## exists() — check if record still alive

```python
lead = self.env['crm.lead'].browse(42)
if lead.exists():
    print(lead.name)
# Without exists(), accessing a deleted record returns empty values silently
```
