# 23 — Config parameters

## ir.config_parameter — system-wide key/value

```python
# Read a param (always sudo — normal users can't read system params)
url = self.env['ir.config_parameter'].sudo().get_param(
    'crm_custom.api_url',
    default='https://api.example.com'
)

# Write a param
self.env['ir.config_parameter'].sudo().set_param(
    'crm_custom.api_key', 'secret-value'
)
```

## Set a default in data XML

```xml
<record id="param_api_url" model="ir.config_parameter">
  <field name="key">crm_custom.api_url</field>
  <field name="value">https://api.example.com</field>
</record>
```

## res.config.settings — proper Settings UI page

Extend the Settings form to let admins configure your module.

```python
class ResConfigSettings(models.TransientModel):
    _inherit = 'res.config.settings'

    # config_parameter links field directly to ir.config_parameter
    # No manual get/set needed — Odoo handles it
    crm_api_url = fields.Char(
        string='CRM API URL',
        config_parameter='crm_custom.api_url'
    )
    crm_api_key = fields.Char(
        string='API Key',
        config_parameter='crm_custom.api_key'
    )
    # Boolean stored as company property (per-company setting)
    crm_auto_qualify = fields.Boolean(
        string='Auto-qualify leads',
        config_parameter='crm_custom.auto_qualify'
    )
```

## Settings view extension

```xml
<record id="res_config_settings_view_crm" model="ir.ui.view">
  <field name="name">res.config.settings.crm</field>
  <field name="model">res.config.settings</field>
  <field name="inherit_id" ref="base_setup.action_general_configuration"/>
  <field name="arch" type="xml">
    <xpath expr="//div[@id='integration_settings']" position="before">
      <app string="CRM Custom" name="crm_custom">
        <setting>
          <field name="crm_api_url"/>
          <div class="text-muted">URL of the external leads API</div>
        </setting>
        <setting>
          <field name="crm_auto_qualify"/>
        </setting>
      </app>
    </xpath>
  </field>
</record>
```

## Company-specific settings (res.company field)

For per-company config (not global), add the field to `res.company` and link it
via `related=`:

```python
class ResCompany(models.Model):
    _inherit = 'res.company'
    crm_api_url = fields.Char(string='CRM API URL')

class ResConfigSettings(models.TransientModel):
    _inherit = 'res.config.settings'
    crm_api_url = fields.Char(related='company_id.crm_api_url', readonly=False)
```
