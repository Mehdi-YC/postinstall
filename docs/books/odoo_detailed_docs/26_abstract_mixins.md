# 26 — AbstractModel & useful mixins

## AbstractModel — reusable logic without a table

An `AbstractModel` has no DB table. It exists only to be inherited.
Use it to share fields or methods across unrelated models.

```python
class ApiImportMixin(models.AbstractModel):
    """Reusable mixin for models that import from external APIs."""
    _name        = 'crm.api.import.mixin'
    _description = 'API Import Mixin'

    api_import_id   = fields.Char(string='External ID', readonly=True, copy=False)
    api_import_date = fields.Datetime(string='Last Import', readonly=True)

    def _fetch_from_external(self, url, params=None):
        """Shared HTTP fetch logic with auth."""
        import requests
        api_key = self.env['ir.config_parameter'].sudo().get_param(
            'crm_custom.api_key'
        )
        response = requests.get(
            url, params=params,
            headers={'Authorization': f'Bearer {api_key}'},
            timeout=10,
        )
        response.raise_for_status()
        return response.json()

    def _mark_imported(self, external_id):
        self.write({
            'api_import_id':   external_id,
            'api_import_date': fields.Datetime.now(),
        })
```

```python
# Use it in any model
class CrmLead(models.Model):
    _name    = 'crm.lead'
    _inherit = ['crm.api.import.mixin', 'mail.thread', 'mail.activity.mixin']
    # Gets api_import_id, api_import_date, _fetch_from_external(), _mark_imported()
```

## Built-in mixins

| Mixin | What it adds |
|---|---|
| `mail.thread` | Chatter, followers, message log, `tracking=True` on fields |
| `mail.activity.mixin` | Activities (schedule call/email/meeting), `activity_ids` |
| `portal.mixin` | Portal access URL, share token for external users |
| `rating.mixin` | Customer rating/satisfaction on records |
| `avatar.mixin` | Avatar image field + fallback generation |
| `image.mixin` | Standardized image/image_medium/image_small fields |
| `sequence.mixin` | `sequence_number` field + drag-to-reorder |

## portal.mixin — shareable links

```python
class CrmLead(models.Model):
    _name    = 'crm.lead'
    _inherit = ['mail.thread', 'portal.mixin']

    def _compute_access_url(self):
        super()._compute_access_url()
        for lead in self:
            lead.access_url = f'/crm/lead/{lead.id}'

# Now lead.get_portal_url() returns a signed URL for external sharing
```

## image.mixin — standard image fields

```python
class CrmLead(models.Model):
    _name    = 'crm.lead'
    _inherit = ['image.mixin']
    # Adds: image_1920, image_1024, image_512, image_256, image_128
    # Automatically resizes on write

# In view
<field name="image_128" widget="image" class="oe_avatar"/>
```
