# Odoo 19 Technical Guide — CRM Module

A complete backend developer reference for Odoo 19.
Assumes you know Python and OOP. Does not cover Owl/frontend.

---

## Table of contents

### Setup
- [01 — Overview & vocabulary](01_overview.md)
- [02 — Module structure](02_module_structure.md)
- [03 — __manifest__](03_manifest.md)

### Core
- [04 — Model fields & methods](04_model.md)
- [05 — Security](05_security.md)
- [06 — Views & tabs](06_views.md)
- [07 — Status bar](07_statusbar.md)
- [08 — Kanban view](08_kanban.md)
- [09 — Filters & domains](09_filters_domains.md)
- [10 — Activities](10_activities.md)

### Actions
- [11 — Menus & actions](11_menus_actions.md)
- [12 — Wizard](12_wizard.md)
- [13 — Server action](13_server_action.md)

### ORM Deep Dive
- [14 — env & self](14_env_self.md)
- [15 — Recordsets](15_recordsets.md)
- [16 — search / read / browse](16_search_read.md)
- [17 — Model inheritance](17_inheritance.md)
- [18 — API decorators](18_api_decorators.md)

### Business Logic
- [19 — Exceptions & return actions](19_exceptions.md)
- [20 — Context & sudo](20_context_sudo.md)
- [21 — Cron jobs](21_cron.md)
- [22 — Sequences](22_sequences.md)
- [23 — Config parameters](23_config_params.md)

### Advanced ORM
- [24 — Raw SQL](24_raw_sql.md)
- [25 — Performance & N+1](25_performance.md)
- [26 — AbstractModel & mixins](26_abstract_mixins.md)

### Module Patterns
- [27 — Extending existing modules](27_extending.md)
- [28 — Data XML tricks](28_data_xml.md)
- [29 — ir.* models](29_ir_models.md)
- [30 — post_init_hook](30_hooks.md)

### HTTP & RPC
- [31 — HTTP controllers](31_controllers.md)
- [32 — JSON-RPC](32_jsonrpc.md)

### Testing
- [33 — Unit tests](33_testing.md)

---

## The crm_custom module

The companion code implements a full CRM module with:
- Lead model with state machine, computed fields, constraints
- Call log (One2many), interests (Many2many)
- Wizard that fetches from a mock API and imports leads
- Kanban + list + form with 3 tabs + search view
- Activities, chatter, status bar
- Server action (bulk qualify from list view)
- Cron job, sequences, security rules
