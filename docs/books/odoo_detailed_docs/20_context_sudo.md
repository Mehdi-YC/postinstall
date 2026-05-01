# 20 — Context & sudo

## Reading context

```python
lang       = self.env.context.get('lang', 'en_US')
active_id  = self.env.context.get('active_id')    # ID of record in list
active_ids = self.env.context.get('active_ids', []) # selected IDs in list view
tz         = self.env.context.get('tz')
```

## Injecting context

```python
# Add keys to context without losing existing ones
self.with_context(mail_notrack=True).write({'state': 'new'})

# Merge with existing context (safe pattern)
ctx = dict(self.env.context, default_state='new', default_source='api')
Lead = self.env['crm.lead'].with_context(**ctx)
Lead.create({'name': 'New Lead'})   # state and source are pre-filled
```

## Common context flags

| Flag | Effect |
|---|---|
| `mail_notrack` | Skip field tracking / chatter log for this write |
| `mail_nolog` | Skip the "Created" chatter message |
| `mail_auto_subscribe_no_notify` | Don't email new followers |
| `tracking_disable` | Disable all tracking for this operation |
| `no_recompute` | Skip computed field recompute |
| `default_field_name` | Pre-fill `field_name` on new records via action |
| `active_test` | Set to False to include archived records in search |

## sudo() patterns

```python
# sudo() with no args = run as admin (uid=1)
partner = self.env['res.partner'].sudo().browse(partner_id)

# sudo() then drop it for writes
def action_read_all_write_own(self):
    all_leads = self.env['crm.lead'].sudo().search([])  # read all (bypass access)
    # Write only with original user's rights
    my_leads = all_leads.filtered(lambda r: r.user_id == self.env.user)
    my_leads.write({'state': 'qualified'})   # no sudo here

# with_user() — run as a specific user (not admin)
other_user = self.env.ref('base.user_demo')
self.with_user(other_user).create({'name': 'Test'})
```

## active_test — include archived records

```python
# By default, search() excludes records with active=False
leads = self.env['crm.lead'].search([])          # only active=True

# Include archived (active=False) records
all_leads = self.env['crm.lead'].with_context(active_test=False).search([])
```
