# 17 — Model inheritance

One of the most important Odoo concepts — not the same as Python class inheritance.

## Type 1: Extension (same _name, no new table)

The most common. Adds fields/methods to an existing model.
The original table is modified with `ALTER TABLE`.

```python
# In your module: extend res.partner
class ResPartner(models.Model):
    _inherit = 'res.partner'    # no _name = extension

    lead_count = fields.Integer(compute='_compute_lead_count')
    crm_score  = fields.Integer(string='Lead Score', default=0)

    @api.depends('lead_ids')
    def _compute_lead_count(self):
        for rec in self:
            rec.lead_count = self.env['crm.lead'].search_count(
                [('partner_id', '=', rec.id)]
            )

    def write(self, vals):
        result = super().write(vals)
        if 'email' in vals:
            self._sync_with_crm()
        return result
```

## Type 2: Prototype (new _name, copies all fields)

Creates a NEW table. Copies all fields from the parent.
Rare — used when you want a variant of a model.

```python
class CrmLeadArchived(models.Model):
    _name    = 'crm.lead.archived'
    _inherit = 'crm.lead'           # copy all fields from crm.lead
    _description = 'Archived Lead'

    archived_date = fields.Date()
    archive_reason = fields.Text()
```

## Type 3: Delegation (_inherits — composition via FK)

One table embeds another via FK. Fields of the embedded model are
accessible directly on the outer model. Used by `res.users` → `res.partner`.

```python
class CrmContact(models.Model):
    _name     = 'crm.contact'
    _inherits = {'res.partner': 'partner_id'}   # delegate to partner

    partner_id = fields.Many2one('res.partner', required=True,
                                  ondelete='cascade')
    # Now crm.contact.name → partner_id.name (transparent)
    # crm.contact.email → partner_id.email
```

## Inheriting multiple mixins

```python
class CrmLead(models.Model):
    _name    = 'crm.lead'
    _inherit = ['mail.thread', 'mail.activity.mixin', 'portal.mixin']
```

Order matters if there are conflicting method names — left-most wins.

## Overriding methods — always call super()

```python
@api.model_create_multi
def create(self, vals_list):
    for vals in vals_list:
        vals.setdefault('source', 'manual')
    records = super().create(vals_list)
    records._post_create_actions()
    return records   # always return what super() returned

def write(self, vals):
    result = super().write(vals)
    return result    # write returns True, pass it along

def unlink(self):
    if self.filtered(lambda r: r.state == 'won'):
        raise UserError('Cannot delete Won leads.')
    return super().unlink()
```

> **Rule:** Always call `super()` unless you _intentionally_ want to
> block the original behavior. Not returning its value is a common silent bug.
