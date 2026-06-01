+++
title = "Django Development Reference"
description = "Project structure, ORM, views, templates, forms, admin, and dev operations"
[extra]
date = 2024-01-15
+++

# Django Development Reference

*Project structure, ORM, views, templates, forms, admin, and dev operations.*

## File Structure

A Django **project** contains one or more **apps**. The project is the configuration container; apps are the reusable units of functionality.

    myproject/
    ├── manage.py                  # CLI entry point. Don't edit this file.
    ├── myproject/                # Project configuration package (same name as project)
    │   ├── __init__.py          # Empty. Makes this a Python package.
    │   ├── settings.py          # All project settings (DB, apps, middleware, etc.)
    │   ├── urls.py              # Root URL configuration. Delegates to app URL configs.
    │   ├── wsgi.py              # WSGI entry point for production servers.
    │   └── asgi.py              # ASGI entry point for async servers (Channels).
    ├── myapp/                     # Your app (created with python manage.py startapp myapp)
    │   ├── __init__.py
    │   ├── apps.py              # App configuration (name, verbose_name, default_auto_field)
    │   ├── models.py            # ORM model definitions
    │   ├── views.py             # View functions or classes
    │   ├── urls.py              # App-specific URL patterns
    │   ├── admin.py             # Admin site registration and customization
    │   ├── forms.py             # Form and ModelForm definitions
    │   ├── serializers.py       # DRF serializers (if using Django REST Framework)
    │   ├── signals.py           # Signal handlers (optional, but good to separate)
    │   ├── tests.py             # Tests (or a tests/ package for more structure)
    │   ├── migrations/          # Auto-generated migration files. NEVER edit manually.
    │   │   ├── __init__.py
    │   │   └── 0001_initial.py       # First migration (auto-generated)
    │   ├── templates/           # App-specific templates (if using app-level template dirs)
    │   │   └── myapp/
    │   │       └── model_list.html
    │   └── static/              # App-specific static files (CSS, JS, images)
    │       ├── css/
    │       ├── js/
    │       └── img/
    ├── templates/                 # Project-level templates (shared across apps)
    │   └── base.html            # Base template with shared layout
    ├── static/                     # Project-level static files
    ├── requirements.txt           # Python dependencies
    ├── .env                        # Environment variables (NEVER commit this)
    └── .gitignore                  # Git ignore rules

> **One project, many apps.** The project folder holds configuration. Each app is a self-contained unit with its own models, views, templates. Apps should be reusable across projects in theory.

> **Split large apps.** If `models.py` exceeds ~300 lines, split into a `models/` package with one file per model group. Same for views. Django supports this natively.

## settings.py

The central configuration file. Key settings every developer needs to know:

    # myproject/settings.py

    # Base directory of the project (for building paths)
    import os
    from pathlib import Path
    BASE_DIR = Path(__file__).resolve().parent.parent

    # SECURITY: NEVER hardcode in production. Use environment variables.
    # python-decouple or django-environ are the standard approaches.
    from decouple import config

    SECRET_KEY = config('SECRET_KEY')
    DEBUG = config('DEBUG', default=False, cast=bool)

    ALLOWED_HOSTS = config('ALLOWED_HOSTS', default='', cast=lambda v: [s.strip() for s in v.split(',')])

    # Installed apps: order matters. Your apps come after Django apps.
    INSTALLED_APPS = [
        'django.contrib.admin',
        'django.contrib.auth',
        'django.contrib.contenttypes',
        'django.contrib.sessions',
        'django.contrib.messages',
        'django.contrib.staticfiles',
        # Third-party apps
        'rest_framework',
        'crispy_forms',
        'crispy_tailwind',
        # Your apps
        'myapp',
        'myapp2',
    ]

    # Middleware: processed top-to-bottom for request, bottom-to-top for response.
    MIDDLEWARE = [
        'django.middleware.security.SecurityMiddleware',
        'django.contrib.sessions.middleware.SessionMiddleware',
        'django.middleware.common.CommonMiddleware',
        'django.middleware.csrf.CsrfViewMiddleware',
        'django.contrib.auth.middleware.AuthenticationMiddleware',
        'django.contrib.messages.middleware.MessageMiddleware',
    ]

    # Database
    DATABASES = {
        'default': {
            'ENGINE': 'django.db.backends.postgresql',
            'NAME': config('DB_NAME'),
            'USER': config('DB_USER'),
            'PASSWORD': config('DB_PASSWORD'),
            'HOST': config('DB_HOST', default='localhost'),
            'PORT': config('DB_PORT', default='5432'),
            'CONN_MAX_AGE': 60,  # Persistent connections (seconds). 60 is a good default.
        }
    }

    # URL configuration
    ROOT_URLCONF = 'myproject.urls'

    # Templates: DIRS looks for project-level templates.
    # App-level templates are found automatically in each app's templates/ dir.
    TEMPLATES = [{
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
        'DIRS': [BASE_DIR / 'templates'],
        'APP_DIRS': True,  # Auto-discover templates in each app's templates/ dir
        'OPTIONS': {
            'context_processors': [
                'django.template.context_processors.debug',
                'django.template.context_processors.request',
                'django.contrib.auth.context_processors.auth',
                'django.contrib.messages.context_processors.messages',
            ],
        },
    }]

    # Static files (CSS, JS, images)
    STATIC_URL = '/static/'
    STATICFILES_DIRS = [BASE_DIR / 'static']  # Additional static dirs (project-level)
    STATIC_ROOT = BASE_DIR / 'staticfiles'     # Where collectstatic puts files (for production)

    # Media files (user uploads)
    MEDIA_URL = '/media/'
    MEDIA_ROOT = BASE_DIR / 'media'

    # Auth
    AUTH_USER_MODEL = 'myapp.CustomUser'  # If you have a custom user model
    LOGIN_URL = 'login'
    LOGIN_REDIRECT_URL = 'home'
    LOGOUT_REDIRECT_URL = 'home'

    # Default primary key field type (Django 3.2+)
    DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'

    # Internationalization
    LANGUAGE_CODE = 'en-us'
    TIME_ZONE = 'UTC'  # Always UTC in the database. Convert in templates.
    USE_TZ = True  # Keep True. Stores all datetimes as UTC in DB.

> **`SECRET_KEY` in version control = compromised.** Use `python-decouple` or `django-environ` to load from `.env`. Add `.env` to `.gitignore`.

    # .env (never commit this)
    SECRET_KEY=django-insecure-xxx-change-this-in-production
    DEBUG=True
    ALLOWED_HOSTS=localhost,127.0.0.1
    DB_NAME=mydb
    DB_USER=myuser
    DB_PASSWORD=mypassword
    DB_HOST=localhost
    DB_PORT=5432

    # .env.example (commit this)
    SECRET_KEY=
    DEBUG=False
    ALLOWED_HOSTS=
    DB_NAME=
    DB_USER=
    DB_PASSWORD=
    DB_HOST=localhost
    DB_PORT=5432

> **`USE_TZ = True` is the standard.** All datetimes are stored as UTC in PostgreSQL. Use `{{ mydate|localtime }}` in templates to convert to the user's timezone, or set `TIME_ZONE` in settings for a fixed display timezone.

## Models (models.py)

    # myapp/models.py
    from django.db import models
    from django.core.validators import MinValueValidator, MaxLengthValidator
    from django.urls import reverse


    class Category(models.Model):
        # ---- Field Types ----

        # CharField: short text. ALWAYS specify max_length.
        name = models.CharField(max_length=100, unique=True)

        # SlugField: for URL-friendly strings.
        # auto_created or prepopulated via admin prepopulated_fields.
        slug = models.SlugField(max_length=120, unique=True)

        # BooleanField: use null=True only if you need None (three-state).
        # For true/false, default is fine. In forms, use forms.NullBooleanSelect.
        is_active = models.BooleanField(default=True)

        # IntegerField: whole numbers.
        sort_order = models.IntegerField(default=0)

        # DateTimeField: auto_now_add on create, auto_now on every save.
        # IMPORTANT: These cannot be edited manually. Use null=True if you want
        # to set them explicitly in some cases.
        created_at = models.DateTimeField(auto_now_add=True)
        updated_at = models.DateTimeField(auto_now=True)

        # DateField: date only (no time).
        start_date = models.DateField(null=True, blank=True)

        # DecimalField: for money. ALWAYS specify max_digits and decimal_places.
        # max_digits includes the decimal places.
        price = models.DecimalField(
            max_digits=10, decimal_places=2,
            validators=[MinValueValidator(0)],
        )

        # FloatField: for non-critical decimals (not money!).
        weight = models.FloatField(null=True, blank=True)

        # TextField: long text, no max_length.
        description = models.TextField(blank=True)

        # EmailField: validates email format.
        email = models.EmailField(blank=True)

        # URLField: validates URL format.
        website = models.URLField(blank=True)

        # FileField: stores file path in DB, file on disk.
        # upload_to: subdirectory within MEDIA_ROOT.
        # Can be a callable: upload_to='uploads/%Y/%m/'
        document = models.FileField(upload_to='documents/', blank=True)

        # ImageField: FileField with image validation. Requires Pillow.
        image = models.ImageField(upload_to='images/', blank=True)

        # UUIDField: for public identifiers (instead of auto-increment PK).
        # primary_key=True replaces the default id field.
        # uuid = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)

        # JSONField: stores JSON. Queryable in PostgreSQL.
        # null=True allows NULL in DB (different from empty dict {}).
        metadata = models.JSONField(default=dict, blank=True)

        # ---- null vs blank ----
        # null=True: allows NULL in the DATABASE. For DateField, IntegerField, etc.
        # blank=True: allows empty in FORMS. For CharField, TextField, etc.
        # CharField/TextField: use blank=True, NOT null=True (empty string vs NULL).
        # DateField/IntegerField: use null=True, blank=True (can't store empty string).
        # BooleanField: NEVER use null=True unless you need three-state logic.

        # ---- Choices ----
        # Use Enumeration (Python 3.4+). Cleaner than raw tuples.
        class Status(models.TextChoices):
            DRAFT = 'draft', 'Draft'
            PUBLISHED = 'published', 'Published'
            ARCHIVED = 'archived', 'Archived'

        status = models.CharField(
            max_length=20,
            choices=Status.choices,
            default=Status.DRAFT,
        )

        # ---- Meta ----
        class Meta:
            # Default sort. Use '-' prefix for descending.
            ordering = ['-created_at', 'name']

            # Plural name in admin.
            verbose_name_plural = 'Categories'

            # Prevent duplicate combinations.
            # unique_together is older but still works.
            # constraints = [models.UniqueConstraint(fields=['name', 'status'], name='unique_name_status')]

            # Database-level permissions (beyond Django's default).
            # permissions = [('can_publish', 'Can publish items')]

            # Default manager name (rarely needed).
            # default_manager_name = 'objects'

        # ---- String representations ----
        def __str__(self):
            return self.name

        # ---- Absolute URL (for get_absolute_url in templates) ----
        def get_absolute_url(self):
            return reverse('myapp:category-detail', kwargs={'slug': self.slug})

        # ---- Methods ----
        def is_published(self):
            return self.status == self.Status.PUBLISHED


    # ---- Relationships ----

    class Product(models.Model):
        category = models.ForeignKey(
            'Category',              # 'app.Model' or the model class itself
            on_delete=models.CASCADE,    # What happens when category is deleted
            related_name='products',  # Access: category.products.all()
            related_query_name='product', # For queryset filtering: Category.objects.filter(product__name=...)
            null=True,
            blank=True,
        )
        # on_delete options:
        #   CASCADE: delete this too (default for ForeignKey)
        #   PROTECT: block deletion with ProtectedError
        #   SET_NULL: set to NULL (requires null=True)
        #   SET_DEFAULT: set to default value (requires default=)
        #   SET(): set to a specific value or callable
        #   DO_NOTHING: do nothing (you must handle it yourself)

        name = models.CharField(max_length=200)
        price = models.DecimalField(max_digits=10, decimal_places=2)
        sku = models.CharField(max_length=50, unique=True)

        # ManyToMany: no on_delete needed.
        # through: explicit intermediate model (for extra fields on the relation).
        tags = models.ManyToManyField('Tag', blank=True, related_name='products')
        # through='ProductTag'  # if you need extra fields like "added_at"

        # OneToOne: like ForeignKey with unique=True.
        # Use related_name, not the default <model>_set.
        detail = models.OneToOneField(
            'ProductDetail',
            on_delete=models.CASCADE,
            related_name='product',
            null=True,
            blank=True,
        )

        class Meta:
            ordering = ['name']
            # Composite unique constraint (preferred over unique_together)
            constraints = [
                models.UniqueConstraint(
                    fields=['name', 'category'],
                    name='unique_product_name_per_category',
                ),
            ]

        def __str__(self):
            return self.name

        # ---- Overriding save() ----
        def save(self, *args, **kwargs):
            # Pre-save logic (e.g., auto-generate slug)
            if not self.sku:
                self.sku = self.generate_sku()
            super().save(*args, **kwargs)
            # Post-save logic (e.g., clear cache)

        # ---- Overriding delete() ----
        def delete(self, *args, **kwargs):
            # Pre-delete logic (e.g., delete associated files)
            if self.image:
                self.image.delete(save=False)
            super().delete(*args, **kwargs)

        # ---- Properties (not stored in DB, computed on access) ----
        @property
        def is_expensive(self):
            return self.price > 1000
            # NOTE: cannot be used in queryset filters. Use annotated fields instead.


    # ---- Custom Managers ----
    # For reusable query logic. Attach to model via objects = MyManager().

    class ProductManager(models.Manager):
        def published(self):
            return self.filter(status='published')

        def cheap(self, threshold=100):
            return self.filter(price__lte=threshold)


    # ---- Abstract Base Model ----
    # For shared fields across models (created_at, updated_at, etc.)

    class TimeStampedModel(models.Model):
        created_at = models.DateTimeField(auto_now_add=True)
        updated_at = models.DateTimeField(auto_now=True)

        class Meta:
            abstract = True  # No table created. Fields copied into child models.

    # class Product(TimeStampedModel):  # now has created_at, updated_at
    #     name = models.CharField(...)

### QuerySet API Cheat Sheet

| Operation | Code | Returns |
|-----------|------|---------|
| Get all | `Product.objects.all()` | QuerySet |
| Filter | `Product.objects.filter(price__gt=100)` | QuerySet |
| Exclude | `Product.objects.exclude(status='draft')` | QuerySet |
| Get single | `Product.objects.get(pk=1)` | Object or DoesNotExist |
| Get or create | `Product.objects.get_or_create(sku='ABC', defaults={...})` | (object, created) |
| Update or create | `Product.objects.update_or_create(sku='ABC', defaults={...})` | (object, created) |
| Count | `Product.objects.count()` | int |
| Exists | `Product.objects.filter(...).exists()` | bool (faster than count) |
| First / Last | `Product.objects.first()` | Object or None |
| Bulk create | `Product.objects.bulk_create([...])` | List (no signals fired) |
| Bulk update | `Product.objects.filter(...).update(price=99)` | int (no signals fired) |
| Delete | `Product.objects.filter(...).delete()` | (count, details) |
| Values | `Product.objects.values('name', 'price')` | QuerySet of dicts |
| Values list | `Product.objects.values_list('name', flat=True)` | QuerySet of tuples/values |
| Select related | `Product.objects.select_related('category')` | Optimized QuerySet (FK) |
| Prefetch related | `Product.objects.prefetch_related('tags')` | Optimized QuerySet (M2M) |
| Annotate | `Product.objects.annotate(total=Sum('line__price'))` | QuerySet with extra fields |
| Order | `Product.objects.order_by('-price', 'name')` | QuerySet |
| Slice | `Product.objects.all()[:10]` | QuerySet (LIMIT 10) |

### Lookup Reference

| Lookup | Meaning | Lookup | Meaning |
|--------|---------|--------|---------|
| `__exact` | Exact match (default) | `__iexact` | Case-insensitive exact |
| `__contains` | Contains substring | `__icontains` | Case-insensitive contains |
| `__gt` | Greater than | `__gte` | Greater than or equal |
| `__lt` | Less than | `__lte` | Less than or equal |
| `__in` | In a list | `__range` | Between two values |
| `__startswith` | Starts with | `__istartswith` | Case-insensitive starts with |
| `__endswith` | Ends with | `__iendswith` | Case-insensitive ends with |
| `__isnull` | Is NULL / is not NULL | | |
| `__year` | Date year | `__month` | Date month |
| `__day` | Date day | `__hour` | Datetime hour |
| `__regex` | Regex match | `__iregex` | Case-insensitive regex |

> **`get()` raises `DoesNotExist` if no match.** Use `get_object_or_404()` in views to return a 404 page instead of a 500 error.

## Migrations

Migrations are Django's version control for the database schema. They are **auto-generated** from model changes and applied in order.

### Common Commands

| Command | What It Does |
|---------|---------------|
| `python manage.py makemigrations` | Detect model changes and create migration files |
| `python manage.py makemigrations myapp` | Only for a specific app |
| `python manage.py migrate` | Apply all pending migrations |
| `python manage.py migrate myapp 0003` | Migrate to specific migration (rollback) |
| `python manage.py showmigrations` | Show migration status (X = applied) |
| `python manage.py sqlmigrate myapp 0002` | Show SQL for a migration (without applying) |
| `python manage.py migrate --fake` | Mark as applied without running (use carefully) |
| `python manage.py migrate --fake-initial` | Skip CreateModel if table already exists |

> **Never edit migration files by hand** unless you're writing a data migration. Auto-generated operations depend on the exact structure.

### Data Migrations

For data changes (inserting, updating, deleting rows) that need to happen alongside schema changes:

    # Generated with: python manage.py makemigrations --empty myapp

    from django.db import migrations


    def set_default_status(apps, schema_editor):
        # apps: historical model registry (use this, NOT direct model import)
        # schema_editor: for raw SQL if needed
        Product = apps.get_model('myapp', 'Product')
        Product.objects.filter(status__isnull=True).update(status='draft')


    def reverse_set_default_status(apps, schema_editor):
        Product = apps.get_model('myapp', 'Product')
        Product.objects.filter(status='draft').update(status=None)


    class Migration(migrations.Migration):
        dependencies = [
            ('myapp', '0002_auto_20240101_1234'),  # MUST list dependencies
        ]
        operations = [
            migrations.RunPython(set_default_status, reverse_set_default_status),
            # RunPython(forward_func, reverse_func) -- reverse is optional but recommended
            # RunSQL("UPDATE ...", "UPDATE ... reverse ...") -- for raw SQL
        ]

> **Use `apps.get_model()` in migrations, not direct imports.** The model you import reflects the current code state, not the state at the migration's point in history. This causes subtle bugs.

## Views

### Function-Based Views (FBV)

    # myapp/views.py
    from django.shortcuts import render, redirect, get_object_or_404
    from django.http import HttpResponse, JsonResponse, Http404
    from django.contrib.auth.decorators import login_required, permission_required
    from .models import Product
    from .forms import ProductForm


    # Basic view
    def product_list(request):
        products = Product.objects.select_related('category').order_by('-created_at')
        return render(request, 'myapp/product_list.html', {
            'products': products,
        })

    # Detail view with 404
    def product_detail(request, pk):
        product = get_object_or_404(Product.objects.select_related('category'), pk=pk)
        return render(request, 'myapp/product_detail.html', {
            'product': product,
        })

    # Form handling (create)
    def product_create(request):
        if request.method == 'POST':
            form = ProductForm(request.POST, request.FILES)  # FILES for FileField/ImageField
            if form.is_valid():
                product = form.save()
                return redirect(product.get_absolute_url())
        else:
            form = ProductForm()
        return render(request, 'myapp/product_form.html', {'form': form})

    # Form handling (update)
    def product_update(request, pk):
        product = get_object_or_404(Product, pk=pk)
        if request.method == 'POST':
            form = ProductForm(request.POST, request.FILES, instance=product)
            if form.is_valid():
                form.save()
                return redirect(product.get_absolute_url())
        else:
            form = ProductForm(instance=product)
        return render(request, 'myapp/product_form.html', {'form': form})

    # JSON API endpoint
    def product_api(request, pk):
        product = get_object_or_404(Product, pk=pk)
        data = {'name': product.name, 'price': str(product.price)}
        return JsonResponse(data)

    # Login required
    @login_required
    def my_protected_view(request):
        # request.user is available
        return render(request, 'myapp/protected.html')

    # Permission required
    @permission_required('myapp.change_product', raise_exception=True)
    def product_edit(request, pk):
        ...

### Class-Based Views (CBV)

    from django.views.generic import ListView, DetailView, CreateView, UpdateView, DeleteView, TemplateView
    from django.contrib.auth.mixins import LoginRequiredMixin, PermissionRequiredMixin
    from django.urls import reverse_lazy
    from .models import Product
    from .forms import ProductForm


    class ProductListView(ListView):
        model = Product
        template_name = 'myapp/product_list.html'
        context_object_name = 'products'  # Default: object_list
        paginate_by = 20
        ordering = ['-created_at']

        # Optimize queries
        def get_queryset(self):
            return Product.objects.select_related('category').prefetch_related('tags')

        # Add extra context
        def get_context_data(self, **kwargs):
            ctx = super().get_context_data(**kwargs)
            ctx['total_count'] = Product.objects.count()
            return ctx


    class ProductDetailView(DetailView):
        model = Product
        template_name = 'myapp/product_detail.html'
        # URL kwarg default: pk. Change with pk_url_kwarg or slug_url_kwarg.
        # slug_field = 'slug'
        # slug_url_kwarg = 'slug'


    class ProductCreateView(LoginRequiredMixin, CreateView):
        model = Product
        form_class = ProductForm
        template_name = 'myapp/product_form.html'

        # Set the user automatically on save
        def form_valid(self, form):
            form.instance.created_by = self.request.user
            return super().form_valid(form)


    class ProductUpdateView(LoginRequiredMixin, PermissionRequiredMixin, UpdateView):
        model = Product
        form_class = ProductForm
        template_name = 'myapp/product_form.html'
        permission_required = 'myapp.change_product'

        # Override get_success_url instead of success_url if URL is dynamic
        def get_success_url(self):
            return reverse('myapp:product-detail', kwargs={'pk': self.object.pk})


    class ProductDeleteView(LoginRequiredMixin, DeleteView):
        model = Product
        template_name = 'myapp/product_confirm_delete.html'
        success_url = reverse_lazy('myapp:product-list')  # reverse_lazy for class-level URLs

### Common CBV Mixins

| Mixin | What It Does |
|-------|---------------|
| `LoginRequiredMixin` | Redirects to LOGIN_URL if not authenticated |
| `PermissionRequiredMixin` | Requires specific permission. Set `permission_required`. |
| `UserPassesTestMixin` | Custom test function: `test_func()` |
| `AccessMixin` | Base for login/permission. Controls `login_url`, `redirect_field_name`. |
| `FormMixin` | Form processing for non-model forms |
| `PaginatorMixin` | Adds pagination (built into ListView) |

> **Mixin order matters.** In Python MRO, the leftmost mixin wins conflicts. Put `LoginRequiredMixin` before the view class.

## URLs

    # myproject/urls.py (root)
    from django.contrib import admin
    from django.urls import path, include

    urlpatterns = [
        path('admin/', admin.site.urls),
        path('accounts/', include('django.contrib.auth.urls')),  # Login/logout/password reset
        path('myapp/', include('myapp.urls', namespace='myapp')),
        path('', include('myapp2.urls', namespace='myapp2')),
    ]

    # myapp/urls.py (app-level)
    from django.urls import path
    from . import views

    app_name = 'myapp'  # Required if using include() with namespace

    urlpatterns = [
        # FBV
        path('', views.product_list, name='product-list'),
        path('<int:pk>/', views.product_detail, name='product-detail'),
        path('create/', views.product_create, name='product-create'),
        path('<int:pk>/edit/', views.product_update, name='product-update'),

        # CBV
        path('cbv/', views.ProductListView.as_view(), name='product-list-cbv'),
        path('cbv/<int:pk>/', views.ProductDetailView.as_view(), name='product-detail-cbv'),
        path('cbv/create/', views.ProductCreateView.as_view(), name='product-create-cbv'),

        # Slug-based URL
        path('category/<slug:slug>/', views.category_detail, name='category-detail'),

        # Custom path converters: int, str, slug, uuid, path
        # path('file/<path:filepath>/', views.file_view),  # matches including /
    ]

### Path Converters

| Converter | Matches | Example URL |
|-----------|---------|-------------|
| `int` | Non-negative integers | `<int:pk>` → `42` |
| `str` | Non-empty strings (no /) | `<str:name>` → `hello` |
| `slug` | Slug strings (letters, numbers, hyphens, underscores) | `<slug:slug>` → `my-product` |
| `uuid` | UUID format | `<uuid:id>` → `a1b2c3d4-...` |
| `path` | Any string including slashes | `<path:filepath>` → `docs/file.pdf` |

> **Use `reverse()` and `{% url %}` instead of hardcoding URLs.** In Python: `reverse('myapp:product-detail', kwargs={'pk': 42})`. In templates: `{% url 'myapp:product-detail' pk=42 %}`. This means URL changes only need to happen in `urls.py`.

## Templates

### Template Inheritance

    # templates/base.html
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>{% block title %}My Site{% endblock %}</title>
        {% block extra_css %}{% endblock %}
    </head>
    <body>
        <nav>...</nav>
        <main>
            {% block content %}{% endblock %}
        </main>
        {% block extra_js %}{% endblock %}
    </body>
    </html>

    # myapp/templates/myapp/product_list.html
    {% extends "base.html" %}

    {% block title %}Products{% endblock %}

    {% block content %}
    <h1>Products</h1>

    {% if products %}
    <ul>
        {% for product in products %}
        <li>
            <a href="{{ product.get_absolute_url }}">{{ product.name }}</a>
            <span>${{ product.price }}</span>
        </li>
        {% empty %}
        <li>No products yet.</li>
        {% endfor %}
    </ul>
    {% else %}
    <p>No products found.</p>
    {% endif %}

    {% if page_obj.has_previous %}
    <a href="?page={{ page_obj.previous_page_number }}">Previous</a>
    {% endif %}
    Page {{ page_obj.number }} of {{ page_obj.paginator.num_pages }}
    {% if page_obj.has_next %}
    <a href="?page={{ page_obj.next_page_number }}">Next</a>
    {% endif %}
    {% endblock %}

### Essential Template Tags

| Tag | Purpose | Example |
|-----|---------|---------|
| `{% extends %}` | Inherit from parent template | `{% extends "base.html" %}` |
| `{% block %}` | Overridable section | `{% block content %}...{% endblock %}` |
| `{{ var }}` | Output variable (auto-escaped) | `{{ product.name }}` |
| `{% autoescape off %}` | Disable auto-escaping (XSS risk) | Use `|safe` filter instead |
| `{% if %}` / `{% elif %}` / `{% else %}` | Conditionals | `{% if user.is_authenticated %}` |
| `{% for %}` / `{% empty %}` | Loop (empty shown if no items) | `{% for item in items %}` |
| `{% url %}` | Reverse URL resolution | `{% url 'myapp:detail' pk=obj.pk %}` |
| `{% load static %}` | Load static files tag | `{% static 'css/style.css' %}` |
| `{% csrf_token %}` | CSRF token for POST forms | Inside every `<form>` |
| `{% include %}` | Include another template | `{% include "snippets/card.html" %}` |
| `{% with %}` | Assign variable in scope | `{% with total=cart.total %}{{ total }}{% endwith %}` |
| `{% comment %}` | Template comments | `{% comment %}...{% endcomment %}` |

### Essential Template Filters

| Filter | Purpose | Example |
|--------|---------|---------|
| `|safe` | Mark as safe HTML (no escaping) | `{{ html_content|safe }}` |
| `|escape` | Force HTML escaping | `{{ data|escape }}` |
| `|date` | Format a date | `{{ obj.created_at|date:"M d, Y" }}` |
| `|time` | Format a time | `{{ obj.time|time:"H:i" }}` |
| `|localtime` | Convert to current timezone | `{{ obj.created_at|localtime }}` |
| `|truncatechars:N` | Truncate to N chars | `{{ text|truncatechars:100 }}` |
| `|truncatewords:N` | Truncate to N words | `{{ text|truncatewords:20 }}` |
| `|lower` / `|upper` | Case conversion | `{{ name|lower }}` |
| `|slugify` | Convert to URL slug | `{{ title|slugify }}` |
| `|linebreaks` | Newlines to <p> and <br> | `{{ desc|linebreaks }}` |
| `|join:` | Join list items | `{{ tags|join:", " }}` |
| `|length` | Length of list/string | `{{ items|length }}` |
| `|default:` | Default if value is falsy | `{{ name|default:"Anonymous" }}` |
| `|floatformat:N` | Round to N decimal places | `{{ price|floatformat:2 }}` |
| `|add:` | Add (concatenate or math) | `{{ count|add:"1" }}` |

> **`|safe` is an XSS vector if the content comes from users.** Only use it on content you fully control (e.g., admin-entered HTML).

## Forms

    # myapp/forms.py
    from django import forms
    from .models import Product


    # ---- ModelForm: auto-generates fields from a model ----
    class ProductForm(forms.ModelForm):

        class Meta:
            model = Product
            fields = ['name', 'category', 'price', 'description', 'tags', 'image']
            # exclude = ['created_at']  # alternative: list what to exclude
            widgets = {
                'description': forms.Textarea(attrs={'rows': 4}),
                'tags': forms.CheckboxSelectMultiple,  # instead of default multi-select
            }
            labels = {
                'name': 'Product Name',
            }
            help_texts = {
                'price': 'Price in USD',
            }
            error_messages = {
                'name': {
                    'required': 'Product name is required.',
                },
            }

        # Custom validation: clean_<fieldname>()
        def clean_price(self):
            price = self.cleaned_data.get('price')
            if price is not None and price < 0:
                raise forms.ValidationError('Price must be positive.')
            return price

        # Cross-field validation: clean()
        def clean(self):
            cleaned = super().clean()
            category = cleaned.get('category')
            price = cleaned.get('price')
            if category and category.name == 'Premium' and price and price < 50:
                raise forms.ValidationError('Premium products must cost at least $50.')
            return cleaned


    # ---- Plain Form (not tied to a model) ----
    class ContactForm(forms.Form):
        name = forms.CharField(max_length=100)
        email = forms.EmailField()
        message = forms.CharField(widget=forms.Textarea)
        # Add a honeypot field for spam prevention:
        # website = forms.CharField(required=False, widget=forms.HiddenInput)

        def clean_email(self):
            email = self.cleaned_data['email']
            if User.objects.filter(email=email).exists():
                raise forms.ValidationError('This email is already registered.')
            return email


    # ---- Form template ----
    # <form method="post" enctype="multipart/form-data">  enctype for FileField/ImageField
    # {% csrf_token %}
    # {{ form.as_p }}  or render fields individually:
    # {% for field in form %}
    #     <div>
    #         {{ field.label_tag }}
    #         {{ field.errors }}  <!-- shows validation errors -->
    #         {{ field }}  <!-- or {{ field.as_widget }} for no label -->
    #         {% if field.help_text %}<small>{{ field.help_text }}</small>{% endif %}
    #     </div>
    # {% endfor %}
    # <button type="submit">Save</button>
    # </form>

## Admin (admin.py)

    # myapp/admin.py
    from django.contrib import admin
    from .models import Product, Category


    # ---- Basic registration ----
    # admin.site.register(Product)  # Minimal, no customization

    # ---- ModelAdmin customization ----
    @admin.register(Product)
    class ProductAdmin(admin.ModelAdmin):
        list_display = ['name', 'category', 'price', 'status', 'created_at']
        list_display_links = ['name']  # Clickable to edit (default: first item)
        list_filter = ['status', 'category', 'created_at']
        search_fields = ['name', 'sku']  # Searches these fields
        ordering = ['-created_at']

        # Filter on related field (uses double underscore)
        list_filter = ['status', 'category__name']

        # Editable fields directly in list view
        list_editable = ['price', 'status']

        # Actions dropdown
        actions = ['mark_as_published']

        def mark_as_published(self, request, queryset):
            updated = queryset.update(status='published')
            self.message_user(request, f"{updated} products published.")
        mark_as_published.short_description = "Mark selected as published"

        # Custom column with HTML
        def price_with_currency(self, obj):
            return f"${obj.price}"
        price_with_currency.short_description = "Price"
        price_with_currency.admin_order_field = 'price'  # Make column sortable

        # Read-only fields in edit form
        readonly_fields = ['created_at', 'updated_at']

        # Fieldsets: organize edit form into sections
        fieldsets = [
            ('Basic Info', {
                'fields': ['name', 'sku', 'category', 'price'],
            }),
            ('Details', {
                'fields': ['description', 'tags', 'image'],
                'classes': ['collapse'],  # Collapsible section
            }),
            ('Status', {
                'fields': ['status', 'created_at', 'updated_at'],
                'classes': ['collapse'],
            }),
        ]

        # Auto-populate slug from name
        prepopulated_fields = {'slug': ('name',)}

        # Date hierarchy navigation
        date_hierarchy = 'created_at'

        # Pagination
        list_per_page = 50

        # FK shown as a search widget instead of dropdown (for large tables)
        autocomplete_fields = ['category']
        # Requires search_fields defined on the related model's admin too.

        # Inline admin for related models
        inlines = [ProductImageInline]  # See below


    # ---- Inline admin ----
    @admin.register(ProductImage)
    class ProductImageInline(admin.TabularInline):  # or admin.StackedInline
        model = ProductImage
        extra = 1  # Number of empty forms to show
        readonly_fields = ['image_preview']

        def image_preview(self, obj):
            if obj.image:
                return f'<img src="{obj.image.url}" width="100">'
            return ""
        image_preview.allow_tags = True  # Django 4.x: use format_html instead


    # ---- Custom User Model (if using AUTH_USER_MODEL) ----
    # from django.contrib.auth.admin import UserAdmin
    # @admin.register(CustomUser)
    # class CustomUserAdmin(UserAdmin):
    #     list_display = ['email', 'first_name', 'is_staff']
    #     fieldsets = UserAdmin.fieldsets + (
    #         ('Extra', {'fields': ['phone', 'department']}),
    #     )

## Signals

    # myapp/signals.py
    from django.db.models.signals import post_save, pre_delete, m2m_changed
    from django.dispatch import receiver
    from django.core.mail import send_mail
    from .models import Product, Order


    # ---- post_save: after a model is saved ----
    @receiver(post_save, sender=Product)
    def product_created(sender, instance, created, **kwargs):
        if created:
            # Only runs on creation, not updates
            send_mail(
                'New Product Created',
                f'Product {instance.name} was created.',
                'noreply@example.com',
                ['admin@example.com'],
            )

    # ---- pre_delete: before a model is deleted ----
    @receiver(pre_delete, sender=Order)
    def order_about_to_delete(sender, instance, **kwargs):
        # Log or take action before deletion
        instance.log(f"Order {instance.id} deleted")

    # ---- m2m_changed: when a ManyToMany relation changes ----
    @receiver(m2m_changed, sender=Product.tags.through)
    def product_tags_changed(sender, instance, action, pk_set, **kwargs):
        if action == "post_add":
            # Tags were added
            pass
        elif action == "post_remove":
            # Tags were removed
            pass

    # myapp/apps.py (register signals)
    from django.apps import AppConfig


    class MyappConfig(AppConfig):
        default_auto_field = 'django.db.models.BigAutoField'
        name = 'myapp'

        def ready(self):
            # Import signals here so they get registered.
            # This avoids circular imports and ensures AppConfig is ready.
            import myapp.signals

> **Always connect signals in `AppConfig.ready()`.** Connecting at module level in `models.py` can cause issues with AppConfig not being ready, and the import may happen multiple times in testing.

## Middleware

    # myapp/middleware.py
    import time
    import logging

    logger = logging.getLogger(__name__)


    class TimingMiddleware:
        # Middleware can be either a function or a class.
        # Class-based: __init__ gets called once at server start.

        def __init__(self, get_response):
            self.get_response = get_response

        def __call__(self, request):
            # Before view runs
            start = time.time()

            # Call the view (or next middleware)
            response = self.get_response(request)

            # After view runs
            duration = time.time() - start
            logger.info(f"{request.method} {request.path} - {duration:.3f}s")

            # Add header to response
            response['X-Response-Time'] = f"{duration:.3f}"
            return response

    # In settings.py
    MIDDLEWARE = [
        # ... existing middleware ...
        'myapp.middleware.TimingMiddleware',
        # Order matters: first in list = first to process request,
        # last to process response.
    ]

> **Middleware order is critical.** Request flows top-to-bottom. Response flows bottom-to-top. Authentication middleware must come before anything that checks `request.user`.

## Static Files

    # Static file locations (checked in this order):
    # 1. Each app's static/ directory (if APP_DIRS or StaticFilesFinder is active)
    # 2. Directories listed in STATICFILES_DIRS (project-level static files)
    # 3. STATIC_ROOT (only used by collectstatic, not for development)

    # In templates:
    # {% load static %}
    # <link rel="stylesheet" href="{% static 'css/style.css' %}">
    # <img src="{% static 'img/logo.png' %}">

    # In development: Django's runserver serves static files automatically.
    # In production: run collectstatic, then serve STATIC_ROOT with nginx/whitenoise.

    # Whitenoise (simplest approach for small/medium projects):
    # pip install whitenoise
    # In settings.py MIDDLEWARE (after SecurityMiddleware, before CommonMiddleware):
    # 'whitenoise.middleware.WhiteNoiseMiddleware',
    # In settings.py:
    # STATICFILES_STORAGE = 'whitenoise.storage.CompressedManifestStaticFilesStorage'

    # Collectstatic command:
    # python manage.py collectstatic  # copies all static files to STATIC_ROOT

## Commonly Missed Things

1. **Forgetting `request.FILES` in form view.** `ModelForm(request.POST)` won't capture file uploads. Must be `ModelForm(request.POST, request.FILES)`. Also need `enctype="multipart/form-data"` on the HTML form.

2. **Forgetting `{% csrf_token %}` in POST forms.** Django blocks the request with a 403 Forbidden. Always include it inside every `<form>` tag with `method="post"`.

3. **Not using `select_related()` for ForeignKey lookups.** Every `product.category.name` in a loop causes a separate query (N+1 problem). Use `select_related('category')`.

4. **Not using `prefetch_related()` for ManyToMany/reverse FK.** Same N+1 problem. Use `prefetch_related('tags')` for M2M fields.

5. **Using `null=True` on CharField/TextField.** Use `blank=True` instead. Empty string `""` is the Django convention for "no data" on text fields. NULL means "unknown" and creates bugs in filters (`name=""` vs `name__isnull=True`).

6. **Not setting `on_delete` on ForeignKey.** Django defaults to `CASCADE` in most cases, but being explicit prevents surprises. Always state it.

7. **Forgetting `related_name` on ForeignKey/ManyToMany.** Without it, Django uses `modelname_set`, which is ugly and ambiguous if multiple FKs point to the same model.

8. **Querying in a loop.** `for item in items: item.category.name` is N+1. Always prefetch or select related first.

9. **Not migrating after model changes.** Changed a field, added a field, renamed a field? Run `makemigrations` then `migrate`. Skipping this causes runtime errors.

10. **Using `DateTimeField` without `USE_TZ=True`.** Timezone-naive datetimes cause subtle bugs. Always use timezone-aware datetimes in Django.

11. **Serving media files in production.** `runserver` serves media files, but production servers (gunicorn/uwsgi) don't. Configure nginx or use a storage backend.

12. **Not using `get_object_or_404()`.** `Model.objects.get()` raises `DoesNotExist` which becomes a 500 error. `get_object_or_404()` returns a clean 404.

13. **Putting business logic in views instead of models.** Views should handle HTTP concerns (request/response). Business logic belongs in model methods or a service layer.

14. **Not handling `bulk_create`/`bulk_update` signal limitation.** These don't fire `save()` signals or call `save()` methods. If you need signals, use a loop or `post_save` manually after.

15. **Setting `DEBUG=True` in production.** Exposes stack traces, settings, and SQL queries to the world. Always use environment variables.

16. **Not using `reverse_lazy()` in class-level attributes.** `reverse()` fails at module load time because URL conf isn't ready yet. `reverse_lazy()` defers the lookup.

## Best Practices

- **Custom User Model:** Create one at the start of every project. Adding one later requires complex migrations. Use `AbstractUser` (keep standard auth fields) or `AbstractBaseUser` (full control).

- **One app per concern:** Keep apps focused. `orders`, `products`, `users` — not `everything`.

- **Use `select_related` and `prefetch_related` everywhere.** Profile with `django.db.connection.queries` or Django Debug Toolbar to find N+1 queries.

- **Use `values()` or `values_list()` when you only need specific fields.** No need to instantiate full model objects for a dropdown.

- **Never hardcode URLs.** Always use `reverse()` in Python and `{% url %}` in templates.

- **Use `F()` expressions for atomic updates.** `Product.objects.filter(pk=1).update(price=F('price') + 10)` avoids race conditions.

- **Use `bulk_create` for inserting many objects.** 10-100x faster than individual `save()` calls.

- **Use `transaction.atomic()` for multi-step operations.** If anything fails, the whole transaction rolls back.

- **Use `django-axes` or `django-ratelimit` for brute-force protection.** Don't roll your own.

- **Use `django-filter` with DRF for complex query filtering.** Don't build filter logic from scratch.

- **Validate at the form/model level, not just the template.** Client-side validation is UX only. Server-side validation is security.

- **Use environment variables for all sensitive config.** Database password, secret key, API keys, email credentials.

- **Keep migrations in version control.** They are part of your code history. Never delete applied migrations.

- **Use `django-extensions` `shell_plus`** for a better shell that auto-imports your models.

- **Use `logger` instead of `print()`.** `logging.getLogger(__name__)` gives you level control and output routing.

## DevOps: CLI, PostgreSQL, Backups

### Starting a Project

    django-admin startproject myproject
    cd myproject
    python manage.py startapp myapp

    # Or with a custom template:
    django-admin startproject --template https://github.com/... myproject

### Common CLI Commands

| Command | What It Does |
|---------|---------------|
| `manage.py runserver` | Development server (auto-reloads on file changes) |
| `manage.py runserver 0.0.0.0:8000` | Listen on all interfaces (for LAN testing) |
| `manage.py shell` | Interactive Python shell |
| `manage.py shell_plus` | Enhanced shell (requires django-extensions) |
| `manage.py dbshell` | Direct psql shell connected to the DB |
| `manage.py createsuperuser` | Create admin user interactively |
| `manage.py changepassword <user>` | Change a user's password |
| `manage.py makemigrations` | Create migration files from model changes |
| `manage.py migrate` | Apply pending migrations |
| `manage.py showmigrations` | Show migration status |
| `manage.py collectstatic` | Copy static files to STATIC_ROOT |
| `manage.py test` | Run all tests |
| `manage.py test myapp` | Run tests for one app |
| `manage.py test myapp.tests.TestClass` | Run one test class |
| `manage.py test --keepdb` | Don't destroy/recreate test DB (faster) |
| `manage.py check` | Run system checks (no DB access needed) |
| `manage.py check --deploy` | Production deployment checks |
| `manage.py show_urls` | List all URLs (requires django-extensions) |
| `manage.py dumpdata myapp` | Serialize app data to JSON |
| `manage.py loaddata backup.json` | Load data from a fixture |
| `manage.py flush` | Reset the DB (drops all data, keeps tables) |
| `manage.py reset_db` | Drops and recreates the entire DB (requires django-extensions) |

### Django Shell

    python manage.py shell

    # Or with shell_plus (auto-imports models):
    python manage.py shell_plus

    # Inside the shell:
    >>> from myapp.models import Product
    >>> Product.objects.count()
    42
    >>> p = Product.objects.first()
    >>> p.name, p.price
    ('Widget', Decimal('29.99'))
    >>> Product.objects.filter(price__gt=50).count()
    10
    >>> Product.objects.create(name='New Product', price=19.99)
    <Product: New Product>
    >>> from django.db import connection
    >>> connection.queries[-1]['sql']
    # Shows the last SQL query executed

### Working with PostgreSQL

    psql -U myuser -d mydb -h localhost

    # List all tables
    \dt

    # Describe a table
    \d myapp_product

    # Count records
    SELECT count(*) FROM myapp_product;

    # Find large tables
    SELECT relname AS table_name, pg_size_pretty(pg_total_relation_size(relid)) AS size
    FROM pg_stat_user_tables ORDER BY pg_total_relation_size(relid) DESC LIMIT 20;

    # Index usage stats (find unused indexes)
    SELECT schemaname, relname, indexname, idx_scan
    FROM pg_stat_user_indexes WHERE idx_scan = 0 ORDER BY pg_relation_size(indexrelid) DESC;

    # Kill connections before dropping DB
    SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'mydb';

    # Reset a sequence (if auto-increment gets out of sync)
    SELECT setval('myapp_product_id_seq', (SELECT max(id) FROM myapp_product));

> **Don't modify Django tables via raw SQL** unless you know exactly what you're doing. The ORM manages constraints, triggers, and caches that raw SQL bypasses.

### Database Backup and Restore

    # Backup
    pg_dump -U myuser -d mydb -F c -f mydb_backup.dump

    # Restore (create fresh DB first)
    createdb -U myuser mydb_restored
    pg_restore -U myuser -d mydb_restored -j 4 --no-owner mydb_backup.dump

    # Quick SQL backup (smaller file, slower restore)
    pg_dump -U myuser -d mydb -f mydb_backup.sql

    # Also back up media files (user uploads):
    tar -czf media_backup.tar.gz /path/to/media/

> **Media files are NOT in the database.** They live in `MEDIA_ROOT` on the filesystem. Back them up separately. Consider using `django-storages` with S3/Blob storage to avoid filesystem management entirely.

### Debug Mode

| Feature | How |
|---------|-----|
| Error pages with stack traces | `DEBUG=True` (auto) |
| All SQL queries logged | `DEBUG=True` + check `django.db.connection.queries` |
| Django Debug Toolbar | `pip install django-debug-toolbar`, add to INSTALLED_APPS and MIDDLEWARE |
| Silk (performance profiling) | `pip install django-silk`, profiles every request |
| Production error pages | Configure `DEBUG=False` + `ALLOWED_HOSTS` + logging |
| 500 email on error | Set `ADMINS = [('Name', 'email')]` in settings |

### Logging

    # In settings.py
    LOGGING = {
        'version': 1,
        'disable_existing_loggers': False,
        'formatters': {
            'verbose': {
                'format': '[{levelname}] {asctime} {module} {process:d} {thread:d} {message}',
                'style': '{',
            },
            'simple': {
                'format': '{levelname} {message}',
            },
        },
        'handlers': {
            'console': {
                'class': 'logging.StreamHandler',
                'formatter': 'simple',
            },
            'file': {
                'class': 'logging.handlers.RotatingFileHandler',
                'filename': BASE_DIR / 'logs' / 'django.log',
                'maxBytes': 10 * 1024 * 1024,  # 10 MB
                'backupCount': 5,
                'formatter': 'verbose',
            },
        },
        'loggers': {
            'django': {
                'handlers': ['console'],
                'level': 'INFO',
            },
            'myapp': {
                'handlers': ['console', 'file'],
                'level': 'DEBUG',  # Debug your app, info for Django
                'propagate': False,  # Don't bubble up to django logger
            },
            'django.requests': {
                'handlers': ['file'],
                'level': 'WARNING',  # Log 4xx/5xx to file
                'propagate': False,
            },
        },
    }

    # In Python code
    import logging
    logger = logging.getLogger(__name__)

    logger.debug("Variable value: %s", value)
    logger.info("Operation completed")
    logger.warning("Something unexpected")
    logger.error("Failed: %s", error)
    logger.critical("System failure")

### Development Workflow Tips

- **Create a custom User model at project start.** Adding one later is painful. Use `AbstractUser` to extend the standard user with custom fields.

- **Use `django-extensions`.** Adds `shell_plus`, `show_urls`, `reset_db`, `runserver_plus` (with Werkzeug debugger), and more.

- **Use `Django Debug Toolbar` during development.** Shows all queries, timing, templates used, signals fired, and more for every request.

- **Use `--keepdb` for tests.** Avoids recreating the test database on every run. Much faster for iterative testing.

- **Use `manage.py check --deploy` before deploying.** Catches common production misconfigurations (missing ALLOWED_HOSTS, DEBUG=True, etc.).

- **Use `pip freeze > requirements.txt` sparingly.** Better: use `pip-tools` with separate `requirements.in` (specifiers) and `requirements.txt` (pinned).

- **Use `.env` files for local config.** Never commit secrets. Commit `.env.example` with empty values.

- **Don't commit `staticfiles/` or `media/`.** Add them to `.gitignore`. These are generated / user-uploaded content.

- **Use `django-storages` with S3 for media files in production.** Simplifies deployment and scaling. No need to manage filesystem storage on servers.

- **Use `gunicorn` for production.** `runserver` is single-threaded and not suitable for production. Gunicorn handles concurrent requests.