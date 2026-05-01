# 21 — Cron jobs

Scheduled automation — a Python method that runs automatically on a schedule.

## Python method

```python
@api.model
def _cron_auto_qualify_leads(self):
    """Mark leads as qualified if they have 3+ answered calls."""
    from datetime import timedelta
    cutoff = fields.Datetime.now() - timedelta(days=7)
    leads = self.search([
        ('state', '=', 'new'),
        ('create_date', '<=', cutoff),
    ])
    for lead in leads:
        answered = lead.call_ids.filtered('answered')
        if len(answered) >= 3:
            lead.with_context(mail_notrack=True).write({'state': 'qualified'})
```

## XML declaration

Place in `data/` folder and add to manifest.

```xml
<record id="cron_auto_qualify" model="ir.cron">
  <field name="name">CRM: Auto-qualify leads</field>
  <field name="model_id" ref="model_crm_lead"/>
  <field name="state">code</field>
  <field name="code">model._cron_auto_qualify_leads()</field>
  <field name="interval_number">1</field>
  <field name="interval_type">days</field>  <!-- minutes/hours/days/weeks/months -->
  <field name="numbercall">-1</field>        <!-- -1 = run forever -->
  <field name="active" eval="True"/>
  <field name="user_id" ref="base.user_root"/>  <!-- runs as admin by default -->
</record>
```

## interval_type values

| Value | Meaning |
|---|---|
| `minutes` | Every N minutes |
| `hours` | Every N hours |
| `days` | Every N days |
| `weeks` | Every N weeks |
| `months` | Every N months |

## Cron safety rules

- Crons run as admin (uid=1) by default — no user session.
- There is no `active_id` or `active_ids` context.
- Always use `self.search([...])` inside a cron, never rely on `self` being populated.
- Wrap in try/except if partial failure is acceptable:

```python
@api.model
def _cron_send_reminders(self):
    leads = self.search([('state', '=', 'new')])
    for lead in leads:
        try:
            lead._send_reminder_email()
        except Exception as e:
            _logger.error('Failed to send reminder for lead %s: %s', lead.id, e)
```

## Logging in crons

```python
import logging
_logger = logging.getLogger(__name__)

@api.model
def _cron_auto_qualify_leads(self):
    _logger.info('Starting auto-qualify cron')
    # ...
    _logger.info('Auto-qualify cron done: %d leads qualified', qualified_count)
```
