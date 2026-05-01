# 12 — Wizard

A `TransientModel` — temporary, auto-deleted after the session.
Used for dialogs, import flows, multi-step forms.

## Key differences from Model

| | `models.Model` | `models.TransientModel` |
|---|---|---|
| DB table | Permanent | Exists, but rows auto-deleted |
| Data | Persistent | Lives only during the user session |
| Use case | Business data | Dialogs, import, config steps |

## Python class

```python
from odoo import api, fields, models
from odoo.exceptions import UserError


class ImportLeadsWizardLine(models.TransientModel):
    """Preview lines shown in the wizard table."""
    _name = 'crm.import.wizard.line'
    _description = 'Wizard Preview Line'

    wizard_id        = fields.Many2one('crm.import.wizard')
    name             = fields.Char(string='Lead Name')
    phone            = fields.Char()
    email            = fields.Char()
    expected_revenue = fields.Float()


class ImportLeadsWizard(models.TransientModel):
    _name        = 'crm.import.wizard'
    _description = 'Import Leads from API'

    category_id  = fields.Many2one('crm.lead.category',
                                    required=True, string='Category')
    preview_ids  = fields.One2many('crm.import.wizard.line',
                                    'wizard_id', string='Leads Preview')
    import_count = fields.Integer(compute='_compute_import_count')

    @api.depends('preview_ids')
    def _compute_import_count(self):
        for rec in self:
            rec.import_count = len(rec.preview_ids)

    @api.onchange('category_id')
    def _onchange_category(self):
        if not self.category_id:
            return
        data = self._fetch_from_api(self.category_id.code)
        self.preview_ids = [(5, 0, 0)]       # clear existing lines
        self.preview_ids = [
            (0, 0, {
                'name': lead['name'],
                'phone': lead['phone'],
                'email': lead['email'],
                'expected_revenue': lead['revenue'],
            })
            for lead in data
        ]

    def _fetch_from_api(self, category_code):
        """Mock — replace with requests.get() for production."""
        mock_data = {
            'tech': [
                {'name': 'TechCorp DZ', 'phone': '0550001111',
                 'email': 'info@techcorp.dz', 'revenue': 85000},
            ],
        }
        return mock_data.get(category_code, [])

    def action_import(self):
        if not self.preview_ids:
            raise UserError('No leads to import. Select a category first.')
        Lead = self.env['crm.lead']
        created = Lead.create([{
            'name':             line.name,
            'phone':            line.phone,
            'email':            line.email,
            'expected_revenue': line.expected_revenue,
            'category_id':      self.category_id.id,
            'source':           'api',
        } for line in self.preview_ids])

        return {
            'type':      'ir.actions.act_window',
            'name':      f'Imported {len(created)} Leads',
            'res_model': 'crm.lead',
            'view_mode': 'list,form',
            'domain':    [('id', 'in', created.ids)],
            'target':    'current',
        }
```

## Wizard view XML

```xml
<record id="crm_import_wizard_form" model="ir.ui.view">
  <field name="name">crm.import.wizard.form</field>
  <field name="model">crm.import.wizard</field>
  <field name="arch" type="xml">
    <form string="Import Leads from API">
      <sheet>
        <group>
          <field name="category_id"/>
          <field name="import_count" readonly="1" string="Leads found"/>
        </group>
        <field name="preview_ids" readonly="1">
          <list>
            <field name="name"/>
            <field name="phone"/>
            <field name="email"/>
            <field name="expected_revenue"/>
          </list>
        </field>
      </sheet>
      <footer>
        <button name="action_import" string="Import All"
                type="object" class="btn-primary"/>
        <button string="Cancel" class="btn-secondary" special="cancel"/>
      </footer>
    </form>
  </field>
</record>

<!-- Action to open wizard as a dialog -->
<record id="action_import_leads_wizard" model="ir.actions.act_window">
  <field name="name">Import Leads from API</field>
  <field name="res_model">crm.import.wizard</field>
  <field name="view_mode">form</field>
  <field name="target">new</field>  <!-- 'new' = popup dialog -->
</record>
```

## ORM commands for relational fields

| Command | Effect |
|---|---|
| `(0, 0, vals)` | Create and link a new record |
| `(1, id, vals)` | Update existing linked record |
| `(2, id, 0)` | Delete linked record from DB |
| `(3, id, 0)` | Unlink (Many2many only — remove link, keep record) |
| `(4, id, 0)` | Link existing record (Many2many) |
| `(5, 0, 0)` | Remove all links (clear the field) |
| `(6, 0, [ids])` | Replace entire link set with these ids |
