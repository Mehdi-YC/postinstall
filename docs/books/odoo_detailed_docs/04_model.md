# 04 — Model fields & methods

A Model is a Python class that maps to a DB table. Fields are class attributes. Methods are business logic.

## Class skeleton

```python
from odoo import api, fields, models
from odoo.exceptions import ValidationError

class CrmLead(models.Model):
    _name        = 'crm.lead'        # dot-separated → table crm_lead
    _description = 'Lead'            # human label
    _inherit     = ['mail.thread', 'mail.activity.mixin']
    _order       = 'create_date desc'
    _rec_name    = 'name'            # field used as display name
```

## Field types

| Field | Python type | Notes |
|---|---|---|
| `Char` | str | Single line, has `size=` |
| `Text` | str | Multi-line |
| `Html` | str | Rich text editor |
| `Integer` | int | Whole number |
| `Float` | float | Has `digits=(16,2)` |
| `Boolean` | bool | Checkbox |
| `Date` | date | Odoo auto-converts |
| `Datetime` | datetime | Odoo auto-converts |
| `Selection` | str | Dropdown: `selection=[('a','A')]` |
| `Many2one` | int (FK) | Dropdown to another model |
| `One2many` | list | Inline table, needs `inverse_name` |
| `Many2many` | list | Tags widget |

## Common field options

```python
name = fields.Char(
    string='Lead Name',     # label in UI
    required=True,          # NOT NULL
    readonly=True,          # can't edit in UI
    copy=False,             # not copied on duplicate
    tracking=True,          # log changes in chatter
    default='New',          # default value
    index=True,             # add DB index
)
```

## Computed fields

```python
call_count = fields.Integer(
    compute='_compute_call_count',
    store=True    # store=True saves to DB (searchable/sortable)
                  # store=False (default) recomputes on every read
)

@api.depends('call_ids')   # recomputes when call_ids changes
def _compute_call_count(self):
    for rec in self:
        rec.call_count = len(rec.call_ids)
```

## Onchange (UI only)

```python
@api.onchange('partner_id')
def _onchange_partner(self):
    # Runs in the browser when user changes partner_id
    # self.id may be False (record not saved yet)
    if self.partner_id:
        self.phone = self.partner_id.phone
        self.email = self.partner_id.email

    # Optionally return a warning
    if self.partner_id and not self.partner_id.email:
        return {'warning': {
            'title': 'No email',
            'message': 'This customer has no email address.',
        }}
```

> **Important:** `@api.onchange` only fires in the UI. It does NOT run during
> `create()` or `write()`. If you need the same logic on save, override `write()`.

## Constraints

```python
# SQL constraint — enforced at DB level (fastest)
_sql_constraints = [
    ('name_unique', 'UNIQUE(name)', 'Lead name must be unique!'),
]

# Python constraint — more flexible logic
@api.constrains('expected_revenue')
def _check_revenue(self):
    for rec in self:
        if rec.expected_revenue < 0:
            raise ValidationError('Revenue cannot be negative.')
```

## Override CRUD

```python
@api.model_create_multi
def create(self, vals_list):
    for vals in vals_list:
        if not vals.get('ref'):
            vals['ref'] = self.env['ir.sequence'].next_by_code('crm.lead')
    return super().create(vals_list)

def write(self, vals):
    result = super().write(vals)
    if 'state' in vals:
        self._on_state_change()
    return result   # always return what super() returned

def unlink(self):
    if self.filtered(lambda r: r.state == 'won'):
        raise UserError('Cannot delete Won leads.')
    return super().unlink()
```

## Useful `_` class attributes

| Attribute | Purpose |
|---|---|
| `_name` | Model technical name (required) |
| `_description` | Human-readable label (required) |
| `_inherit` | List of mixins or model to extend |
| `_inherits` | Dict for delegation inheritance |
| `_order` | Default sort order: `'create_date desc'` |
| `_rec_name` | Field used as display name (default: `name`) |
| `_sql_constraints` | List of SQL-level constraints |
| `_table` | Override DB table name (rarely needed) |
| `_log_access` | Set False to skip create_date/write_date/uid fields |
