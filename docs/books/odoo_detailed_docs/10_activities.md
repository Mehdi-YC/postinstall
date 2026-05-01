# 10 — Activities

Scheduled tasks (call, email, meeting) shown on records. Free with the `mail` module.

## Enable on your model

```python
class CrmLead(models.Model):
    _name    = 'crm.lead'
    _inherit = ['mail.thread', 'mail.activity.mixin']
    # One line gives you: chatter, followers, activities, message log, tracking
```

The fields `activity_ids`, `message_ids`, `message_follower_ids` come from the
mixins — you do NOT define them yourself.

## In the form view

```xml
<div class="oe_chatter">
  <field name="message_follower_ids"/>  <!-- Follow button -->
  <field name="activity_ids"/>          <!-- Activities widget -->
  <field name="message_ids"/>           <!-- Message thread -->
</div>
```

## Schedule an activity from Python

```python
from datetime import timedelta
from odoo import fields

def schedule_followup_call(self):
    activity_type = self.env.ref('mail.mail_activity_data_call')
    self.activity_schedule(
        activity_type_id=activity_type.id,
        summary='Follow-up call',
        date_deadline=fields.Date.today() + timedelta(days=2),
        user_id=self.user_id.id or self.env.uid,
    )
```

## Built-in activity types (mail module)

| External ID | Label |
|---|---|
| `mail.mail_activity_data_call` | Phone Call |
| `mail.mail_activity_data_email` | Email |
| `mail.mail_activity_data_meeting` | Meeting |
| `mail.mail_activity_data_todo` | To-Do |
| `mail.mail_activity_data_upload_document` | Upload Document |

## Custom activity type

```xml
<record id="activity_type_demo" model="mail.activity.type">
  <field name="name">Demo Call</field>
  <field name="icon">fa-phone</field>
  <field name="res_model">crm.lead</field>  <!-- restrict to this model -->
  <field name="delay_count">1</field>       <!-- default deadline: 1 day -->
  <field name="delay_unit">days</field>
</record>
```

## Useful activity methods

```python
# Schedule
self.activity_schedule(activity_type_id=..., summary=..., date_deadline=...)

# Mark done
self.activity_feedback(activity_types=activity_type, feedback='Called back')

# Cancel all activities of a type
self.activity_unlink([activity_type.id])

# Check for overdue activities
overdue = self.filtered(lambda r: r.activity_ids.filtered(
    lambda a: a.date_deadline < fields.Date.today()
))
```
