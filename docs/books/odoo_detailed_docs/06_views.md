# 06 — Views & tabs

Views are XML records stored in `ir.ui.view`. The form view uses `<notebook>` for tabs.

## View types

| View | Used for |
|---|---|
| `form` | Single record, full editing |
| `list` (was `tree`) | Multiple records, table format |
| `kanban` | Cards grouped by a field |
| `search` | Defines searchbar filters and group-by options |
| `pivot` | Spreadsheet-like aggregation |
| `graph` | Bar/line/pie charts |
| `calendar` | Timeline view (needs a Date field) |

## Form view with 3 tabs

```xml
<record id="crm_lead_form" model="ir.ui.view">
  <field name="name">crm.lead.form</field>
  <field name="model">crm.lead</field>
  <field name="arch" type="xml">
    <form>

      <!-- STATUS BAR -->
      <header>
        <button name="action_qualify" string="Qualify"
                type="object" class="oe_highlight"
                invisible="state != 'new'"/>
        <field name="state" widget="statusbar"
               statusbar_visible="new,qualified,won,lost"/>
      </header>

      <sheet>
        <!-- Two-column layout -->
        <group>
          <group>
            <field name="name"/>
            <field name="partner_id"/>
          </group>
          <group>
            <field name="expected_revenue"/>
            <field name="user_id"/>
          </group>
        </group>

        <!-- TABS -->
        <notebook>

          <page string="Calls" name="calls_tab">
            <field name="call_ids">
              <list editable="bottom">
                <field name="date"/>
                <field name="duration"/>
                <field name="answered"/>
                <field name="response"/>
              </list>
            </field>
          </page>

          <page string="General Info" name="general_tab">
            <group>
              <group string="Contact">
                <field name="phone" widget="phone"/>
                <field name="email" widget="email"/>
                <field name="website" widget="url"/>
              </group>
              <group string="Classification">
                <field name="category_id"/>
                <field name="source"/>
                <field name="create_date"/>
              </group>
            </group>
            <field name="notes" placeholder="Internal notes..."/>
          </page>

          <page string="Interests" name="interests_tab">
            <field name="interest_ids" widget="many2many_tags"/>
            <field name="interest_notes"/>
          </page>

        </notebook>
      </sheet>

      <!-- CHATTER -->
      <div class="oe_chatter">
        <field name="message_follower_ids"/>
        <field name="activity_ids"/>
        <field name="message_ids"/>
      </div>

    </form>
  </field>
</record>
```

## Visibility control (Odoo 17+ syntax)

```xml
<!-- New syntax: Python-like expressions -->
<button invisible="state != 'new'"/>
<field name="lost_reason" invisible="state != 'lost'"/>
<field name="revenue" invisible="state in ['new', 'lost']"/>

<!-- Old syntax (still works but deprecated) -->
<button attrs="{'invisible': [('state', '!=', 'new')]}"/>
```

## Common widgets

| Widget | Field type | Renders as |
|---|---|---|
| `statusbar` | Selection | Top pipeline bar |
| `badge` | Selection | Colored pill badge |
| `many2many_tags` | Many2many | Tag bubbles |
| `monetary` | Float | Currency formatted |
| `phone` | Char | Clickable phone link |
| `email` | Char | Clickable mailto link |
| `url` | Char | Clickable URL link |
| `html` | Html | Rich text editor |
| `image` | Binary | Image with upload |
| `priority` | Selection | Star rating |
| `kanban_activity` | Many2many | Activity circles on kanban |

## List view

```xml
<record id="crm_lead_list" model="ir.ui.view">
  <field name="name">crm.lead.list</field>
  <field name="model">crm.lead</field>
  <field name="arch" type="xml">
    <!-- decoration-* classes color rows conditionally -->
    <list decoration-success="state == 'won'"
          decoration-danger="state == 'lost'"
          decoration-muted="state == 'lost'">
      <field name="name"/>
      <field name="partner_id"/>
      <field name="expected_revenue"/>
      <field name="state" widget="badge"
             decoration-success="state == 'won'"
             decoration-warning="state == 'new'"
             decoration-danger="state == 'lost'"/>
      <field name="activity_ids" widget="list_activity"/>
    </list>
  </field>
</record>
```
