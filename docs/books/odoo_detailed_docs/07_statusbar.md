# 07 — Status bar

The pipeline indicator shown at the top of a form. Clicking a stage moves the record.

## In the model

```python
state = fields.Selection([
    ('new',       'New'),
    ('qualified', 'Qualified'),
    ('proposal',  'Proposal'),
    ('won',       'Won'),
    ('lost',      'Lost'),
], default='new', string='Status', tracking=True)
# tracking=True logs every state change in the chatter automatically
```

## In the view

```xml
<header>
  <!-- Button visible only when state = 'new' -->
  <button name="action_qualify"
          string="Mark as Qualified"
          type="object"
          class="oe_highlight"
          invisible="state != 'new'"/>

  <button name="action_won"
          string="Won"
          type="object"
          invisible="state in ['won', 'lost']"/>

  <button name="action_lost"
          string="Lost"
          type="object"
          invisible="state in ['won', 'lost']"/>

  <!-- statusbar_visible: only show these stages in the bar -->
  <!-- 'proposal' will be reachable but not shown as a step -->
  <field name="state"
         widget="statusbar"
         statusbar_visible="new,qualified,won,lost"/>
</header>
```

## Button handlers in Python

```python
def action_qualify(self):
    self.write({'state': 'qualified'})

def action_won(self):
    self.write({'state': 'won'})

def action_lost(self):
    self.write({'state': 'lost'})
```

## Clickable status bar (stage-based)

If you want a Many2one to stages instead of a Selection (like CRM pipeline stages),
use `statusbar` widget on the Many2one:

```xml
<field name="stage_id" widget="statusbar" options="{'clickable': '1'}"/>
```

The stage model needs a `sequence` field for ordering and a `fold` field
to collapse stages in kanban.
