# 32 — JSON-RPC (calling Odoo from outside)

Odoo has a built-in JSON-RPC API. All model methods are callable without extra setup.

## Authenticate + call (Python)

```python
import requests

BASE = 'https://yourinstance.odoo.com'
DB   = 'mydb'

session = requests.Session()

# 1. Authenticate — get session cookie
auth = session.post(f'{BASE}/web/session/authenticate', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {'db': DB, 'login': 'admin', 'password': 'admin'}
})
uid = auth.json()['result']['uid']
print(f'Logged in as uid={uid}')

# 2. Call any model method
resp = session.post(f'{BASE}/web/dataset/call_kw', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {
        'model':  'crm.lead',
        'method': 'search_read',
        'args':   [[('state', '=', 'new')]],
        'kwargs': {'fields': ['name', 'phone', 'state'], 'limit': 10}
    }
})
leads = resp.json()['result']

# 3. Create a record
resp = session.post(f'{BASE}/web/dataset/call_kw', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {
        'model':  'crm.lead',
        'method': 'create',
        'args':   [{'name': 'New Lead', 'phone': '0550001234'}],
        'kwargs': {}
    }
})
new_id = resp.json()['result']
```

## Key endpoints

| Endpoint | Purpose |
|---|---|
| `/web/session/authenticate` | Login, get session cookie |
| `/web/dataset/call_kw` | Call any ORM method |
| `/web/dataset/call_button` | Call a button method (returns action) |
| `/api/{model}` | Odoo 17+ REST API (GET/POST/PATCH/DELETE) |

## Odoo 17+ REST API (preferred for Odoo 19)

```python
import requests

BASE    = 'https://yourinstance.odoo.com'
API_KEY = 'your-api-key'   # generated in user settings

headers = {
    'Content-Type':  'application/json',
    'Authorization': f'Bearer {API_KEY}',
}

# GET — search records
resp = requests.get(
    f'{BASE}/api/crm.lead',
    params={'domain': "[('state','=','new')]", 'fields': 'name,phone,state'},
    headers=headers,
)
leads = resp.json()

# POST — create
resp = requests.post(
    f'{BASE}/api/crm.lead',
    json={'name': 'API Lead', 'phone': '0550001234'},
    headers=headers,
)
new_id = resp.json()['id']

# PATCH — update
resp = requests.patch(
    f'{BASE}/api/crm.lead/{new_id}',
    json={'state': 'qualified'},
    headers=headers,
)

# DELETE
resp = requests.delete(f'{BASE}/api/crm.lead/{new_id}', headers=headers)
```

## Error handling

```python
resp = session.post(f'{BASE}/web/dataset/call_kw', json={...})
data = resp.json()

if 'error' in data:
    err = data['error']
    print(f"Error {err['code']}: {err['data']['message']}")
else:
    result = data['result']
```

## Generate an API key (Odoo 17+)

Settings → Users → your user → API Keys tab → New Key.
Store it securely — it's shown only once.
