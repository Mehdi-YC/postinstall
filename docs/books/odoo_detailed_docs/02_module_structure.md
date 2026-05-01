# 02 — Module structure

Odoo expects a specific file layout. Every folder and file has a purpose.

## Directory tree

```
crm_custom/
├── models/
│   ├── __init__.py           # imports all model files
│   ├── crm_lead.py           # main Lead model
│   └── crm_lead_category.py  # Category model
├── wizard/
│   ├── __init__.py
│   └── import_leads_wizard.py
├── views/
│   ├── crm_lead_views.xml    # form, list, kanban, search
│   ├── crm_lead_wizard.xml   # wizard form view
│   └── menus.xml             # menu items + actions
├── security/
│   ├── ir.model.access.csv   # CRUD permissions
│   └── crm_security.xml      # groups (optional)
├── data/
│   └── crm_stage_data.xml    # default stages/categories
├── __init__.py               # imports models/ and wizard/
└── __manifest__.py
```

## `__init__.py` pattern

Each folder's `__init__.py` imports the Python files inside it.

```python
# crm_custom/__init__.py
from . import models
from . import wizard

# crm_custom/models/__init__.py
from . import crm_lead
from . import crm_lead_category

# crm_custom/wizard/__init__.py
from . import import_leads_wizard
```

This is standard Python package importing. Odoo discovers your models because it
imports the module's root `__init__.py` on startup.

## Why this layout?

- `models/` — one file per logical model. Don't put all models in one file.
- `wizard/` — separate folder makes it clear these are `TransientModel` classes.
- `views/` — one XML file per concern. Mix menus + actions in `menus.xml`.
- `security/` — loaded first so all views can reference access rights correctly.
- `data/` — records that need to exist when the module is installed (stages, sequences, crons).
