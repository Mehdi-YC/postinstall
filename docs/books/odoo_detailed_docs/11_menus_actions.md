# 11 — Menus & actions

Actions define what opens. Menus are the links that trigger actions.

## Window action (opens a view)

```xml
<record id="action_crm_lead" model="ir.actions.act_window">
  <field name="name">Leads</field>
  <field name="res_model">crm.lead</field>
  <field name="view_mode">kanban,list,form</field>
  <field name="domain">[]</field>
  <field name="context">{'default_source': 'manual'}</field>
  <field name="help" type="html">
    <p class="o_view_nocontent_smiling_face">
      No leads yet. Create one or import from the API.
    </p>
  </field>
</record>
```

## Menu hierarchy

```xml
<!-- 1. App root (top navigation bar) -->
<menuitem id="menu_crm_root"
          name="CRM"
          sequence="10"/>

<!-- 2. Category submenu -->
<menuitem id="menu_crm_sales"
          name="Sales"
          parent="menu_crm_root"
          sequence="10"/>

<!-- 3. Leaf item: links to an action -->
<menuitem id="menu_crm_leads"
          name="Leads"
          parent="menu_crm_sales"
          action="action_crm_lead"
          sequence="10"/>

<!-- 4. Wizard launch -->
<menuitem id="menu_crm_import"
          name="Import from API"
          parent="menu_crm_sales"
          action="action_import_leads_wizard"
          sequence="20"/>

<!-- 5. Config section -->
<menuitem id="menu_crm_config"
          name="Configuration"
          parent="menu_crm_root"
          sequence="90"/>
```

## Action types

| Model | Opens |
|---|---|
| `ir.actions.act_window` | A model's views (form, list, kanban...) |
| `ir.actions.server` | Python code (server actions, bulk actions) |
| `ir.actions.act_url` | An external URL |
| `ir.actions.client` | A frontend client action (Owl component) |
| `ir.actions.report` | A report (PDF/XLSX) |

## Returning actions from Python

```python
# Open a specific record
def action_open_form(self):
    self.ensure_one()
    return {
        'type':      'ir.actions.act_window',
        'res_model': 'crm.lead',
        'res_id':    self.id,
        'view_mode': 'form',
        'target':    'current',   # 'new' = popup dialog
    }

# Open a filtered list
def action_open_won(self):
    return {
        'type':      'ir.actions.act_window',
        'res_model': 'crm.lead',
        'view_mode': 'list,form',
        'domain':    [('state', '=', 'won')],
        'name':      'Won Leads',
    }

# Open URL
def action_open_website(self):
    return {
        'type':   'ir.actions.act_url',
        'url':    'https://example.com',
        'target': 'new',
    }

# Reload current view
def action_reload(self):
    return {'type': 'ir.actions.client', 'tag': 'reload'}
```
