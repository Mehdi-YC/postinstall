# 14 — env & self

Every method in a Model has access to `self` (the recordset) and `self.env` (the environment).

## self.env — what it gives you

```python
# Access any model
Lead    = self.env['crm.lead']
Partner = self.env['res.partner']

# Current user
self.env.user          # res.users record
self.env.uid           # user id (int)
self.env.user.name     # 'Admin'
self.env.user.company_id

# Current company
self.env.company       # res.company record
self.env.companies     # all active companies (multi-company)

# Context (dict)
self.env.context       # {'lang': 'en_US', 'tz': 'Africa/Algiers', ...}

# Get a record by XML external ID
self.env.ref('mail.mail_activity_data_call')

# Raw DB cursor (use sparingly)
self.env.cr
```

## Changing the environment

```python
# sudo() — bypass access rights (run as superuser uid=1)
self.env['crm.lead'].sudo().search([])

# with_user() — run as a specific user
self.with_user(self.env.ref('base.user_admin'))

# with_context() — inject context values
self.with_context(no_recompute=True).write({...})
self.with_context(lang='ar_001').name

# with_company() — switch active company
self.with_company(company_id)
```

## sudo() — use responsibly

```python
# GOOD: sudo only for the read, not the whole flow
def action_do_thing(self):
    all_leads = self.env['crm.lead'].sudo().search([])  # bypass access for read
    # Drop sudo for the write — user's permissions apply
    all_leads.sudo(False).write({'state': 'new'})

# BAD: blanket sudo on the whole recordset
def action_do_thing(self):
    self.sudo().write({'state': 'new'})  # hides missing permissions
```

> Use `sudo()` only when you genuinely need to bypass access rules:
> cron jobs reading all records, creating logs the user can't normally write.
> Never use it to hide a missing access right.

## Useful env patterns

```python
# Check if a module is installed
'sale' in self.env['ir.module.module']._installed()

# Get a parameter
url = self.env['ir.config_parameter'].sudo().get_param('web.base.url')

# Check user groups
self.env.user.has_group('base.group_system')   # is admin?
self.env.user.has_group('crm_custom.group_crm_manager')
```
