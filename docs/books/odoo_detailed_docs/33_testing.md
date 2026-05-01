# 33 — Unit tests

Odoo's test framework extends Python's `unittest`. Tests run in a transaction
that rolls back after each test class, so no test data persists.

## Test file location

```
crm_custom/
└── tests/
    ├── __init__.py          # from . import test_crm_lead
    └── test_crm_lead.py
```

Add `tests/` to `.gitignore` is optional — tests are NOT listed in `__manifest__['data']`.
Odoo discovers them automatically.

## Basic test class

```python
from odoo.tests import TransactionCase, tagged

@tagged('post_install', '-at_install', 'crm_custom')
class TestCrmLead(TransactionCase):
    """Tests for crm.lead model."""

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        # Runs ONCE for the class — shared, expensive setup
        cls.category = cls.env['crm.lead.category'].create({
            'name': 'Test Category',
            'code': 'test',
        })
        cls.user = cls.env.ref('base.user_demo')

    def setUp(self):
        super().setUp()
        # Runs before EACH test — isolated per-test setup
        self.lead = self.env['crm.lead'].create({
            'name':        'Test Lead',
            'category_id': self.category.id,
        })

    def test_initial_state(self):
        self.assertEqual(self.lead.state, 'new')
        self.assertEqual(self.lead.source, 'manual')

    def test_qualify(self):
        self.lead.action_qualify()
        self.assertEqual(self.lead.state, 'qualified')

    def test_negative_revenue_raises(self):
        from odoo.exceptions import ValidationError
        with self.assertRaises(ValidationError):
            self.lead.write({'expected_revenue': -1})

    def test_call_count(self):
        self.env['crm.lead.call'].create({
            'lead_id': self.lead.id,
            'answered': True,
        })
        self.lead.invalidate_recordset()   # flush cache
        self.assertEqual(self.lead.call_count, 1)

    def test_bulk_qualify(self):
        leads = self.env['crm.lead'].create([
            {'name': f'Lead {i}', 'category_id': self.category.id}
            for i in range(3)
        ])
        leads.action_bulk_qualify()
        self.assertTrue(all(l.state == 'qualified' for l in leads))

    def test_cannot_delete_won(self):
        from odoo.exceptions import UserError
        self.lead.write({'state': 'won'})
        with self.assertRaises(UserError):
            self.lead.unlink()
```

## Running tests

```bash
# Run all tests in your module
./odoo-bin -d mydb --test-enable --stop-after-init -i crm_custom

# Run only tests with a specific tag
./odoo-bin -d mydb --test-enable --test-tags crm_custom --stop-after-init

# Run a specific test class
./odoo-bin -d mydb --test-enable --test-tags /crm_custom/TestCrmLead --stop-after-init
```

## Test class types

| Class | Use when |
|---|---|
| `TransactionCase` | Standard — each test class in its own transaction (rolled back) |
| `HttpCase` | Full HTTP + browser tests (portal, controllers) |
| `SingleTransactionCase` | All tests share one transaction (faster, less isolated) |

## Tags

```python
@tagged('post_install', '-at_install', 'crm_custom')
```

| Tag | Meaning |
|---|---|
| `post_install` | Run after all modules are installed (default for most tests) |
| `-at_install` | Do NOT run during module installation |
| `at_install` | Run during module installation (for install-time tests) |
| `'crm_custom'` | Custom tag — use `--test-tags crm_custom` to filter |

## Useful assertions

```python
self.assertEqual(a, b)
self.assertNotEqual(a, b)
self.assertTrue(expr)
self.assertFalse(expr)
self.assertIn(item, container)
self.assertRaises(ExceptionClass, func, *args)

# Context manager form (most common in Odoo)
with self.assertRaises(UserError):
    self.lead.unlink()

# Record assertions
self.assertRecordValues(leads, [
    {'name': 'Lead A', 'state': 'new'},
    {'name': 'Lead B', 'state': 'qualified'},
])
```
