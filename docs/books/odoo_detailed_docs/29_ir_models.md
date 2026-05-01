# 29 — ir.* models — Odoo's meta-layer

The `ir.*` models are Odoo's own infrastructure. You interact with them constantly.

## Overview

| Model | Stores / Does |
|---|---|
| `ir.model` | Registry of all installed models |
| `ir.model.fields` | Registry of all fields on all models |
| `ir.ui.view` | All views (form, list, kanban, search) |
| `ir.ui.menu` | All menu items |
| `ir.actions.act_window` | Window actions (open model views) |
| `ir.actions.server` | Server actions (run Python code) |
| `ir.actions.act_url` | URL actions |
| `ir.actions.report` | Report actions (PDF/XLSX) |
| `ir.rule` | Record rules — row-level access control |
| `ir.sequence` | Auto-increment sequences |
| `ir.config_parameter` | System-wide key/value settings |
| `ir.cron` | Scheduled jobs |
| `ir.attachment` | File attachments (any model) |
| `ir.translation` | Translations for all strings |
| `ir.module.module` | Installed modules registry |

## ir.rule — row-level access

```xml
<!-- Users can only see their own leads -->
<record id="rule_crm_lead_own" model="ir.rule">
  <field name="name">CRM Lead: own only</field>
  <field name="model_id" ref="model_crm_lead"/>
  <field name="domain_force">[('user_id', '=', user.id)]</field>
  <field name="groups" eval="[(4, ref('crm_custom.group_crm_user'))]"/>
  <field name="perm_read"   eval="True"/>
  <field name="perm_write"  eval="True"/>
  <field name="perm_create" eval="True"/>
  <field name="perm_unlink" eval="True"/>
</record>

<!-- Multi-company: only see your company's leads -->
<record id="rule_crm_lead_company" model="ir.rule">
  <field name="name">CRM Lead: company</field>
  <field name="model_id" ref="model_crm_lead"/>
  <!-- company_ids = all companies the current user belongs to -->
  <field name="domain_force">[('company_id', 'in', company_ids)]</field>
</record>
```

Magic variables in `domain_force`: `user` (current user record), `company_id` (current company), `company_ids` (list of company IDs).

## ir.attachment — file management

```python
import base64

# Attach a file to a record
attachment = self.env['ir.attachment'].create({
    'name':      'lead_report.pdf',
    'type':      'binary',
    'datas':     base64.b64encode(pdf_bytes),
    'res_model': 'crm.lead',
    'res_id':    self.id,
    'mimetype':  'application/pdf',
})

# Get all attachments for a record
attachments = self.env['ir.attachment'].search([
    ('res_model', '=', 'crm.lead'),
    ('res_id',    '=', self.id),
])

# Get attachment data
for att in attachments:
    raw = base64.b64decode(att.datas)
```

## ir.model.fields — dynamic field access

```python
# Check if a field exists on a model
field = self.env['ir.model.fields'].search([
    ('model', '=', 'crm.lead'),
    ('name',  '=', 'expected_revenue'),
])
if field:
    print(field.ttype)   # 'float'

# Get all fields of a model
fields = self.env['ir.model.fields'].search([
    ('model_id.model', '=', 'crm.lead'),
])
```

## ir.module.module — check installed modules

```python
# Check if a module is installed
def _is_sale_installed(self):
    return bool(self.env['ir.module.module'].search([
        ('name', '=', 'sale'),
        ('state', '=', 'installed'),
    ]))

# Use this to conditionally add features
if self._is_sale_installed():
    # add sale-related field
    pass
```
