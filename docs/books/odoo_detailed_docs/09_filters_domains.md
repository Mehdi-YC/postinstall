# 09 — Filters & domains

## Domain syntax

A domain is a list of conditions. Each condition is a 3-tuple: `(field, operator, value)`.
Multiple conditions are ANDed by default. Prefix with `'|'` for OR.

```python
# Single condition
[('state', '=', 'new')]

# AND (implicit): new AND revenue > 10000
[('state', '=', 'new'), ('expected_revenue', '>', 10000)]

# OR: new OR qualified
['|', ('state', '=', 'new'), ('state', '=', 'qualified')]

# Nested: (new OR qualified) AND revenue > 5000
['|', ('state', '=', 'new'), ('state', '=', 'qualified'),
      ('expected_revenue', '>', 5000)]

# Relational: leads where partner is in Algeria
[('partner_id.country_id.code', '=', 'DZ')]

# In list
[('state', 'in', ['new', 'qualified'])]

# Not in
[('state', 'not in', ['won', 'lost'])]

# Case-insensitive contains
[('name', 'ilike', 'algiers')]

# Is set / not set
[('partner_id', '!=', False)]
[('partner_id', '=', False)]
```

## Operators

| Operator | Meaning |
|---|---|
| `=`, `!=` | Equal, not equal |
| `<`, `>`, `<=`, `>=` | Numeric comparisons |
| `like` | Contains (case sensitive, SQL LIKE) |
| `ilike` | Contains (case insensitive) |
| `=like` | Exact match, case sensitive |
| `=ilike` | Exact match, case insensitive |
| `in`, `not in` | Value is / isn't in list |
| `child_of` | All children in a hierarchy |
| `parent_of` | All parents in a hierarchy |

## Search view XML

```xml
<record id="crm_lead_search" model="ir.ui.view">
  <field name="name">crm.lead.search</field>
  <field name="model">crm.lead</field>
  <field name="arch" type="xml">
    <search>

      <!-- Searchable fields (appear in dropdown when user types) -->
      <field name="name" string="Lead"/>
      <field name="partner_id"/>
      <field name="category_id"/>

      <!-- Pre-defined filter buttons -->
      <filter name="filter_new"
              string="New Leads"
              domain="[('state','=','new')]"/>

      <!-- 'uid' = current user's ID -->
      <filter name="filter_my_leads"
              string="My Leads"
              domain="[('user_id','=',uid)]"/>

      <filter name="filter_high_revenue"
              string="High Revenue (&gt;50k)"
              domain="[('expected_revenue','&gt;',50000)]"/>

      <!-- Date filters -->
      <filter name="filter_this_month"
              string="This Month"
              domain="[('create_date','&gt;=', (datetime.datetime.now() - datetime.timedelta(days=30)).strftime('%Y-%m-%d'))]"/>

      <separator/>   <!-- visual divider in filter dropdown -->

      <!-- Group By options -->
      <filter name="group_state"
              string="Status"
              context="{'group_by': 'state'}"/>

      <filter name="group_category"
              string="Category"
              context="{'group_by': 'category_id'}"/>

      <filter name="group_month"
              string="Month"
              context="{'group_by': 'create_date:month'}"/>

    </search>
  </field>
</record>
```

## Domains in Python

```python
# In search()
leads = self.env['crm.lead'].search([
    ('state', '=', 'new'),
    ('user_id', '=', self.env.uid),
])

# In action definition
action['domain'] = [('category_id', '=', self.category_id.id)]

# Dynamic domain building
domain = [('state', 'not in', ['won', 'lost'])]
if self.env.context.get('only_mine'):
    domain += [('user_id', '=', self.env.uid)]
```
