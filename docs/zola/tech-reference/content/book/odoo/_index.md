+++
title = "Odoo 19 Module Development Reference"
description = "File structure, conventions, commonly missed details, OWL introduction, and dev operations"
[extra]
date = 2024-01-15
+++

## Module File Structure

Every Odoo module follows a standard directory layout. Not all folders are required. Only include what your module needs, but what you do include must follow this structure exactly.
```py
    my_module/
    __manifest__.py          # Required. Without this, the folder is ignored.
    __init__.py              # Required. Makes this a Python package. must exist.
    
    # Models
    models/
        __init__.py          # Must import every model file in this directory.
        my_model.py
        my_model_line.py
    
    # Views
    views/
        my_model_views.xml   # Form, list, search, kanban views + actions + menus
        templates.xml        # QWeb templates (email, reports, snippets)
    
    # Security
    security/
        ir.model.access.csv  # Who can read/write/create/delete on each model
        ir.rule.csv          # Row-level rules (e.g. "only see own records")
        groups.xml           # Custom user groups / roles
    
    # Data (initial records, mail templates, sequences)
    data/
        mail_template.xml
        sequence.xml
    
    # Static assets (JS, CSS, images)
    static/
        description/
            icon.png         # Module icon. 120x120 transparent PNG.
        src/
            js/
                my_component.js   # OWL components
            xml/
                my_templates.xml  # QWeb templates for OWL
            css/
                style.css
    
    # Tests
    tests/
        __init__.py
        test_my_model.py
    
    # Wizards (transient multi-step flows)
    wizard/
        __init__.py
        my_wizard.py
        my_wizard_views.xml
    
    # Reports
    report/
        __init__.py
        my_report.py
        my_report_templates.xml
```
> **Only three things are truly mandatory:** `__manifest__.py`, root `__init__.py`, and `static/description/icon.png`. Everything else is optional.

## __manifest__.py

The manifest tells Odoo everything about your module before loading it.
```python,linenos

    {
        # Internal technical name. Must be unique. Convention: snake_case.
        'name': 'My Module',

        # Version: Odoo_major.series.iteration.patch.build
        'version': '19.0.1.0.0',

        'summary': 'Does X and Y',

        'description': """
            Detailed explanation of what this module does.
        """,

        'author': 'Your Name',
        'website': 'https://example.com',
        'license': 'LGPL-3',
        'category': 'Sales',

        # Modules that must be installed first.
        # If your code references sale.order, 'sale' MUST be here.
        'depends': ['base', 'sale', 'mail'],

        # Files loaded on install/update. ORDER MATTERS.
        # Security first, then data, then views, then wizards/reports.
        'data': [
            'security/groups.xml',
            'security/ir.model.access.csv',
            'security/ir.rule.csv',
            'data/mail_template.xml',
            'views/my_model_views.xml',
            'views/templates.xml',
            'wizard/my_wizard_views.xml',
            'report/my_report_templates.xml',
        ],

        # Demo data: only loaded in databases with demo data enabled.
        'demo': ['data/demo_data.xml'],

        'assets': {
            'web.assets_backend': [
                'my_module/static/src/js/my_component.js',
                'my_module/static/src/css/style.css',
            ],
            'web.assets_qweb': [
                'my_module/static/src/xml/my_templates.xml',
            ],
        },

        'installable': True,
        'auto_install': False,
        'application': True,

        'external_dependencies': {
            'python': ['requests'],
        },

        # Hooks: format is 'file:function' (function at module root level)
        'post_init_hook': '_post_init_hook',
        'uninstall_hook': '_uninstall_hook',
    }
```
> **Data file order is critical.** Security files must come first. If a view references a model and its access CSV hasn't loaded yet, installation fails with an access error.

## Models (models/)
```py,linenos
    # models/__init__.py
    # MUST import every model file. Miss one = Odoo doesn't discover it.
    from . import my_model
    from . import my_model_line

    # models/my_model.py
    from odoo import models, fields, api, _
    from odoo.exceptions import ValidationError, UserError


    class MyModel(models.Model):

        _name = 'my.module'
        _description = 'My Module Record'
        _order = 'name asc, create_date desc'
        _rec_name = 'name'

        # ---- Field Types ----

        name = fields.Char(string='Name', required=True, translate=True)
        description = fields.Text(string='Description')

        state = fields.Selection([
            ('draft', 'Draft'),
            ('confirmed', 'Confirmed'),
            ('done', 'Done'),
            ('cancelled', 'Cancelled'),
        ], string='Status', default='draft', tracking=True)

        priority = fields.Integer(string='Priority', default=0)

        # Float: ALWAYS specify digits to avoid rounding issues.
        amount = fields.Float(string='Amount', digits=(10, 2))

        # Monetary: use for currency amounts, NOT Float.
        total = fields.Monetary(string='Total', currency_field='currency_id')
        currency_id = fields.Many2one('res.currency', string='Currency')

        # Many2one: foreign key. index=True on searched/grouped fields.
        partner_id = fields.Many2one(
            'res.partner', string='Customer',
            ondelete='set null', index=True,
        )

        # One2many: inverse of Many2one. Virtual (not a real DB column).
        line_ids = fields.One2many('my.module.line', 'order_id', string='Lines')

        # Many2many: stored via a relational table.
        tag_ids = fields.Many2many('my.module.tag', string='Tags')

        date = fields.Date(string='Date')
        date_done = fields.Datetime(string='Done At')

        # 'active' is special: False = archived, hidden from searches by default.
        active = fields.Boolean(string='Active', default=True)

        # attachment=True: stores in ir.attachment instead of base64 in DB.
        document = fields.Binary(string='Document', attachment=True)

        notes = fields.Html(string='Notes')

        # ---- Computed Fields ----
        # store=True: required for search, sort, group by.
        # depends: fields that trigger recomputation when changed.
        display_name = fields.Char(compute='_compute_display_name', store=True)

        def _compute_display_name(self):
            for rec in self:
                rec.display_name = rec.name or '(unnamed)'

        # ---- Related Fields ----
        # Shortcut to a field on a related model. Same as computed+stored.
        partner_email = fields.Email(
            related='partner_id.email', string='Customer Email',
            readonly=False, store=True,
        )

        # ---- Constraints ----
        _sql_constraints = [
            ('name_uniq', 'UNIQUE(name)', 'Name must be unique.'),
        ]

        @api.constrains('amount', 'state')
        def _check_amount(self):
            for rec in self:
                if rec.state == 'confirmed' and rec.amount <= 0:
                    raise ValidationError(_('Amount must be positive when confirmed.'))

        # ---- Methods ----
        # self is ALWAYS a recordset. Use 'for rec in self' for per-record logic.

        def action_confirm(self):
            for rec in self:
                if rec.state != 'draft':
                    raise UserError(_('Only draft records can be confirmed.'))
            self.write({'state': 'confirmed'})

        # ---- Overrides: always call super() ----
        def write(self, vals):
            result = super().write(vals)
            return result

        def unlink(self):
            for rec in self:
                if rec.state == 'confirmed':
                    raise UserError(_('Cannot delete confirmed records.'))
            return super().unlink()


    # ---- Transient Model (wizard) ----
    class MyWizard(models.TransientModel):
        _name = 'my.module.wizard'
        _description = 'My Module Wizard'

        date_from = fields.Date(required=True)
        date_to = fields.Date(required=True)

        def action_execute(self):
            records = self.env['my.module'].search([
                ('date', '>=', self.date_from),
                ('date', '<=', self.date_to),
            ])
            return {'type': 'ir.actions.act_window_close'}
```
> **Access models via `self.env['model.name']`**, never by importing the class. This ensures correct database, user context, and cache.

### Inheritance: Extending Existing Models
```py
    # Inherit WITHOUT _name: extends the existing model in place.
    # This is what you want 95% of the time.
    from odoo import models, fields

    class SaleOrder(models.Model):
        _inherit = 'sale.order'

        my_custom_field = fields.Char(string='Custom Field')

        def action_confirm(self):
            result = super().action_confirm()
            return result
```
> **If you set both `_name` and `_inherit`**, it creates a NEW model that copies the parent (prototype inheritance). This is rarely what you want.

### Useful Mixins

| Mixin | Adds |
|-------|------|
| `mail.thread` | Chatter: messages, notes, followers |
| `mail.activity.mixin` | Scheduled activities with deadlines |
| `image.mixin` | image_1920 field with auto-thumbnail |
| `portal.mixin` | Share records with portal users via link |

    # Usage:
    _name = 'my.module'
    _inherit = ['mail.thread', 'mail.activity.mixin']

## Views (views/)

| Type | Purpose | Key Detail |
|------|---------|-------------|
| `form` | Create/edit one record | Header, sheet, notebook, chatter |
| `list` | Table of records (was "tree") | `editable="bottom"` for inline editing |
| `kanban` | Card board | Needs inner QWeb template |
| `search` | Filters, group-by, search fields | Separate from other views |
| `graph` | Bar/line/pie charts | `type="bar|line|pie"` |
| `pivot` | Pivot table analysis | OLAP-style data exploration |
| `calendar` | Calendar view | Requires `date_start` field |
```xml,linenos
    # views/my_model_views.xml
    <?xml version="1.0" encoding="utf-8"?>
    <odoo>

    <!-- FORM VIEW -->
    <record id="my_module_form" model="ir.ui.view">
        <field name="name">my.module.form</field>
        <field name="model">my.module</field>
        <field name="arch" type="xml">
            <form>
                <header>
                    <!-- type="object" calls a Python method -->
                    <button name="action_confirm" string="Confirm"
                            type="object" class="btn-primary"
                            invisible="state != 'draft'" />
                    <button name="action_done" string="Done"
                            type="object"
                            invisible="state != 'confirmed'" />
                    <button name="action_cancel" string="Cancel"
                            type="object"
                            invisible="state in ('done', 'cancelled')" />
                    <field name="state" widget="statusbar"
                           statusbar_visible="draft,confirmed" clickable="0" />
                </header>

                <sheet>
                    <div class="oe_title">
                        <h1><field name="name" placeholder="Enter name..." /></h1>
                    </div>
                    <!-- GROUP: 4-col layout (label,field,label,field). col="2" for single col -->
                    <group>
                        <group string="Information">
                            <field name="partner_id" />
                            <field name="date" />
                        </group>
                        <group string="Amounts">
                            <field name="currency_id" />
                            <field name="amount" />
                            <field name="total" />
                        </group>
                    </group>
                    <notebook>
                        <page string="Lines">
                            <field name="line_ids">
                                <list editable="bottom">
                                    <field name="product_id" />
                                    <field name="quantity" />
                                    <field name="price_unit" />
                                </list>
                            </field>
                        </page>
                        <page string="Notes">
                            <field name="description" />
                            <field name="notes" />
                        </page>
                    </notebook>
                </sheet>

                <!-- CHATTER: outside sheet. Requires mail.thread. -->
                <div class="oe_chatter">
                    <field name="message_follower_ids" />
                    <field name="message_ids" />
                </div>
            </form>
        </field>
    </record>

    <!-- LIST VIEW -->
    <record id="my_module_list" model="ir.ui.view">
        <field name="name">my.module.list</field>
        <field name="model">my.module</field>
        <field name="arch" type="xml">
            <!-- decoration-*: danger, success, info, warning, muted -->
            <list string="Records"
                  decoration-danger="state == 'cancelled'"
                  decoration-success="state == 'done'">
                <field name="name" />
                <field name="partner_id" />
                <field name="amount" />
                <field name="state" widget="badge"
                       decoration-success="state == 'done'"
                       decoration-info="state == 'confirmed'"
                       decoration-danger="state == 'cancelled'" />
            </list>
        </field>
    </record>

    <!-- SEARCH VIEW -->
    <record id="my_module_search" model="ir.ui.view">
        <field name="name">my.module.search</field>
        <field name="model">my.module</field>
        <field name="arch" type="xml">
            <search>
                <field name="name" />
                <field name="partner_id" />
                <filter name="draft" string="Drafts"
                        domain="[('state', '=', 'draft')]" />
                <separator />
                <filter name="by_partner" string="Customer"
                        context="{'group_by': 'partner_id'}" />
                <filter name="by_state" string="Status"
                        context="{'group_by': 'state'}" />
            </search>
        </field>
    </record>

    <!-- ACTION -->
    <record id="my_module_action" model="ir.actions.act_window">
        <field name="name">My Records</field>
        <field name="res_model">my.module</field>
        <field name="view_mode">list,form</field>
        <field name="view_id" ref="my_module_list" />
        <field name="search_view_id" ref="my_module_search" />
        <field name="context">{'search_default_draft': 1}</field>
        <field name="help" type="html">
            <p class="o_view_nocontent_smiling_face">Create your first record!</p>
        </field>
    </record>

    <!-- MENUS -->
    <menuitem id="my_module_menu_root" name="My Module" sequence="40" />
    <menuitem id="my_module_menu_records" name="Records"
              parent="my_module_menu_root" action="my_module_action" />

    </odoo>
```
> **Chatter goes OUTSIDE `<sheet>`.** Putting it inside won't render correctly.

## Security (security/)
```csv
    # security/ir.model.access.csv
    # id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
    # group_id empty = public. base.group_user = all internal users.
    access_my_module_manager,my.module.manager,model_my_module,my_module.group_manager,1,1,1,1
    access_my_module_user,my.module.user,model_my_module,my_module.group_user,1,1,1,0
```
```xml,linenos
    # security/groups.xml
    <?xml version="1.0" encoding="utf-8"?>
    <odoo>
        <record id="module_category_my_module" model="ir.module.category">
            <field name="name">My Module</field>
        </record>
        <record id="group_user" model="res.groups">
            <field name="name">My Module User</field>
            <field name="category_id" ref="module_category_my_module" />
            <field name="implied_ids" eval="[(4, ref('base.group_user'))]" />
        </record>
        <record id="group_manager" model="res.groups">
            <field name="name">My Module Manager</field>
            <field name="category_id" ref="module_category_my_module" />
            <field name="implied_ids" eval="[(4, ref('my_module.group_user'))]" />
        </record>
    </odoo>
```
```
    # security/ir.rule.csv (optional)
    # Row-level security
    my_module_rule_user,my.module.user.rule,model_my_module,[('create_uid','=',user.id)],my_module.group_user,1,1,1,0
```
> **Missing ACL = silent access denied.** If a model has no row in `ir.model.access.csv`, even admin gets errors in the UI.

## Data Files (data/)
```xml,linenos
    # data/sequence.xml (example)
    <odoo>
        <!-- noupdate="1": user modifications survive module updates -->
        <data noupdate="1">
            <record id="my_module_sequence" model="ir.sequence">
                <field name="name">My Module Sequence</field>
                <field name="code">my.module</field>
                <field name="prefix">MYM/</field>
                <field name="padding" eval="5" />
            </record>
        </data>
    </odoo>
```
> **`noupdate="1"` matters.** Without it, every module update resets sequences, mail templates, and settings to defaults, erasing user customizations.

## Static Assets (static/)
```python,linenos
    # In __manifest__.py:
    'assets': {
        'web.assets_backend': [
            'my_module/static/src/js/my_component.js',
            'my_module/static/src/css/style.css',
        ],
        'web.assets_qweb': [
            'my_module/static/src/xml/my_templates.xml',
        ],
        'web.assets_frontend': [
            'my_module/static/src/js/website_stuff.js',
        ],
    },
```
> **After editing static files, clear your browser cache** (Ctrl+Shift+R). In development, enable **Developer Mode → Debug Assets** or start Odoo with `--dev=reload`.

## Commonly Missed Things

1. **Forgetting to import in `__init__.py`.** Every model file must be imported in its directory's `__init__.py`, and every sub-directory in the root `__init__.py`.

2. **Missing ACL entries.** No row in `ir.model.access.csv` for a model = "Access Denied" when clicking the menu.

3. **Wrong data file order in manifest.** Security must load first.

4. **Not calling `super()` in overrides.** Silently breaks other modules overriding the same method.

5. **Missing `store=True` on computed fields.** Without it, can't search, sort, or group by that field.

6. **No `index=True` on frequently searched Many2one fields.**

7. **Treating `self` as a single record.** Always use `for rec in self`.

8. **Not using `_()` for translatable strings.**

9. **Chatter inside `<sheet>`.** Must be outside.

10. **No `noupdate="1"` on user-configurable data.**

11. **Forgetting `active` filter.** To include archived: `[('active', 'in', [True, False])]`.

12. **Using Float for money.** Use `fields.Monetary`.

13. **Not clearing cache after asset changes.**

14. **Duplicate XML IDs across modules.** Last one loaded wins silently.

15. **Using `sudo()` carelessly.** Use `with_user()` when possible. Scope `sudo()` narrowly.

## Best Practices

- Prefix all XML IDs: `my_module_form_view`, not `form_view`
- Prefix model names: `my_module.order`, not `order`
- Use built-in automatic fields: `create_date`, `write_date`, `create_uid`, `write_uid`
- Keep models under ~30 fields — split with One2many if larger
- `self.env.context.get('key')` to read context safely
- `with_context()` to pass context without modifying current environment
- `_logger = logging.getLogger(__name__)` instead of `print()`
- Write tests for critical business logic
- Bump version after schema changes: `19.0.1.0.0` → `19.0.1.0.1`
- Use `useService("orm")` over `useService("rpc")` in OWL

## OWL — Odoo Web Library: Introduction

**OWL** is Odoo's internal UI framework since Odoo 16, replacing the old `odoo.Widget` system. Not a general-purpose framework — built specifically for Odoo.

- **Is:** Lightweight component framework with reactive state, QWeb templates, lifecycle hooks. Similar concept to Vue/React but much smaller.
- **Isn't:** Usable outside Odoo. No router, no state library, no CLI.
- **Replaces:** `odoo.Widget` (Odoo 15 and earlier). If you see it, it's old code.
- **Used for:** Custom views, field widgets, client actions, systray items.

| Concept | What It Means |
|---------|----------------|
| `Component` | JS class extending `owl.Component`. Has state, methods, template. |
| `Template` | QWeb XML with `t-if`, `t-foreach`, `t-on-click` etc. |
| `State` | Reactive properties. Change them → template re-renders automatically. |
| `Props` | Data from parent to child (one-way, top-down). |
| `Lifecycle` | `willStart()`, `mounted()`, `willUpdate()`, `updated()`, `willUnmount()` |
| `useService` | Access Odoo services: ORM, RPC, notifications, router. |
| `Registry` | How components are registered for Odoo to use. |

**Reactivity loop:** State changes → Template re-renders → DOM updates. Never touch the DOM directly.

### Where OWL Files Go

    static/src/
        js/
            my_component.js     # Component class (web.assets_backend)
        xml/
            my_templates.xml   # QWeb templates (web.assets_qweb, NOT backend)
        css/
            style.css          # Use a scoping class name

### OWL vs Legacy Widget

| Aspect | Legacy (odoo.Widget) | OWL |
|--------|---------------------|-----|
| Rendering | Manual `this.$el.append()` | Declarative QWeb, auto-re-render |
| State | No reactivity; manual `render()` | Automatic on property change |
| Performance | Slower | Faster (compiled templates) |
| Lifecycle | `init()`, `start()`, `destroy()` | `setup()`, `willStart()`, `mounted()`, `willUnmount()` |
| Files | JS + inline template strings | Separate .js and .xml |
| Era | Odoo 15 and earlier | Odoo 16+ |

> **They can coexist during migration.** Start new components in OWL, migrate old ones incrementally.

## OWL Components
```js,linenos
    # static/src/js/my_component.js
    /** @odoo-module */
    # Required for Odoo's bundler. Without it, imports break.

    import { Component } from "@odoo/owl";
    import { registry } from "@web/core/registry";
    import { useService } from "@web/core/utils/hooks";

    class MyCounter extends Component {

        //Must match template's t-name exactly.
        static template = "my_module.MyCounter";

        //Declare expected props (optional but recommended).
        static props = {
            initialValue: { type: Number, optional: true },
        };

        setup() {
            super.setup();  # MUST be first line

            //State: reactive. Changing this.count triggers re-render.
            this.count = this.props.initialValue || 0;

            //Odoo services (singletons).
            this.orm = useService("orm");
            this.rpc = useService("rpc");
            this.notification = useService("notification");
            this.router = useService("router");
            this.action = useService("action");
        }

        // ---- Lifecycle ----

        async willStart() {
            await super.willStart();
            //Fetch initial data: this.data = await this.orm.searchRead(...)
        }

        mounted() {
            super.mounted();
            //DOM ready. Init third-party libs here.
        }

        willUnmount() {
            super.willUnmount();
            //CRITICAL: clean up timers, listeners, subscriptions.
        }

        # ---- Methods ----

        increment() {
            this.count++;  // No manual update. OWL detects change.
        }

        async saveToServer() {
            try {
                await this.orm.write("my.module", [this.props.recordId], {
                    count: this.count,
                });
                this.notification.add({ type: "success", message: "Saved!" });
            } catch (e) {
                this.notification.add({ type: "danger", message: e.message });
            }
        }
    }

    //Register as client action (openable from menu).
    registry.category("actions").add("my_module.counter_action", MyCounter);
```
### Common Odoo Services

| Service | What | Key Methods |
|---------|------|--------------|
| `orm` | Model access | `searchRead()`, `create()`, `write()`, `unlink()`, `read()` |
| `rpc` | Raw RPC | `rpc('/my/route', params)` |
| `notification` | Toasts | `add({type, message, title})` |
| `router` | Navigation | `navigate('/')`, `currentRoute` |
| `action` | Odoo actions | `doAction(action)` |
| `dialog` | Dialogs | `add(Component, {props})` |
| `user` | Current user | `userId`, `isAdmin`, `hasGroup()` |
| `http` | HTTP requests | `get()`, `post()`, `put()`, `delete()` |

### Lifecycle Order

| Hook | When | Use For |
|------|------|---------|
| `setup()` | Instantiation (before render) | State, useService, bind methods |
| `willStart()` | Before first render (async OK) | Fetch initial data |
| `mounted()` | After first render, DOM ready | DOM measurements, init libs |
| `willUpdate()` | Before re-renders (not first) | Save scroll position |
| `updated()` | After re-renders (not first) | Restore scroll position |
| `willUnmount()` | Before removal | Clean up timers, listeners |

## OWL Templates (QWeb)
```xml,linenos
    # static/src/xml/my_templates.xml
    <?xml version="1.0" encoding="UTF-8"?>
    <templates xml:space="preserve">

    # t-name must match static template in component class
    <t t-name="my_module.MyCounter">
        <div class="my_counter">

            # t-esc: escaped text (XSS safe). Use for all user data.
            <h2><t t-esc="props.title or 'Counter'" /></h2>

            # t-if: conditional. Element removed from DOM if false.
            <div t-if="state.count > 10" class="alert alert-warning">
                Count is getting high!
            </div>

            # t-attf-class: dynamic class with {{interpolation}}
            <span t-attf-class="badge {{state.count > 0 ? 'bg-success' : 'bg-secondary'}}">
                <t t-esc="state.count" />
            </span>

            # t-on-click: event. "->" calls a method.
            <button class="btn btn-primary" t-on-click="increment">+</button>
            <button class="btn btn-secondary" t-on-click="decrement">-</button>

            # t-foreach + t-as + t-key (required for efficient DOM patching)
            <ul>
                <li t-foreach="state.items" t-as="item" t-key="item.id">
                    <t t-esc="item.name" />
                </li>
            </ul>

            # t-model: two-way binding on inputs
            <input t-model="state.searchQuery" placeholder="Search..." />

        </div>
    </t>

    </templates>
```
### Key QWeb Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `t-esc` | Escaped text (safe) | `<t t-esc="name" />` |
| `t-raw` | Unescaped HTML (XSS risk) | `<t t-raw="html" />` |
| `t-if` | Conditional | `<div t-if="x > 0">` |
| `t-elif` / `t-else` | Else chains | `<div t-elif="...">` |
| `t-foreach` + `t-as` | Loop | `<li t-foreach="items" t-as="item">` |
| `t-key` | Unique key for loops (required) | `t-key="item.id"` |
| `t-on-*` | Event binding | `t-on-click="method"` |
| `t-att-*` | HTML attribute | `t-att-href="url"` |
| `t-attf-*` | Attribute with interpolation | `t-attf-class="btn {{type}}"` |
| `t-model` | Two-way binding | `<input t-model="state.name" />` |

> **`t-key` is required in loops.** Use a unique stable ID, not an index.

## OWL Advanced

### Opening an OWL Component from a Menu

    # In component JS:
    registry.category("actions").add("my_module.counter_action", MyCounter);

    # In views XML:
    <record id="my_module_counter_action" model="ir.actions.client">
        <field name="name">My Counter</field>
        <field name="tag">my_module.counter_action</field>
        <field name="context">{'initial_value': 42}</field>
    </record>

    <menuitem id="my_module_menu_counter" name="Counter"
              parent="my_module_menu_root"
              action="my_module_counter_action" />

> **`tag` must match the registry key.** Context dict is spread into props automatically.

### Component Composition (Parent + Child)

    # static/src/js/child_component.js
    /** @odoo-module */
    import { Component } from "@odoo/owl";

    class ChildBadge extends Component {
        static template = "my_module.ChildBadge";
        static props = { label: String, color: { type: String, optional: true } };
    }

    # static/src/xml/child_templates.xml
    <?xml version="1.0" encoding="UTF-8"?>
    <templates xml:space="preserve">
    <t t-name="my_module.ChildBadge">
        <span t-attf-class="badge {{props.color || 'bg-secondary'}}"
              t-esc="props.label" />
    </t>
    </templates>

    # Parent using the child
    /** @odoo-module */
    import { Component } from "@odoo/owl";
    import { ChildBadge } from "./child_component";

    class MyCounter extends Component {
        static template = "my_module.MyCounter";
        static components = { ChildBadge };  # REQUIRED: declare child components
    }

    <!-- In parent template XML: -->
    <ChildBadge label="Active" color="bg-success" />
    <ChildBadge label="Draft" />

### useState for Reactive Objects and Arrays

Plain objects/arrays on `this` are **not** deeply reactive. Use `useState()`:

    import { Component, useState } from "@odoo/owl";

    class MyList extends Component {
        static template = "my_module.MyList";

        setup() {
            super.setup();
            this.state = useState({ items: [], loading: false });
        }

        async loadItems() {
            this.state.loading = true;
            this.state.items = await this.orm.searchRead("my.module", [], ["name"]);
            this.state.loading = false;
        }

        addItem(item) {
            # DON'T: this.state.items.push(item) -- OWL won't detect it.
            # DO: reassign the array.
            this.state.items = [...this.state.items, item];
        }

        removeItem(id) {
            this.state.items = this.state.items.filter(i => i.id !== id);
        }
    }

> **Never mutate in place.** `push()`, `splice()`, and direct property assignment on nested objects won't trigger re-renders. Always reassign.

### Event Bus for Cross-Component Communication

    import { Component, useState, onWillUnmount } from "@odoo/owl";
    import { useService } from "@web/core/utils/hooks";

    # Emitter component:
    class Emitter extends Component {
        setup() {
            super.setup();
            this.bus = useService("bus");
        }
        notifyOthers() {
            this.bus.emit("my_module:updated", { id: 42 });
        }
    }

    # Listener component:
    class Listener extends Component {
        static template = "my_module.Listener";
        setup() {
            super.setup();
            this.bus = useService("bus");
            this.state = useState({ lastId: null });
            this._handler = this._handler.bind(this);
            this.bus.addEventListener("my_module:updated", this._handler);
            # MUST clean up:
            onWillUnmount(() => {
                this.bus.removeEventListener("my_module:updated", this._handler);
            });
        }
        _handler({ detail }) { this.state.lastId = detail.id; }
    }

### Other Registration Types

    # Custom field widget:
    registry.category("fields").add("my_color_picker", MyColorPicker);
    # Use: <field name="color" widget="my_color_picker" />

    # Custom view type:
    registry.category("views").add("my_custom_view", MyCustomView);
    # Use: <field name="view_mode">list,my_custom_view</field>

### Common OWL Mistakes

1. **Forgetting `/** @odoo-module */`** at top of JS files
2. **Templates in wrong bundle.** XML goes in `web.assets_qweb`, JS in `web.assets_backend`
3. **Missing `t-key` in `t-foreach`**
4. **Not cleaning up in `willUnmount()`** — memory leaks
5. **Mutating arrays/objects in place** — reassign instead
6. **Mismatched `t-name` and `static template`** — blank render, no error
7. **`t-raw` with user data** — XSS. Use `t-esc`
8. **Missing `static components = { Child }`** in parent
9. **Not binding `this` in event callbacks** — use `.bind(this)` or arrow functions
10. **`super.setup()` not first line of `setup()`**

## DevOps: CLI, PostgreSQL, Backups

### Starting Odoo
```bash,linenos
    odoo-bin -c odoo.conf --dev=reload
    # -c odoo.conf     : config file (recommended over CLI flags)
    # --dev=reload      : auto-reload Python + assets on file changes
    # --dev=all         : reload + verbose SQL logging + asset debug
```
> **Use a config file.** CLI flags work but get unwieldy. Put database settings, paths, and addons paths in `odoo.conf`. Only use CLI flags for development overrides.
```conf,linenos
    # odoo.conf (minimal)
    [options]
    addons_path = /path/to/odoo/addons,/path/to/custom_addons
    db_host = localhost
    db_port = 5432
    db_user = odoo
    db_password = yourpassword
    http_port = 8069
    logfile = /path/to/odoo.log
    log_level = info
    # log_level: debug_sql shows all queries (slow but useful)
```
### Common CLI Commands

| Command | What It Does |
|---------|---------------|
| `odoo-bin -c odoo.conf` | Start Odoo with config file |
| `odoo-bin -d mydb -i my_module` | Create DB + install module |
| `odoo-bin -d mydb -u my_module` | Update module (apply changes) |
| `odoo-bin -d mydb -u my_module --test-enable` | Update + run tests |
| `odoo-bin -s` | Save current config to `odoo.conf` and exit |
| `odoo-bin --stop-after-init -i base` | Initialize DB then stop (useful for CI) |
| `odoo-bin shell -d mydb` | Interactive Python shell with Odoo env loaded |
| `odoo-bin scaffold my_module /path/to/addons` | Generate a module skeleton |

> **`-i` creates the database if it doesn't exist.** Be careful — if you typo the database name, you get a fresh empty DB instead of updating your existing one. Always use `-u` for updates.

### Odoo Shell
```python,linenos
The shell gives you a Python REPL with `self.env` (the Odoo environment) already set up. Useful for quick debugging without writing tests.

    odoo-bin -c odoo.conf shell -d mydb

    # Inside the shell:
    >>> self.env['sale.order'].search_count([])
    42
    >>> order = self.env['sale.order'].browse(1)
    >>> order.name
    'SO001'
    >>> order.partner_id.name
    'Azure Interior'
    >>> self.env['sale.order'].search([('state', '=', 'draft')], limit=5)
    # Returns a recordset
    >>> exit()
```
### Working with PostgreSQL
```bash,linenos
    # Connect (default Odoo user is usually "odoo")
    psql -U odoo -d mydb -h localhost

    # If you get "peer authentication failed", use:
    sudo -u postgres psql -d mydb
```
#### Useful Queries
```sql,linenos
    --List all tables in the Odoo database
    \dt

    --Describe a model's table structure (dots become underscores)
    \d sale_order

    -- Count records in a model
    SELECT count(*) FROM sale_order;

    -- Find records by state
    SELECT id, name, state FROM sale_order WHERE state = 'draft';

    -- Check ir.model.access.csv coverage (models without ACL)
    SELECT m.model
    FROM ir_model m
    LEFT JOIN ir_model_access a ON a.model_id = m.id
    WHERE a.id IS NULL AND m.state = 'base';

    -- List all installed modules
    SELECT name, state FROM ir_module_module WHERE state = 'installed';

    --Find which module owns a view (by XML ID)
    SELECT * FROM ir_model_data WHERE name = 'my_module_form';

    --Check table size (useful for finding bloated models)
    SELECT relname AS table_name,
           pg_size_pretty(pg_total_relation_size(relid)) AS size
    FROM pg_stat_user_tables
    ORDER BY pg_total_relation_size(relid) DESC
    LIMIT 20;

    --Kill all connections to a database (needed before dropping it)
    SELECT pg_terminate_backend(pid)
    FROM pg_stat_activity WHERE datname = 'mydb' AND pid <> pg_backend_pid();
```
> **Never modify Odoo tables directly via SQL** unless you know exactly what you're doing. The ORM manages caches, computed fields, and triggers that raw SQL bypasses. For data fixes, use the shell or an `_post_init_hook`.

### Database Backup and Restore

#### Method 1: CLI (recommended for automation/scripts)
```bash
    # Backup: dump the database
    pg_dump -U odoo -d mydb -F c -f mydb_backup.dump
    # -F c: custom format (compressed, supports parallel restore)

    # Restore: create a fresh database first
    createdb -U odoo mydb_restored
    pg_restore -U odoo -d mydb_restored -j 4 --no-owner mydb_backup.dump
    # -j 4: use 4 parallel jobs (much faster)
    # --no-owner: ignore original ownership (avoids permission issues)

    # Quick SQL dump (smaller file, slower restore)
    pg_dump -U odoo -d mydb -f mydb_backup.sql
    psql -U odoo -d mydb_restored -f mydb_backup.sql
```
#### Method 2: Odoo UI (Settings → Database)

Go to `http://localhost:8069/web/database/manager`. Backup and restore from the browser. This handles the filestore (attachments) automatically.

> **Filestore is separate from the database.** When using `pg_dump`, you only get the database. The filestore (actual file attachments stored on disk) lives at `~/.local/share/Odoo/filestore/mydb/`. You must back this up separately. The UI backup handles this automatically.
```bash
    # Filestore location (default):
    ~/.local/share/Odoo/filestore/<database_name>/

    # Back it up:
    tar -czf mydb_filestore.tar.gz ~/.local/share/Odoo/filestore/mydb/

    # Restore it:
    tar -xzf mydb_filestore.tar.gz -C ~/.local/share/Odoo/filestore/
    # Make sure the folder name matches the new database name exactly.
```
### Complete Backup Script
```bash
    #!/bin/bash
    # backup_odoo.sh <database_name>
    # Usage: ./backup_odoo.sh mydb

    DB=$1
    DATE=$(date +%Y%m%d_%H%M%S)
    BACKUP_DIR="./backups/$DATE"
    FILESTORE_DIR="$HOME/.local/share/Odoo/filestore"

    mkdir -p $BACKUP_DIR

    echo "Dumping database..."
    pg_dump -U odoo -d $DB -F c -f "$BACKUP_DIR/${DB}.dump"

    echo "Backing up filestore..."
    if [ -d "$FILESTORE_DIR/$DB" ]; then
        cp -r "$FILESTORE_DIR/$DB" "$BACKUP_DIR/filestore"
    fi

    echo "Done: $BACKUP_DIR"
```
### Debug Mode

Activate via the URL: append `?debug=1` (or `?debug=assets` for asset debugging only).

| Feature | Where to Find |
|---------|----------------|
| View field technical names | Hover over any field label |
| Edit view XML directly | Click the bug icon on any view |
| View model records | Settings → Technical → Models |
| View IR records (views, actions, etc.) | Settings → Technical → corresponding menu |
| Run SQL queries | Does NOT exist natively. Use shell or pgAdmin. |
| Asset debug mode | `?debug=assets` — non-minified JS/CSS, no cache |
| Show translated fields | Icon in top-right when multiple languages installed |

> **`?debug=1` is persistent per session.** Once activated, it stays on until you click "Leave Developer Mode" in the user menu or remove `?debug=1` from the URL.

### Logging
```python,linenos
    # In Python code
    import logging

    _logger = logging.getLogger(__name__)

    # Use these instead of print():
    _logger.debug("Detailed info for debugging: %s", value)
    _logger.info("Normal operational info")
    _logger.warning("Something unexpected happened")
    _logger.error("Something went wrong: %s", error.message)
```
```toml,linenos
    # In odoo.conf
    [options]
    # Log levels (from least to most verbose): critical, error, warning, info, debug
    log_level = warn

    # Log only specific modules at debug level (very useful):
    log_handler = odoo.models:DEBUG
    log_handler = odoo.http:DEBUG

    # Log to file instead of stdout:
    logfile = /var/log/odoo/odoo.log

    # Log database queries (very slow, only for debugging):
    # Either use --dev=debug_sql or:
    log_level = debug_sql
```
> **`log_handler` with module-level filtering is the best approach.** Instead of setting the entire log to debug (which floods output), log only the module you're working on: `log_handler = odoo.addons.my_module:DEBUG`.

### Development Workflow Tips

- **Use `--dev=reload`** during development. It watches Python files for changes and reloads the server automatically. No need to restart after every model change.
- **Use `--dev=all`** when debugging hard issues. It adds SQL query logging and asset debugging on top of reload.
- **Keep your custom addons in a separate directory** from the Odoo source. Point to both in `addons_path`. This makes updates and git management much cleaner.
- **Use a separate database per project.** Don't develop multiple features in the same DB. Create/restore backups to test fresh installs.
- **Version your addons directory with git.** Commit after every working change. Makes it trivial to revert when experiments go wrong.
- **Test module install on a fresh database** before deploying. Update-only testing misses issues in `__init__.py`, security CSV, and data files.
- **Use `odoo-bin scaffold`** to start new modules. It generates the correct folder structure, manifest, and __init__.py files.
- **Don't edit core Odoo files.** Ever. Always extend via inheritance. If you must patch core behavior, use `_patch()` or a monkey-patch in `post_init_hook`, and document it heavily.

## Odoo Technical Terms Reference

Comprehensive reference of key Odoo terms, their French translations, module locations, and workflow connections. Use this as a quick lookup for what each term means and where to find it in the UI.

| Term (English) | Term (French) | Module | Example Workflow / Connection |
|----------------|---------------|--------|------------------------------|
| **Lead / Opportunity** | Prospect / Opportunité | CRM | Convert to Quotation → Sales Order → Invoice |
| **Quotation** | Devis | Sales | Create → Send to customer → Confirm as Sales Order |
| **Sales Order** | Commande client / Bon de commande | Sales | Confirm → Deliver products → Invoice customer |
| **Order Tracking** | Suivi des commandes | Sales | Track order status from confirmation to delivery |
| **Loyalty** | Fidélité | Sales | Reward repeat customers |
| **Invoice** | Facture (client) | Accounting | Create → Validate → Send to customer → Register payment |
| **Bill / Supplier Invoice** | Facture fournisseur | Accounting / Purchasing | Receive bill → Validate → Schedule payment → Register payment |
| **Credit Note** | Avoir | Accounting | Cancel or refund part/all of an invoice, often linked to returns |
| **Chart of Accounts** | Plan Comptable | Accounting | Base of accounting: defines all accounts for financial transactions |
| **Accounting Journal** | Journal Comptable | Accounting | Register where all financial transactions are recorded |
| **Customer Payment** | Paiement Client | Accounting | Payment received from customer to settle an invoice |
| **Supplier Payment** | Paiement Fournisseur | Accounting | Payment made to supplier to settle a supplier invoice |
| **Bank Reconciliation** | Rapprochement Bancaire | Accounting | Match bank transactions with recorded entries |
| **General Ledger** | Grand Livre | Accounting | Register of all accounting transactions in a single document |
| **Balance Sheet** | Bilan | Accounting | Assets, liabilities, and equity at a given date |
| **Profit and Loss** | Compte de Résultat | Accounting | Revenue and expenses over a period |
| **Taxes** | Taxes / TVA | Accounting | Tax rules applied to sales and purchase transactions |
| **Journal Entries** | Écritures Comptables | Accounting | Individual entries documenting financial transactions |
| **Depreciation** | Amortissement | Accounting | Loss of value of fixed assets over time |
| **Open Invoices** | Encours | Accounting | Invoices still open and unpaid |
| **Accounting Analysis** | Analyse Comptable | Accounting | Examine financial flows by account categories |
| **Payroll** | Paie | HR | Employee contract → Generate payslip → Approve → Process payment |
| **Leave Request** | Demande de congé | HR | Employee submits → Manager approves → Track absence |
| **Purchase Order** | Bon de commande / Commande fournisseur | Purchasing | Create PO → Approve → Receive goods → Register bill |
| **Purchase** | Achat | Purchasing | Link to supplier orders and stock replenishment |
| **Stock Move** | Mouvement de stock | Inventory | Receive stock → Store in warehouse → Deliver to customer |
| **Picking** | Préparation de commande | Inventory | Pick items from stock → Pack → Ship to customer |
| **Pack** | Emballage | Inventory | Group picked items → Prepare for shipping |
| **Ship** | Expédition | Inventory | Send packed items to customers |
| **Receipt** | Réception | Inventory | Stock update when products are received from suppliers |
| **Inventory / Stock** | Stock | Inventory | Updated with sales, purchases, receipts → linked to warehouse management |
| **Warehouse** | Entrepôt / Stockage | Inventory | Physical location where stock is stored |
| **Reordering Rule** | Règle de réapprovisionnement | Inventory | Define min/max stock levels → Auto-trigger purchase or production |
| **Buy to Stock** | Achat sur stock | Inventory | Purchase goods to stock → Store in warehouse → Sell from stock |
| **Buy to Order** | Achat sur commande | Inventory | Order placed → Purchase goods from supplier → Deliver to customer |
| **Dropshipping** | Livraison directe | Inventory | Customer order → Purchase from supplier → Supplier ships to customer |
| **Backorder** | Commande en attente | Inventory | Partially deliver an order → Create a backorder for missing items |
| **Scrap** | Mise au rebut | Inventory | Remove damaged or obsolete stock from inventory |
| **Barcode Scanning** | Lecture de codes-barres | Inventory | Use barcode scanner for quick stock moves and picking |
| **Make to Stock** | Production sur stock | MRP | Produce goods → Store in warehouse → Sell from stock |
| **Make to Order** | Production sur commande | MRP | Customer order → Manufacture product → Deliver |
| **Manufacturing Order** | Ordre de fabrication | MRP | Create order → Consume raw materials → Produce finished goods |
| **Work Order** | Ordre de travail | MRP | Schedule operations → Execute tasks → Complete manufacturing process |
| **Raw Material** | Matière première | MRP / Inventory | Used to manufacture finished goods → linked to inventory |
| **Report** | Rapport | All modules | Generated from all data to view company performance |
| **Automation** | Automatisation | All modules | Execute actions automatically (notifications, document creation) |
| **Workflow: Pick & Pack & Ship** | Préparer & Emballer & Expédier | Inventory | Pick items from warehouse → Pack → Ship to customer |
| **Workflow: Backordering** | Commande en attente | Inventory | Auto-create backorder for missing stock when partially delivering |
| **Workflow: Scrap Management** | Gestion des mises au rebut | Inventory | Use 'Scrap' function to remove damaged/obsolete stock |

**Access paths by module:**

- **Sales / CRM:** Ventes > Prospects, Commandes, Devis, Programme de fidélité
- **Accounting:** Comptabilité > Factures, Paiements, Rapprochement bancaire, Rapports
- **Purchasing:** Achats > Commandes fournisseurs
- **Inventory:** Stocks > Réceptions, Produits, Entrepôts, Mouvements de stock
- **Manufacturing:** Fabrication > Ordres de fabrication
- **HR:** Ressources Humaines > Paie, Congés
- **Technical / Configuration:** Comptabilité > Configuration > Plan comptable, Journaux, Taxes

