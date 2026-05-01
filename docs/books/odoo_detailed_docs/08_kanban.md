# 08 — Kanban view

Cards grouped by stage/state. The most visual Odoo view.

## Full kanban view

```xml
<record id="crm_lead_kanban" model="ir.ui.view">
  <field name="name">crm.lead.kanban</field>
  <field name="model">crm.lead</field>
  <field name="arch" type="xml">
    <kanban default_group_by="state"
            class="o_kanban_small_column">

      <!-- Fields available inside the card template -->
      <field name="name"/>
      <field name="partner_id"/>
      <field name="expected_revenue"/>
      <field name="state"/>
      <field name="activity_ids"/>

      <templates>
        <t t-name="card">
          <!-- oe_kanban_global_click: clicking anywhere opens the form -->
          <div class="oe_kanban_global_click">

            <strong><field name="name"/></strong>

            <div class="text-muted">
              <field name="partner_id"/>
            </div>

            <div class="oe_kanban_bottom_right">
              <field name="expected_revenue" widget="monetary"/>
            </div>

            <!-- Activity bubbles (automatic with mail module) -->
            <div class="oe_kanban_footer">
              <field name="activity_ids" widget="kanban_activity"/>
            </div>

          </div>
        </t>
      </templates>
    </kanban>
  </field>
</record>
```

## Key kanban attributes

| Attribute | Effect |
|---|---|
| `default_group_by` | Field to group columns by (usually Selection or Many2one) |
| `class="o_kanban_small_column"` | Narrower columns |
| `quick_create="0"` | Disable quick-create in column header |
| `records_draggable="0"` | Disable drag between columns |

## Kanban with color

```xml
<field name="color"/>  <!-- Integer field named 'color' -->

<!-- In card template -->
<div t-attf-class="oe_kanban_color_#{kanban_getcolor(record.color.raw_value)}">
  ...
</div>
```

## `default_group_by` tips

- Works best with `Selection` (fixed columns) or `Many2one` with few records.
- Users can override group-by from the search bar.
- For `Selection`, Odoo creates one column per option, including empty.
