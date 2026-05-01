# 03 — __manifest__.py

The module's identity card. Odoo reads this to know what the module is and what files to load.

## Full example

```python
{
    'name': 'CRM Custom',
    'version': '19.0.1.0.0',        # major.minor.patch (major = Odoo version)
    'category': 'CRM',
    'summary': 'Custom CRM with API import',
    'author': 'You',
    'depends': ['base', 'mail'],    # mail = chatter + activities
    'data': [
        # ORDER MATTERS: security first, then data, then views
        'security/ir.model.access.csv',
        'security/crm_security.xml',
        'data/crm_stage_data.xml',
        'views/crm_lead_views.xml',
        'views/crm_lead_wizard.xml',
        'views/menus.xml',
    ],
    'post_init_hook': 'post_init_hook',  # optional: runs after install
    'installable': True,
    'application': True,
}
```

## Key fields

| Key | Purpose |
|---|---|
| `depends` | Modules that must be installed before this one. Odoo auto-loads their models/views. |
| `data` | XML/CSV files loaded at install/upgrade time, in order. |
| `application` | `True` = shows up in the Apps grid as a standalone app. |
| `installable` | `False` = Odoo ignores this module. |
| `post_init_hook` | Python function name called once after first install. |
| `pre_upgrade_hook` | Called before upgrade (old schema still live). |
| `post_upgrade_hook` | Called after upgrade (new schema live). |

## `depends` — what to include

- `base` — always required (res.users, res.partner, ir.* models)
- `mail` — chatter, followers, activities, `tracking=True` on fields
- `product` — if you need products/pricelists
- `account` — if you need invoicing

> **Rule:** only add what you actually use. Every dependency slows down install
> and creates upgrade coupling.
