# 27 — Extending existing modules

The right way to customize Odoo: always extend, never patch core files.

## Extend a model

```python
# In your module: add fields to res.partner
class ResPartner(models.Model):
    _inherit = 'res.partner'   # no _name — extension, not new model

    lead_count = fields.Integer(compute='_compute_lead_count', store=True)
    crm_score  = fields.Integer(string='Lead Score', default=0)

    @api.depends('lead_ids')
    def _compute_lead_count(self):
        # Use search_count for efficiency
        Lead = self.env['crm.lead']
        for rec in self:
            rec.lead_count = Lead.search_count([('partner_id', '=', rec.id)])
```

## Extend a view with xpath

```xml
<record id="partner_form_inherit_crm" model="ir.ui.view">
  <field name="name">res.partner.form.crm</field>
  <field name="model">res.partner</field>
  <field name="inherit_id" ref="base.view_partner_form"/>
  <field name="arch" type="xml">

    <!-- Add after the phone field -->
    <xpath expr="//field[@name='phone']" position="after">
      <field name="lead_count"/>
      <field name="crm_score"/>
    </xpath>

    <!-- Add a new tab to the notebook -->
    <xpath expr="//notebook" position="inside">
      <page string="CRM" name="crm_tab">
        <field name="lead_ids">
          <list>
            <field name="name"/>
            <field name="state"/>
            <field name="expected_revenue"/>
          </list>
        </field>
      </page>
    </xpath>

    <!-- Modify an attribute -->
    <xpath expr="//field[@name='name']" position="attributes">
      <attribute name="required">1</attribute>
    </xpath>

  </field>
</record>
```

## xpath position values

| Position | Effect |
|---|---|
| `after` | Insert after the matched element |
| `before` | Insert before the matched element |
| `inside` | Append inside the matched element (at the end) |
| `replace` | Replace the matched element entirely |
| `attributes` | Modify XML attributes of the matched element |

## Simpler: use field name as selector

```xml
<!-- Instead of xpath, reference the field directly -->
<field name="phone" position="after">
  <field name="lead_count"/>
</field>
```

This is less fragile than xpath if the view structure changes.

## Priority — control which view wins

```xml
<record id="my_partner_form" model="ir.ui.view">
  <field name="inherit_id" ref="base.view_partner_form"/>
  <field name="priority">20</field>  <!-- higher = applied later, wins conflicts -->
  ...
</record>
```

Default priority is 16. Use higher values to override other extensions.

## Extending a method

```python
# Always call super() and return its result
class SaleOrder(models.Model):
    _inherit = 'sale.order'

    def action_confirm(self):
        # Do something before
        for order in self:
            order._sync_crm_leads()
        # Run original logic
        result = super().action_confirm()
        # Do something after
        return result
```
