# 28 — Data XML tricks

## noupdate — don't overwrite on upgrade

```xml
<odoo>
  <data noupdate="1">
    <!-- Created once on install, never touched on upgrade -->
    <!-- Good for: default stages, categories users might edit -->
    <record id="cat_tech" model="crm.lead.category">
      <field name="name">Technology</field>
      <field name="code">tech</field>
    </record>
  </data>
  <data>
    <!-- Updated on every upgrade -->
    <!-- Good for: sequences, crons, server actions, security rules -->
    <record id="seq_crm_lead" model="ir.sequence">
      <field name="name">CRM Lead</field>
      <field name="code">crm.lead</field>
    </record>
  </data>
</odoo>
```

## eval — Python expressions in XML

```xml
<!-- Today + 30 days -->
<field name="date_deadline"
       eval="(datetime.now() + timedelta(days=30)).strftime('%Y-%m-%d')"/>

<!-- Reference to another record's ID -->
<field name="group_id" eval="ref('base.group_user')"/>

<!-- Boolean -->
<field name="active" eval="True"/>
<field name="active" eval="False"/>

<!-- Many2many command: link multiple records -->
<field name="implied_ids" eval="[(4, ref('base.group_user'))]"/>

<!-- Set a list of IDs -->
<field name="category_ids" eval="[(6, 0, [ref('cat_tech'), ref('cat_retail')])]"/>
```

## ref — stable record references

```xml
<!-- Full external ID: module.xml_id -->
<field name="group_id" ref="base.group_user"/>
<field name="model_id" ref="model_crm_lead"/>

<!-- From same module (no prefix needed) -->
<field name="category_id" ref="cat_tech"/>
```

## delete — remove a record from another module

```xml
<!-- Carefully remove an ir.rule or menuitem from base -->
<delete model="ir.ui.menu" id="base.menu_email_all_leads"/>
```

## function — call a Python method at load time

```xml
<function model="res.partner" name="_update_partner_defaults"/>
```

## Updating existing records (no id conflict)

```xml
<!-- Use the existing record's external ID to update it -->
<record id="base.res_lang_fr" model="res.lang">
  <field name="active" eval="True"/>
</record>
```

## Active flag — archiving records

```xml
<!-- Many models have active=True by default -->
<!-- Set active=False to archive (hide) the record -->
<record id="cat_deprecated" model="crm.lead.category">
  <field name="name">Deprecated</field>
  <field name="active" eval="False"/>
</record>
```

## Tips

- Load order in manifest matters — reference what's defined before.
- External IDs are `module.xml_id`. Within the same module, just `xml_id`.
- Use `noupdate="1"` for any data users are expected to customize.
- Use `noupdate="0"` (default) for technical records like crons and sequences.
