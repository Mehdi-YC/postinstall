# 19 — Exceptions & return actions

## Exception types

```python
from odoo.exceptions import (
    UserError,        # user did something wrong
    ValidationError,  # constraint failed
    AccessError,      # no permission (usually raised by ORM automatically)
    MissingError,     # record was deleted mid-session
    RedirectWarning,  # error + button to go fix it
)
```

## UserError — most common

```python
def unlink(self):
    if self.filtered(lambda r: r.state == 'won'):
        raise UserError('Cannot delete Won leads.')
    return super().unlink()
```

## ValidationError — for @api.constrains

```python
@api.constrains('expected_revenue')
def _check_revenue(self):
    for rec in self:
        if rec.expected_revenue < 0:
            raise ValidationError('Revenue cannot be negative.')
```

## RedirectWarning — error with a fix button

```python
from odoo.exceptions import RedirectWarning

def action_import(self):
    api_url = self.env['ir.config_parameter'].sudo().get_param('crm_custom.api_url')
    if not api_url:
        action = self.env.ref('base.action_res_company_form')
        raise RedirectWarning(
            'API URL is not configured.',
            action.id,
            'Go to Settings',
        )
```

## Never use bare exceptions

```python
# WRONG: Odoo won't display these nicely
raise Exception('Something went wrong')
raise ValueError('Invalid value')

# CORRECT: always use Odoo exception types
raise UserError('Something went wrong')
raise ValidationError('Invalid value')
```

## Returning actions from methods

```python
# Open a specific record (form view)
def action_open_form(self):
    self.ensure_one()
    return {
        'type':      'ir.actions.act_window',
        'res_model': 'crm.lead',
        'res_id':    self.id,
        'view_mode': 'form',
        'target':    'current',   # 'new' = popup, 'fullscreen' = full
    }

# Open filtered list
def action_view_won(self):
    return {
        'type':      'ir.actions.act_window',
        'name':      'Won Leads',
        'res_model': 'crm.lead',
        'view_mode': 'list,form',
        'domain':    [('state', '=', 'won')],
        'context':   {'default_state': 'won'},
    }

# Open URL in new tab
return {'type': 'ir.actions.act_url', 'url': url, 'target': 'new'}

# Reload the current view
return {'type': 'ir.actions.client', 'tag': 'reload'}

# Close a wizard dialog
return {'type': 'ir.actions.act_window_close'}
```
