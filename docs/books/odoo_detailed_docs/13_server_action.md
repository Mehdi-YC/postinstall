# 13 — Server action

A bulk action that appears in the Action menu of a list view.
Great for "mark all selected as X" operations.

## Python method on the model

```python
def action_bulk_qualify(self):
    # self = recordset of selected records from the list view
    self.filtered(lambda r: r.state == 'new').write({
        'state': 'qualified'
    })
```

## XML declaration

```xml
<record id="action_bulk_qualify_leads" model="ir.actions.server">
  <field name="name">Mark as Qualified</field>
  <field name="model_id" ref="model_crm_lead"/>
  <!-- binding_model_id makes it appear in the Action menu -->
  <field name="binding_model_id" ref="model_crm_lead"/>
  <field name="state">code</field>
  <field name="code">records.action_bulk_qualify()</field>
</record>
```

> `binding_model_id` is the key. Without it the action exists but
> never appears in any list view.

## Getting selected records in the action

When a server action fires from a list view, Odoo injects the selected
record IDs into the context. Inside the `code` field, `records` is
automatically the selected recordset.

```python
# In code field: 'records' = selected recordset (same as self in a method)
records.write({'state': 'qualified'})

# Alternatively, call a method:
records.action_bulk_qualify()
```

## Server action that opens a wizard

```xml
<record id="action_server_open_wizard" model="ir.actions.server">
  <field name="name">Import from API</field>
  <field name="model_id" ref="model_crm_lead"/>
  <field name="binding_model_id" ref="model_crm_lead"/>
  <field name="state">code</field>
  <field name="code">
action = env['crm.import.wizard'].sudo().create({}).action_open()
  </field>
</record>
```

## Binding scope

```xml
<!-- binding_view_types: where it shows -->
<field name="binding_view_types">list</field>  <!-- only in list view -->
<field name="binding_view_types">list,form</field>  <!-- both -->
```
