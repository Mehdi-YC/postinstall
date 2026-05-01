# 15 — Recordsets

`self` is always a recordset — a list-like object of zero or more records of the same model.

## Basics

```python
# self can be 0, 1, or many records
# When iterating, each element is also a recordset (1 record)
for rec in self:
    rec.name   # safe — exactly one record

# Direct field access on self is only safe if len(self) == 1
self.ensure_one()   # raises ValueError if len != 1
self.name           # now safe

# Size
len(self)           # number of records
bool(self)          # False if empty recordset
if self:            # idiom: "if any records"
    ...

# IDs shortcut
self.ids            # [1, 2, 3] — list of ints
```

## Set operations

```python
a | b              # union (combines, deduplicates)
a & b              # intersection
a - b              # difference (records in a but not b)
a == b             # same records?
rec in recordset   # membership check
```

## Filtering

```python
# filtered() — returns recordset, no DB call
new_leads = leads.filtered(lambda r: r.state == 'new')

# Filter by truthy field
with_phone = leads.filtered('phone')

# Chain filters
high_new = leads.filtered(lambda r: r.state == 'new' and r.expected_revenue > 10000)
```

## Mapping

```python
# mapped() on a field → list or recordset
names    = leads.mapped('name')           # ['Lead A', 'Lead B'] — list of str
partners = leads.mapped('partner_id')     # recordset of partners (deduplicated)

# mapped() on a lambda → list
totals = leads.mapped(lambda r: r.expected_revenue * 1.2)
```

## Sorting

```python
# sorted() — returns a list (not recordset)
sorted_leads = leads.sorted('expected_revenue', reverse=True)
sorted_leads = leads.sorted(lambda r: r.name.lower())

# sorted() returns list, wrap with browse() to get recordset back
sorted_rs = self.browse([r.id for r in leads.sorted('name')])
```

## Accumulating records

```python
# Build a recordset progressively
result = self.env['crm.lead']   # empty recordset
for rec in leads:
    if rec.call_count >= 3:
        result |= rec           # union-add
result.write({'state': 'qualified'})  # one DB call on all
```

## Common mistakes

```python
# WRONG: field access on multi-record set raises
leads = self.search([('state', '=', 'new')])
leads.name   # Error: Expected singleton

# CORRECT: iterate or ensure_one
for lead in leads:
    print(lead.name)  # ok

# or
single_lead.ensure_one()
single_lead.name
```
