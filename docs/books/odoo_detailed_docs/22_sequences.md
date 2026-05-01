# 22 — Sequences

Auto-generated reference codes like LEAD/2025/00001.

## XML definition

```xml
<record id="seq_crm_lead" model="ir.sequence">
  <field name="name">CRM Lead Reference</field>
  <field name="code">crm.lead</field>       <!-- used to call it in Python -->
  <field name="prefix">LEAD/%(year)s/</field>
  <field name="padding">5</field>           <!-- LEAD/2025/00001 -->
  <field name="number_increment">1</field>
  <field name="number_next">1</field>
</record>
```

## Available prefix variables

| Variable | Value |
|---|---|
| `%(year)s` | 4-digit year (2025) |
| `%(month)s` | 2-digit month (01–12) |
| `%(day)s` | 2-digit day (01–31) |
| `%(doy)s` | Day of year (001–366) |
| `%(woy)s` | Week of year |
| `%(weekday)s` | Weekday (0=Monday) |
| `%(h24)s` | Hour 00–23 |
| `%(min)s` | Minutes 00–59 |
| `%(sec)s` | Seconds 00–59 |

## Python usage

```python
ref = fields.Char(string='Reference', readonly=True, copy=False, default='New')

@api.model_create_multi
def create(self, vals_list):
    for vals in vals_list:
        if vals.get('ref', 'New') == 'New':
            vals['ref'] = self.env['ir.sequence'].next_by_code('crm.lead')
    return super().create(vals_list)
```

## Tips

- Use `copy=False` on the ref field so duplicated records get a new sequence number.
- Set `readonly=True` so users can't edit it.
- Set `default='New'` as a placeholder before the sequence is assigned.
