# 05 — Security

Two files: access rights (CSV) and optional record rules (XML).

## ir.model.access.csv

Every model you create needs a row here or Odoo raises an `AccessError` for all users.

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_crm_lead_user,crm.lead.user,model_crm_lead,base.group_user,1,1,1,0
access_crm_lead_manager,crm.lead.manager,model_crm_lead,base.group_system,1,1,1,1
access_crm_wizard,crm.wizard,model_crm_import_wizard,base.group_user,1,1,1,1
```

### Column rules

- `model_id:id` is always `model_` + the model's `_name` with dots replaced by underscores.
  So `crm.lead` → `model_crm_lead`.
- `group_id:id` can be empty (applies to all users), or an external ID like `base.group_user`.
- `perm_*` is `1` (allow) or `0` (deny).

| Column | Meaning |
|---|---|
| `perm_read` | Can open records / search |
| `perm_write` | Can edit and save |
| `perm_create` | Can create new records |
| `perm_unlink` | Can delete records |

## Groups in XML

```xml
<odoo>
  <record id="group_crm_user" model="res.groups">
    <field name="name">CRM User</field>
    <field name="category_id" ref="base.module_category_hidden"/>
  </record>
  <record id="group_crm_manager" model="res.groups">
    <field name="name">CRM Manager</field>
    <!-- Manager implies User rights -->
    <field name="implied_ids" eval="[(4, ref('group_crm_user'))]"/>
  </record>
</odoo>
```

## ir.rule — row-level access (record rules)

Record rules filter which *rows* a group can see, not which *columns*.
Useful for multi-company, "only see your own records", etc.

```xml
<record id="rule_crm_lead_own" model="ir.rule">
  <field name="name">CRM Lead: own leads only</field>
  <field name="model_id" ref="model_crm_lead"/>
  <field name="domain_force">[('user_id', '=', user.id)]</field>
  <field name="groups" eval="[(4, ref('group_crm_user'))]"/>
  <!-- perm_read=1 (default), leave others default to apply on all ops -->
</record>

<!-- Multi-company rule -->
<record id="rule_crm_lead_company" model="ir.rule">
  <field name="name">CRM Lead: company</field>
  <field name="model_id" ref="model_crm_lead"/>
  <field name="domain_force">[('company_id', 'in', company_ids)]</field>
</record>
```

> `user` and `company_ids` are magic variables available in domain_force expressions.

## Visibility in views (not security)

```xml
<!-- groups= hides the field/button from users not in that group -->
<field name="expected_revenue" groups="crm_custom.group_crm_manager"/>
<button name="action_delete_all" groups="base.group_system"/>
```

> This is UI-only. It hides the element but does NOT block API access.
> Always combine with proper access rights in the CSV.
