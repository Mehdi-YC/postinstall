# 18 — API decorators

Decorators tell the ORM how to call your method.

## Quick reference

| Decorator | `self` is | Use for |
|---|---|---|
| `@api.model` | empty recordset (class-level) | create helpers, class-level logic |
| `@api.model_create_multi` | — | overriding `create()` |
| `@api.depends(...)` | recordset | computed fields |
| `@api.onchange(...)` | single record (UI only) | UI reactions |
| `@api.constrains(...)` | recordset | validation on save |
| (none) | recordset | regular methods, button handlers |

## @api.model

```python
@api.model
def get_default_category(self):
    # Called on the class, not a specific record
    # self.env works, self.id does not make sense
    return self.env.ref('crm_custom.cat_tech')

@api.model
def _get_api_config(self):
    return self.env['ir.config_parameter'].sudo().get_param('crm_custom.api_url')
```

## @api.model_create_multi

Always use this when overriding `create()`. It handles both single and batch creates.

```python
@api.model_create_multi
def create(self, vals_list):
    # vals_list is always a list of dicts, even for single create
    for vals in vals_list:
        if not vals.get('ref'):
            vals['ref'] = self.env['ir.sequence'].next_by_code('crm.lead')
    return super().create(vals_list)
```

## @api.depends

```python
# Single field
@api.depends('call_ids')
def _compute_call_count(self):
    for rec in self:
        rec.call_count = len(rec.call_ids)

# Nested field (dot notation) — recomputes when partner's country changes
@api.depends('partner_id.country_id.name')
def _compute_country(self):
    for rec in self:
        rec.country_name = rec.partner_id.country_id.name or ''

# Multiple fields
@api.depends('call_ids', 'call_ids.answered', 'call_ids.duration')
def _compute_call_stats(self):
    for rec in self:
        answered = rec.call_ids.filtered('answered')
        rec.answered_count = len(answered)
        rec.total_duration = sum(answered.mapped('duration'))
```

## @api.onchange

```python
@api.onchange('partner_id')
def _onchange_partner(self):
    # Only runs in UI, not on create/write
    # self.id may be False (unsaved record)
    if self.partner_id:
        self.phone = self.partner_id.phone
        self.email = self.partner_id.email

    # Return a warning (optional)
    if self.partner_id and self.partner_id.crm_score < 10:
        return {'warning': {
            'title': 'Low score customer',
            'message': 'This partner has a low CRM score.',
        }}
```

## @api.constrains

```python
@api.constrains('expected_revenue', 'state')
def _check_won_revenue(self):
    for rec in self:
        if rec.state == 'won' and rec.expected_revenue <= 0:
            raise ValidationError(
                'A Won lead must have a positive expected revenue.'
            )
```

## Common mistake: onchange vs constrains

```python
# This ONLY runs in the UI (onchange):
@api.onchange('start_date', 'end_date')
def _onchange_dates(self):
    if self.end_date and self.start_date and self.end_date < self.start_date:
        self.end_date = self.start_date

# This runs on SAVE (constrains):
@api.constrains('start_date', 'end_date')
def _check_dates(self):
    for rec in self:
        if rec.end_date and rec.start_date and rec.end_date < rec.start_date:
            raise ValidationError('End date cannot be before start date.')

# Best practice: use BOTH for a good UX
```
