# 31 — HTTP controllers

Expose custom HTTP endpoints: webhooks, portal pages, API routes.

## Basic structure

```python
from odoo import http
from odoo.http import request
import json

class CrmController(http.Controller):
    pass
```

Place in a file like `controllers/main.py` and import in `__init__.py`.

## JSON endpoint (webhook / REST API)

```python
@http.route(
    '/api/crm/leads',
    type='json',
    auth='api_key',    # or 'user', 'public', 'none'
    methods=['POST'],
    csrf=False,
)
def create_lead_api(self, **kwargs):
    vals = request.jsonrequest   # parsed JSON body as dict
    lead = request.env['crm.lead'].sudo().create({
        'name':   vals['name'],
        'phone':  vals.get('phone'),
        'email':  vals.get('email'),
        'source': 'api',
    })
    return {'id': lead.id, 'ref': lead.ref, 'status': 'created'}
```

## HTML page endpoint (portal)

```python
@http.route(
    '/crm/lead/<int:lead_id>',
    type='http',
    auth='public',
    website=True,
)
def lead_portal(self, lead_id, **kw):
    lead = request.env['crm.lead'].sudo().browse(lead_id)
    if not lead.exists():
        return request.not_found()
    return request.render('crm_custom.lead_portal_template', {
        'lead': lead,
    })
```

## auth types

| `auth=` | Who can call it |
|---|---|
| `'user'` | Logged-in Odoo users (session cookie) |
| `'public'` | Anyone, including anonymous |
| `'api_key'` | Authenticated via API key header |
| `'none'` | No auth — raw access (use carefully) |

## Accessing request data

```python
# JSON body (type='json')
data = request.jsonrequest      # dict

# Form/POST data (type='http')
name  = request.httprequest.form.get('name')

# Query string params (?state=new)
state = request.httprequest.args.get('state')

# Headers
token = request.httprequest.headers.get('X-Api-Token')

# Current user
user = request.env.user
uid  = request.uid

# Session
request.session['my_key'] = 'value'
```

## Returning responses

```python
# JSON (for type='json' routes — just return a dict)
return {'success': True, 'id': lead.id}

# HTTP response
from odoo.http import Response
return Response('OK', status=200, content_type='text/plain')

# Redirect
return request.redirect('/crm/leads')

# 404
return request.not_found()

# Render a QWeb template
return request.render('crm_custom.my_template', {'lead': lead})
```

## controller folder structure

```
crm_custom/
├── controllers/
│   ├── __init__.py      # from . import main
│   └── main.py          # http.Controller subclass
├── __init__.py          # add: from . import controllers
```
