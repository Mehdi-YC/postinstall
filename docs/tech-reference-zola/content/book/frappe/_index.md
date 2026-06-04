+++
title = "Frappe Framework Reference"
description = "DocTypes, controllers, hooks, the ORM, REST API, background jobs, and bench — everything the docs underexplain"
[extra]
date = 2026-06-04
+++


# Frappe Framework Reference

*DocTypes, controllers, hooks, the ORM, REST API, background jobs, and bench —
everything the official docs underexplain.*

# Project Structure

```bash,linenos
    # Create a new bench
    bench init my-bench --frappe-branch version-15
    cd my-bench

    # Install an app
    bench get-app https://github.com/your/app
    bench get-app --branch version-15 erpnext   # specific branch

    # Create a new app from scratch
    bench new-app my_app

    # Create a site
    bench new-site mysite.localhost --install-app my_app

    # A bench directory looks like this:
    my-bench/
    ├── apps/
    │   ├── frappe/          # core framework
    │   └── my_app/          # your app
    │       ├── my_app/
    │       │   ├── hooks.py             # app-wide configuration
    │       │   ├── modules.txt          # list of modules
    │       │   └── my_module/
    │       │       ├── doctype/
    │       │       │   └── my_doctype/
    │       │       │       ├── my_doctype.json      # schema
    │       │       │       ├── my_doctype.py        # controller
    │       │       │       └── my_doctype_list.js   # list view config
    │       │       └── ...
    ├── sites/
    │   └── mysite.localhost/
    │       ├── site_config.json    # DB credentials, site settings
    │       └── private/files/
    └── Procfile
```

# Bench CLI

```bash,linenos
    # Development
    bench start                        # start all processes (web, worker, scheduler)
    bench --site mysite console        # interactive Python REPL with frappe loaded
    bench --site mysite mariadb        # open MariaDB shell for the site
    bench --site mysite migrate        # run pending migrations (after schema changes)
    bench --site mysite clear-cache    # clear redis cache
    bench --site mysite clear-website-cache

    # App & site management
    bench --site mysite install-app my_app
    bench --site mysite uninstall-app my_app
    bench --site mysite backup
    bench --site mysite restore path/to/backup.sql.gz

    # Building assets
    bench build                        # build JS/CSS bundles for all apps
    bench build --app my_app           # build only your app
    bench watch                        # watch and rebuild on change (dev)

    # Running scripts
    bench --site mysite execute my_app.utils.some_function
    bench --site mysite run-script path/to/script.py

    # Production
    bench setup production <user>      # configure nginx + supervisor
    bench restart                      # restart web + workers
    bench update                       # pull all apps + migrate + build
```

# DocType Schema

A DocType is defined by its `.json` file. You rarely edit it by hand — use the
Desk UI — but understanding the structure is essential.

```json,linenos
    {
      "name": "Project",
      "module": "My Module",
      "doctype": "DocType",
      "is_submittable": 0,
      "track_changes": 1,
      "fields": [
        {
          "fieldname": "title",
          "fieldtype": "Data",
          "label": "Title",
          "reqd": 1,
          "in_list_view": 1
        },
        {
          "fieldname": "status",
          "fieldtype": "Select",
          "label": "Status",
          "options": "Open\nIn Progress\nClosed",
          "default": "Open"
        },
        {
          "fieldname": "due_date",
          "fieldtype": "Date",
          "label": "Due Date"
        },
        {
          "fieldname": "tasks_section",
          "fieldtype": "Section Break",
          "label": "Tasks"
        },
        {
          "fieldname": "tasks",
          "fieldtype": "Table",
          "label": "Tasks",
          "options": "Project Task"   
        }
      ],
      "permissions": [
        { "role": "System Manager", "read": 1, "write": 1, "create": 1, "delete": 1 }
      ]
    }
```

# Field Types

| Fieldtype | Stores | Notes |
|-----------|--------|-------|
| `Data` | Short string | Single line, max 140 chars by default |
| `Text` | Long string | Multiline, no formatting |
| `Long Text` | Very long string | For large content |
| `Small Text` | Medium string | Textarea |
| `Text Editor` | HTML | Rich text via Quill |
| `Markdown Editor` | Markdown | Rendered as HTML |
| `Int` | Integer | |
| `Float` | Float | |
| `Currency` | Decimal | Respects currency precision setting |
| `Percent` | Float | Displayed as % |
| `Check` | 0 or 1 | Renders as checkbox |
| `Date` | Date | |
| `Datetime` | Datetime | |
| `Time` | Time | |
| `Select` | String | Options defined as newline-separated values |
| `Link` | String (foreign key) | References another DocType |
| `Dynamic Link` | String | FK to a DocType named in another field |
| `Table` | Child rows | References a Child DocType |
| `Attach` | File URL | Single file attachment |
| `Attach Image` | Image URL | Image attachment with preview |
| `JSON` | JSON string | Stored as text, parsed on access |
| `Section Break` | — | Layout only, groups fields visually |
| `Column Break` | — | Layout only, starts a new column |
| `Tab Break` | — | Layout only, starts a new tab |

# Controller (Python)

The controller is a Python class that inherits from `Document`. Frappe calls
lifecycle hooks automatically — you never instantiate or call these yourself.

```python,linenos
    import frappe
    from frappe.model.document import Document

    class Project(Document):

        # ── Lifecycle hooks (called in this order on save) ──────────────────

        def before_insert(self):
            # Runs before the document is inserted for the first time.
            # Good for setting computed defaults.
            if not self.code:
                self.code = frappe.generate_hash(length=8)

        def after_insert(self):
            # Runs after the first INSERT is committed.
            frappe.publish_realtime("project_created", {"name": self.name})

        def validate(self):
            # Runs before every save (insert + update).
            # Raise frappe.ValidationError to abort.
            if self.due_date and self.due_date < frappe.utils.today():
                frappe.throw("Due date cannot be in the past", frappe.ValidationError)

        def before_save(self):
            # Runs after validate, before the DB write.
            self.last_modified_by = frappe.session.user

        def on_update(self):
            # Runs after every save is committed.
            # Safe to trigger side-effects here.
            self.notify_assignees()

        def before_submit(self):
            # Only called on submittable DocTypes when status → Submitted.
            if not self.tasks:
                frappe.throw("Cannot submit a project with no tasks")

        def on_submit(self):
            self.db_set("status", "In Progress")  # direct DB update, no re-save

        def before_cancel(self):
            pass

        def on_cancel(self):
            self.db_set("status", "Cancelled")

        def on_trash(self):
            # Runs before the document is deleted.
            frappe.db.delete("Comment", {"reference_name": self.name})

        # ── Helper methods ────────────────────────────────────────────────

        def notify_assignees(self):
            for task in self.tasks:
                if task.assigned_to:
                    frappe.sendmail(
                        recipients=[task.assigned_to],
                        subject=f"Task updated in {self.title}",
                        message=f"Task {task.title} was updated."
                    )
```

<blockquote class="warn">
<p><strong>Never call <code>self.save()</code> inside <code>validate()</code> or <code>on_update()</code>.</strong> It causes infinite recursion. Use <code>self.db_set("field", value)</code> for direct column updates that bypass the lifecycle, or set <code>self.field = value</code> before the current save completes.</p>
</blockquote>

# The ORM: frappe.db

```python,linenos
    import frappe

    # ── Single value ──────────────────────────────────────────────────────

    # get a field value from a document
    status = frappe.db.get_value("Project", "PROJ-0001", "status")

    # get multiple fields at once (returns a dict)
    data = frappe.db.get_value("Project", "PROJ-0001", ["status", "due_date"], as_dict=True)

    # ── Lists ─────────────────────────────────────────────────────────────

    # get all documents matching filters
    projects = frappe.db.get_all(
        "Project",
        filters={"status": "Open"},
        fields=["name", "title", "due_date"],
        order_by="due_date asc",
        limit=50
    )
    # returns a list of dicts: [{"name": "PROJ-001", "title": "...", ...}, ...]

    # get_list: same as get_all but respects user permissions
    projects = frappe.db.get_list("Project", filters={"status": "Open"}, fields=["name", "title"])

    # get_all with OR filters
    projects = frappe.db.get_all(
        "Project",
        filters=[["status", "in", ["Open", "In Progress"]]],
        fields=["name", "title"]
    )

    # ── Load a full document ──────────────────────────────────────────────

    doc = frappe.get_doc("Project", "PROJ-0001")
    doc.title = "Updated Title"
    doc.save()

    # create a new document
    doc = frappe.get_doc({
        "doctype": "Project",
        "title": "New Project",
        "status": "Open"
    })
    doc.insert()
    frappe.db.commit()   # always commit after insert/update outside a request context

    # ── Direct DB writes ──────────────────────────────────────────────────

    # update a single field without loading the full document
    frappe.db.set_value("Project", "PROJ-0001", "status", "Closed")

    # update multiple fields
    frappe.db.set_value("Project", "PROJ-0001", {
        "status": "Closed",
        "due_date": "2025-12-31"
    })

    # raw SQL (escape your inputs — never use .format() with user input)
    results = frappe.db.sql("""
        SELECT name, title FROM `tabProject`
        WHERE status = %(status)s AND due_date < %(today)s
    """, {"status": "Open", "today": frappe.utils.today()}, as_dict=True)

    # ── Existence checks ──────────────────────────────────────────────────

    frappe.db.exists("Project", "PROJ-0001")          # returns name or None
    frappe.db.exists("Project", {"status": "Open"})   # with filters
    frappe.db.count("Project", {"status": "Open"})    # count matching rows
```

<blockquote class="tip">
<p><strong>Table names in raw SQL are always <code>`tab{DocType}`</code>.</strong> A DocType named <code>Sales Order</code> lives in <code>`tabSales Order`</code>. Frappe adds the prefix automatically in ORM methods but not in raw <code>frappe.db.sql()</code> — you must write it yourself.</p>
</blockquote>

# hooks.py

`hooks.py` is the central wiring file for your app. It controls how your app
integrates with Frappe's event system.

```python,linenos
    # my_app/hooks.py

    app_name = "my_app"
    app_title = "My App"
    app_publisher = "Your Name"
    app_description = "What this app does"
    app_version = "0.0.1"
    app_email = "you@example.com"
    app_license = "MIT"

    # ── Document events ───────────────────────────────────────────────────
    # Hook into any DocType's lifecycle from outside its controller.
    # Useful for cross-app logic without modifying the original controller.

    doc_events = {
        "User": {
            "after_insert": "my_app.handlers.user.on_user_created",
            "on_update": "my_app.handlers.user.on_user_updated",
        },
        "*": {
            # runs on every DocType
            "on_submit": "my_app.handlers.audit.log_submission",
        }
    }

    # ── Scheduled tasks ───────────────────────────────────────────────────

    scheduler_events = {
        "all":            ["my_app.tasks.every_5_minutes"],     # ~every 5 min
        "hourly":         ["my_app.tasks.hourly_sync"],
        "daily":          ["my_app.tasks.daily_cleanup"],
        "weekly":         ["my_app.tasks.weekly_report"],
        "monthly":        ["my_app.tasks.monthly_invoice"],
        "cron": {
            "0 9 * * 1-5": ["my_app.tasks.weekday_morning"],    # Mon-Fri 9am
        }
    }

    # ── Fixtures ──────────────────────────────────────────────────────────
    # Records exported with `bench export-fixtures` and imported on migrate.

    fixtures = [
        "Custom Field",
        "Property Setter",
        {"dt": "Role", "filters": [["role_name", "like", "My App%"]]},
    ]

    # ── Other common hooks ────────────────────────────────────────────────

    # Extend an existing DocType with extra fields (no forking required)
    # Define the fields in a Custom Field fixture instead.

    # Override a controller method from another app
    override_doctype_class = {
        "Sales Invoice": "my_app.overrides.CustomSalesInvoice"
    }

    # Add JS/CSS to the desk globally
    app_include_js = ["assets/my_app/js/global.js"]
    app_include_css = ["assets/my_app/css/global.css"]

    # Add JS to a specific form
    doctype_js = {
        "Project": "public/js/project.js"
    }
```

# REST API

Every DocType automatically gets a REST API. No extra code needed.

```bash,linenos
    # Authentication: get a token
    POST /api/method/login
    { "usr": "admin@example.com", "pwd": "password" }
    # → sets a session cookie, or use token auth below

    # Token auth (generate in User > API Access)
    Authorization: token api_key:api_secret

    # ── CRUD ──────────────────────────────────────────────────────────────

    # List documents
    GET /api/resource/Project
    GET /api/resource/Project?filters=[["status","=","Open"]]&fields=["name","title"]&limit=20

    # Get a single document
    GET /api/resource/Project/PROJ-0001

    # Create
    POST /api/resource/Project
    { "title": "New Project", "status": "Open" }

    # Update (partial — only sends changed fields)
    PUT /api/resource/Project/PROJ-0001
    { "status": "Closed" }

    # Delete
    DELETE /api/resource/Project/PROJ-0001
```

```python,linenos
    # ── Whitelisted methods ───────────────────────────────────────────────
    # Expose a Python function as an API endpoint.

    import frappe

    @frappe.whitelist()
    def get_project_summary(project_name):
        # frappe.session.user is available — you know who is calling
        doc = frappe.get_doc("Project", project_name)
        return {
            "title": doc.title,
            "task_count": len(doc.tasks),
            "open_tasks": sum(1 for t in doc.tasks if t.status == "Open")
        }

    # Call it from JS or curl:
    # POST /api/method/my_app.api.get_project_summary
    # { "project_name": "PROJ-0001" }

    @frappe.whitelist(allow_guest=True)
    def public_endpoint():
        # allow_guest=True → no login required
        return {"status": "ok"}
```

<blockquote class="warn">
<p><strong><code>@frappe.whitelist()</code> does not check permissions.</strong> Anyone authenticated can call it. Add explicit <code>frappe.has_permission()</code> checks inside the function if the data is sensitive.</p>
</blockquote>

# Permissions

```python,linenos
    import frappe

    # Check if current user can access a document
    frappe.has_permission("Project", doc="PROJ-0001", throw=True)
    # throw=True raises PermissionError automatically; throw=False returns bool

    # Check a specific permission type
    frappe.has_permission("Project", ptype="write", doc="PROJ-0001", throw=True)
    # ptype options: read, write, create, delete, submit, cancel, amend

    # Get current user
    frappe.session.user          # "admin@example.com"
    frappe.session.user == "Guest"  # True for unauthenticated requests

    # Check role
    frappe.get_roles(frappe.session.user)           # ["System Manager", "My Role", ...]
    "System Manager" in frappe.get_roles()          # True/False

    # Programmatic permission bypass (use with care — only in trusted server scripts)
    frappe.set_user("Administrator")
    # ... do privileged work ...
    frappe.set_user(original_user)
```

# Background Jobs

Long-running tasks should never block a web request. Enqueue them.

```python,linenos
    import frappe

    # Enqueue a function to run in a background worker
    frappe.enqueue(
        "my_app.tasks.send_bulk_email",     # dotted path to function
        queue="long",                        # default, short, long
        timeout=600,                         # seconds before job is killed
        job_name="bulk_email_job",           # optional, for deduplication
        # kwargs passed to the function:
        project_name="PROJ-0001",
        notify_all=True
    )

    # The function itself (runs in a worker process)
    def send_bulk_email(project_name, notify_all=False):
        doc = frappe.get_doc("Project", project_name)
        recipients = get_recipients(doc, all=notify_all)
        for r in recipients:
            frappe.sendmail(recipients=[r], subject="...", message="...")
        frappe.db.commit()   # always commit in background jobs

    # Enqueue with deduplication (won't enqueue if job_name already queued)
    frappe.enqueue(
        "my_app.tasks.sync",
        job_name="sync_job",
        deduplicate=True
    )
```

# Jinja Templates

Frappe uses Jinja2 for print formats, email templates, and web pages.

```python,linenos
    # Render a Jinja string from Python
    html = frappe.render_template(
        "<p>Hello {{ doc.title }}</p>",
        {"doc": doc}
    )

    # Render a template file (relative to app root)
    html = frappe.render_template("my_app/templates/email/welcome.html", context)
```

```html,linenos
    {# ── Frappe-specific Jinja globals available in all templates ── #}

    {# Current user #}
    {{ frappe.session.user }}

    {# Translate a string #}
    {{ _("Save") }}

    {# Format a date #}
    {{ frappe.format_date(doc.due_date) }}

    {# Format a currency value #}
    {{ frappe.format_value(doc.amount, {"fieldtype": "Currency"}) }}

    {# Link to a document #}
    <a href="/app/project/{{ doc.name }}">{{ doc.title }}</a>

    {# Conditional #}
    {% if doc.status == "Open" %}
      <span class="badge">Open</span>
    {% endif %}

    {# Loop over child table #}
    {% for task in doc.tasks %}
      <li>{{ task.title }} — {{ task.assigned_to }}</li>
    {% endfor %}
```

# Migrations and Patches

```python,linenos
    # patches.txt: list of patch modules to run on migrate (one per line, never remove old ones)
    # my_app/patches.txt
    my_app.patches.v1_0.set_default_status
    my_app.patches.v1_1.backfill_project_codes

    # A patch file
    # my_app/patches/v1_1/backfill_project_codes.py
    import frappe

    def execute():
        # Runs once per site on `bench migrate`
        projects = frappe.db.get_all("Project", filters={"code": ""}, fields=["name"])
        for p in projects:
            frappe.db.set_value("Project", p.name, "code", frappe.generate_hash(length=8))
        frappe.db.commit()
```

<blockquote class="tip">
<p><strong>Patches run exactly once per site.</strong> Frappe tracks executed patches in the <code>tabPatch Log</code> table. If a patch fails halfway, fix it and run <code>bench --site mysite migrate</code> again — it will retry only the failed patch.</p>
</blockquote>

# Useful frappe Utilities

```python,linenos
    import frappe
    from frappe.utils import (
        today, now, nowdate, nowdatetime,
        add_days, add_months, date_diff,
        flt, cint, cstr,
        get_url, get_site_name
    )

    # Dates
    today()                          # "2025-06-04"  (string)
    nowdatetime()                    # "2025-06-04 14:32:00"
    add_days(today(), 7)             # one week from now
    date_diff("2025-12-31", today()) # days between two dates

    # Type coercion (safe — never raises)
    flt("3.14")    # 3.14  (float)
    cint("42")     # 42    (int), cint(None) → 0
    cstr(None)     # ""    (str)

    # Messaging (only works inside a web request)
    frappe.msgprint("Something happened")           # toast on the desk
    frappe.throw("Validation failed")               # raises ValidationError + shows message
    frappe.log_error("something broke", title="My App Error")  # writes to Error Log

    # Caching
    frappe.cache.set_value("my_key", {"data": 1}, expires_in_sec=300)
    frappe.cache.get_value("my_key")

    # Generating names
    frappe.generate_hash(length=10)
    frappe.get_id()                   # UUID
```