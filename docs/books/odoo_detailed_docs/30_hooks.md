# 30 — post_init_hook & migration hooks

Run Python code at install/upgrade time. Essential for data migrations.

## Declare in __manifest__.py

```python
{
    'name': 'CRM Custom',
    'post_init_hook':  'post_init_hook',
    'pre_upgrade_hook':  'pre_upgrade_hook',
    'post_upgrade_hook': 'post_upgrade_hook',
}
```

## Define in module root __init__.py

```python
# crm_custom/__init__.py
from . import models
from . import wizard


def post_init_hook(env):
    """Runs ONCE after the module is first installed.
    Good for: backfilling new fields on existing data.
    """
    leads = env['crm.lead'].search([('source', '=', False)])
    leads.write({'source': 'manual'})


def pre_upgrade_hook(env):
    """Runs BEFORE the upgrade starts.
    The DB still has the OLD schema.
    Good for: saving data you'll need to migrate before the table changes.
    """
    # Example: save old field values before a column is dropped
    env.cr.execute("""
        UPDATE crm_lead
        SET notes = COALESCE(notes, '') || old_field
        WHERE old_field IS NOT NULL
    """)


def post_upgrade_hook(env):
    """Runs AFTER the upgrade completes.
    The new schema is fully live.
    Good for: backfilling new columns, recomputing stored fields.
    """
    # Backfill a new field added in this version
    env['crm.lead'].search([('call_count', '=', 0)])._compute_call_count()
```

## Odoo 17+ vs older versions

```python
# Odoo 17+: hooks receive 'env' directly
def post_init_hook(env):
    ...

# Odoo 14/15/16: hooks received (cr, registry)
def post_init_hook(cr, registry):
    env = api.Environment(cr, SUPERUSER_ID, {})
    ...
```

## When to use each hook

| Hook | When it runs | Use for |
|---|---|---|
| `post_init_hook` | After first install only | Backfill new module data |
| `pre_upgrade_hook` | Before upgrade (old schema) | Save data before migration |
| `post_upgrade_hook` | After upgrade (new schema) | Backfill new columns, recompute |
| `uninstall_hook` | When module is uninstalled | Cleanup, restore defaults |

## uninstall_hook

```python
# __manifest__.py
'uninstall_hook': 'uninstall_hook',

# __init__.py
def uninstall_hook(env):
    """Cleanup when module is uninstalled."""
    # Remove custom data that wouldn't be cleaned up automatically
    env['ir.config_parameter'].sudo().search([
        ('key', 'like', 'crm_custom.%')
    ]).unlink()
```
