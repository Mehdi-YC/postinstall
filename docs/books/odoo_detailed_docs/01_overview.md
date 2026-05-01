# 01 — Overview & vocabulary

## What you'll build

The `crm_custom` module covers every component you'll find in a real-world Odoo module:

- **Model** — fields, states, computed fields, constraints, onchange, CRUD overrides
- **Tabbed form view** — 3 tabs: calls log, general data, interests; status bar + chatter
- **Wizard + API import** — select a category → call mock API → preview → import leads in bulk
- **Kanban + list + filters** — grouped by stage, search filters, group-by, domains
- **Activities** — schedule calls, emails, meetings on leads
- **Server action** — bulk action in list view to mark leads as qualified

---

## Key vocabulary

| Term | What it is |
|---|---|
| `model` | Python class → database table. `_name = 'crm.lead'` becomes table `crm_lead`. |
| `view` | XML that describes how the model looks (form, list, kanban, search). |
| `action` | What happens when you click a menu. Opens a view. |
| `wizard` | Temporary model (`TransientModel`) for multi-step dialogs. |
| `domain` | Filter expression in list/tuple syntax: `[('state', '=', 'new')]` |
| `context` | Dict passed around for defaults, flags: `{'default_stage': 'new'}` |
| `depends` | Decorator telling Odoo which fields trigger a computed field recompute. |
| `onchange` | Decorator for UI-only reactions when a user edits a field. |
| `recordset` | A list-like object of zero or more records. Everything in the ORM is a recordset. |
| `env` | The environment: current user, company, context, and access to all models. |
| `sudo()` | Run an ORM call bypassing access rights (as superuser). |
| `ir.*` | Odoo's meta-models: views, menus, actions, sequences, crons, etc. |
| `external ID` | Stable string reference to a record: `module.xml_id` (e.g. `base.group_user`). |

---

## The mental model

```
__manifest__.py          ← module identity + file load order
    ↓
models/*.py              ← Python classes = DB tables
    ↓
security/access.csv      ← who can CRUD which model
    ↓
data/*.xml               ← default records loaded at install
    ↓
views/*.xml              ← how models look + menus + actions
```

Every file in `data` and `views` is loaded **in the order listed in `__manifest__.py`**.
Security must come before views, or Odoo will raise an access error during install.
