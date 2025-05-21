# 🧠 Mastermind Software Engineer Guide (Personal Edition)

## 🚀 My Tech Stack

- **Backend**:
  - Python (Django, FastAPI)
  - Rust (actix-web, axum)
- **Frontend**:
  - Svelte / SvelteKit
- **Tools**:
  - Linux (daily driver)
  - Git (version control)
  - Docker (containers & services)
- **Patterns & Principles**:
  - Know SOLID, DRY, YAGNI, KISS intuitively
  - Want mastery at naming, architecture, extendibility

---

## 🟢 Foundational Knowledge (Must Always Keep Warm)

### ✅ Data Structures & Algorithms (DSA)
- Arrays, Linked Lists, Trees, Graphs
- Sorting, Searching
- Hash Maps, Sets
- Recursion, Dynamic Programming (DP)
- Time & Space Complexity (Big O)

**Tip**: Revise once per year — but don’t overstress memorization. Deep pattern recognition is more important.

### ✅ Core Principles
- **SOLID** (OOP principles)
- **DRY** (Don’t Repeat Yourself)
- **KISS** (Keep It Simple, Stupid)
- **YAGNI** (You Ain’t Gonna Need It)

---

## 🟠 Patterns You Already Use (Mapped To Official Names)

| Your Intuition                           | Official Name  |
|-------------------------------------------|----------------|
| "Pick the right component/class at runtime"  | **Factory Pattern** |
| "Wrap something external to fit my API"   | **Adapter Pattern** |
| "Add features without changing original"  | **Decorator Pattern** |
| "Assemble stuff step by step"             | **Builder Pattern** |
| "Split big components into smaller ones"  | **Single Responsibility Principle (SRP)** |
| "Centralize state after 2+ components need it" | **Mediator Pattern (Store)** |
| "Wrap API calls inside utils"             | **Service Layer / Adapter** |
| "Extract repeated code to shared utils"   | **DRY Principle** |
| "Consistent variable naming"              | **Clean Code Naming & Style Guide** |

---

## 🟢 Real-World Pattern Examples (Svelte / Python / Rust Logic)

### 🏭 Factory Pattern
- **Svelte**: A `File` component that picks `ImageFile`, `PdfFile`, `TextFile` based on `file.type`.
- **Python**: A function that returns different serializer classes based on the model.
- **Rust**: Match on enum variant and return different structs.

### 🔌 Adapter Pattern
- **Svelte**: Wrap Google Maps JS API into a reactive Svelte component.
- **Python**: Convert a raw SQL API response into Django model objects.
- **Rust**: Implement `From`/`Into` traits to convert between types.

### 🎨 Decorator Pattern
- **Svelte**: Wrap a button to add animations or tooltips.
- **Python**: Use `@login_required`, `@retry` decorators.
- **Rust**: Wrap functions with middleware layers.

### 🏗️ Builder Pattern
- **Svelte**: Create a dynamic `FormBuilder` component that `.addField()`s based on config.
- **Python**: Build complex SQL queries via `.filter().order_by()` chaining.
- **Rust**: Use builder structs to construct HTTP requests.

### 🟡 Mediator (State Store)
- **Svelte**: Move shared state from components into a Svelte `store`.
- **Python**: Use a Pub/Sub system (e.g., Signals in Django).
- **Rust**: Use a channel or shared state wrapped in `Arc<Mutex<_>>`.

---

## 🔥 Builder Pattern vs Functional Chaining (Explained)

| Builder Pattern           | Functional Chaining (FP)         |
|---------------------------|----------------------------------|
| OOP pattern                | FP technique                     |
| Has **internal mutable state** during building | Pure functions, no mutation |
| Ends with `.build()` → final object  | Every step returns new data |
| Used to construct **objects** | Used to transform **data** |

### 🔹 Svelte & Python Mental Rule
- When you **build** → use Builder pattern (forms, objects, configs).
- When you **transform** → use Functional chaining (arrays, data pipelines).

---

## 🔥 Next-Level Topics To Master (Architect Mindset)

### 1️⃣ Architectural Styles
- **Layered Architecture** (Controllers → Services → Repos)
- **Hexagonal Architecture** (Ports & Adapters, decouple domain)
- **CQRS** (Command Query Responsibility Segregation)
- **Event-Driven Architecture** (pub/sub systems, message queues)

### 2️⃣ Domain-Driven Design (DDD)
- ✅ Entities → objects with identity (e.g., `User`, `Order`)
- ✅ Value Objects → immutable, no ID (`Money`, `Coordinates`)
- ✅ Aggregates → groups of entities with a root
- ✅ Ubiquitous Language → name code after domain terms

### 3️⃣ Advanced State Management (Frontend)
- ✅ **Finite State Machines** → use XState or implement manually
- ✅ **Statecharts** → for complex UIs (multi-step flows)
- ✅ **Reactive Patterns** → data flow graphs, not just stores

### 4️⃣ Testing Patterns
- ✅ **Given-When-Then** (BDD style)
- ✅ **Test Doubles** (Mock, Stub, Fake)
- ✅ **Property-Based Testing** (e.g., `hypothesis` in Python, `proptest` in Rust)




---

## 🟢 Side Tools Every Mastermind Should Command

### ✅ Git (Version Control)
- Rebase, squash commits
- Feature branch workflows
- Writing meaningful commit messages
- Handling merge conflicts like a pro

### ✅ Docker (Containers)
- Writing multi-stage Dockerfiles
- Using `docker-compose` for dev envs
- Building minimal, production-ready images
- Understanding networks, volumes, healthchecks

### ✅ Linux (Dev Environment)
- Bash scripting
- `grep`, `awk`, `sed` for text processing
- Managing services with `systemd`
- Networking basics (`netstat`, `curl`, `dig`)

### ✅ CI/CD (Optional but Highly Recommended)
- Github Actions / Gitlab CI pipelines
- Lint → Test → Build → Deploy sequences
- Using Docker in CI pipelines

---

## 📚 Recommended Books & Videos (Curated)

### 📘 Books:
- **Clean Code** by Robert C. Martin (the Bible of code readability)
- **Clean Architecture** by Robert C. Martin
- **Domain-Driven Design Distilled** by Vaughn Vernon (short & actionable)
- **Patterns of Enterprise Application Architecture** by Martin Fowler
- **Refactoring UI** by Adam Wathan (for frontend devs, Svelte too)

### 📺 Videos:
- [**Architecting Frontend Applications**](https://www.youtube.com/watch?v=E8I19uA-wGY) *(Best explanation of frontend patterns)*
- [**Hexagonal Architecture Explained**](https://www.youtube.com/watch?v=th4AgBcrEHA)
- [**Clean Code vs Dirty Code** (Fun Talk)](https://www.youtube.com/watch?v=HzWf-EeE3LE)
- [**Event-Driven Architecture Crash Course**](https://www.youtube.com/watch?v=STKCRSUsyP0)

---

## 🟣 My Personalized Pattern Map (Cheat Sheet)

| When I do this...                             | It's called...         |
|-----------------------------------------------|------------------------|
| Pick components/classes based on condition    | **Factory Pattern**    |
| Wrap external APIs/components to fit my code  | **Adapter Pattern**    |
| Add features without changing base logic      | **Decorator Pattern**  |
| Build objects/config step by step             | **Builder Pattern**    |
| Split big things into smaller units           | **Single Responsibility** |
| Centralize shared state across many parts     | **Mediator / Store Pattern** |
| Extract repeated logic                        | **DRY Principle**      |
| Follow naming conventions                     | **Clean Code Naming**  |

---

## ✅ How To Become Confident When Choosing Architecture

1. **Small Project?**
 - Use **Layered Architecture**.
 - YAGNI: Don’t over-engineer.

2. **Medium to Large?**
 - Use **Hexagonal** + **DDD Lite**.
 - Aim for framework-independent domain logic.

3. **Frontend Complex State?**
 - Model as **State Machines**.
 - Use stores only when 2+ components need shared state.

4. **Backend Scaling?**
 - Move to **Event-Driven** or **CQRS** if reads/writes differ in volume.

5. **Always**
 - Start simple (Layered), then evolve to advanced when complexity forces you.
 - Record decisions in **ADR** docs.

---

## 🔥 Final Tip: 
> Mastermind devs don’t memorize patterns — they **name their instincts** and **record decisions**.  
> Trust your intuition. Level it up by learning the names.  
> Then, you'll not only code well — you'll design systems that last 🔥.

---


# 🌐 Clean API Development Guide (General & REST Focus)

_By: For Mastermind Devs who want to craft solid APIs_

---

## 🚀 Goals of Clean API Design
- ✅ **Consistent** → predictable patterns everywhere
- ✅ **Simple** → easy for clients to understand & use
- ✅ **Versioned** → changes don't break old clients
- ✅ **Secure** → protects against common vulnerabilities
- ✅ **Extensible** → easy to add features without breaking old ones
- ✅ **Well-documented** → clear and accurate API docs

---


## 🔥 General API Design Principles (Always Apply)

### ✅ 1. Use Meaningful Resource Naming
- Use **nouns** for resources, **verbs** for actions.
- Examples:
        ```
        GET /users
        POST /users
        PATCH /users/{id}
        ```


### ✅ 2. Use HTTP Methods Correctly
| Method  | Use For                   |
|---------|---------------------------|
| GET     | Retrieve data             |
| POST    | Create new resource       |
| PUT     | Replace entire resource   |
| PATCH   | Update part of resource   |
| DELETE  | Remove resource           |

### ✅ 3. Version Your API (Always!)
- Example:
    ```
    /api/v1/users
    /api/v2/users
    ```

- Use URI versioning or headers, but **always version**.

### ✅ 4. Use Standard HTTP Status Codes
| Status Code  | Meaning              |
|--------------|----------------------|
| 200 OK       | Success               |
| 201 Created  | Resource created      |
| 400 Bad Request | Client error       |
| 401 Unauthorized | Auth failed       |
| 404 Not Found | Resource missing     |
| 500 Server Error | Internal fail     |

### ✅ 5. Keep Responses Consistent
- Use a common envelope format:
```json
{
  "data": { ... },
  "error": null
}

```
https://swagger.io/



# videos + resources : 

## Python
- [James Powell playlist](https://www.youtube.com/watch?v=cKPlPJyQrt4&list=PLzg3FkRs7fcTjdBdrP6dOTcV3AJwnzL0Y)
- [sentdex web scraping](https://www.youtube.com/watch?v=aIPqt-OdmS0&list=PLQVvvaa0QuDfV1MIRBOcqClP6VZXsvyZS)

### pandas
- [Pandas advanced](https://www.youtube.com/watch?v=ZyhVh-qRZPA&list=PL-osiE80TeTsWmV9i9c58mdDCSskIFdDS)
- [Pandas tricks](https://www.youtube.com/watch?v=RlIiVeig3hc)
- [more pandas tricks](https://www.youtube.com/watch?v=tWFQqaRtSQA)
- [sentdex pandas](https://www.youtube.com/watch?v=Iqjy9UqKKuo&list=PLQVvvaa0QuDc-3szzjeP6N6b0aDrrKyL-)



### django
- [Full django](https://www.youtube.com/watch?v=PtQiiknWUcI)
- [Full django (drf)](https://www.youtube.com/watch?v=c708Nf0cHrs)





## IT (networking / databases / http ...)
long run
- [Best all in one channel i found](https://www.youtube.com/@hnasr/playlists)
- [regex](https://www.youtube.com/watch?v=sa-TUpSx1JA)





## backend / system design
long run
- [bytebytego](https://www.youtube.com/@ByteByteGo/playlists) 
- [OPEN API SPECS](https://www.youtube.com/watch?v=6kwmW_p_Tig)
- [APiarchitecture](https://www.youtube.com/@CodeOpinion/featured)
- [API desing](https://www.youtube.com/watch?v=9Ng00IlBCtw&list=PL9XzOCngAkqs4m0XdULJu_78nM3Ok3Q65)
- [web design standards](https://www.youtube.com/watch?v=uS9wnNsamzA)


## rust
- [All in one video](https://www.youtube.com/watch?v=ygL_xcavzQ4)
- [from tust book](https://www.youtube.com/watch?v=OX9HJsJUDxA&list=PLai5B987bZ9CoVR-QEIN9foz4QCJ0H2Y8)
- [From rust to python](https://www.youtube.com/watch?v=7odJDwhjCXQ&list=PLEIv4NBmh-GsWGE9mY3sF9c5lgh5Z_jLr)
- also check the <b>rustlings & 100 exercices</b>




## git
- [All in one video](https://www.youtube.com/watch?v=RGOj5yH7evk)



## svelte
- [All in one sveltekit](https://www.youtube.com/watch?v=MoGkX4RvZ38)
- [tailwind css](https://www.youtube.com/watch?v=bxmDnn7lrnk&list=PL4cUxeGkcC9gpXORlEHjc5bgnIi5HEGhw)
- also check daisyUI and <b>skeletonUI</b>





## linux
- [Mostafa Hamouda redhat (the best)](https://www.youtube.com/watch?v=oD5Y4Gzr6vw&list=PLy1Fx2HfcmWBpD_PI4AQpjeDK5-5q6TG7)
- also check docker



## algorithms
- [Prime's last algo](https://frontendmasters.com/courses/algorithms/introduction/)
- [Algorithms simple version](https://www.youtube.com/watch?v=kp3fCihUXEg)

- [Datastructures simple version](https://www.youtube.com/watch?v=cQWr9DFE1ww)
- also do some leetcode, best from neetcode


## AI : 
- we don't do that here


# terminal
 - ranger
 - helix
 - tmux
 - docker / docker-compose /lazydocker
 - git / lazygit
 - htop
 - coreutils (grep / find / jq /sed ...)
 - ansible
 - fish / starship
 - ollama

# Saas
 - Wordpress (cf7 elementor)
 - Odoo
 - directus
 - n8n
 - aapanel
 - glpi
 -- moodle / budibase / callcom

# dev
 - python (django,pandas / polars,fasapi,requests,scrapy,sklearn) jupyter
 - js (sveltekit / skeletonUi superform,drizzleORM,pwa,capacitor)
 - fun with go / rust
 - sentry

# desktop
 - insomnia
 - vscodium
 - chromium
 - dbeaver
 - inkscape
 - onlyoffice
 - flameshot
 - lmstudio

# Data
 - dbt
 - dagster
 - airflow/prefect
 - superset
 - grafana
 - pg / mongo / sqlite / clocckhouse
 - great_expectations
 - querybook

# devops
- pormetius / grafana / uptime kuma
- gitlab
- nginx / traefik / caddy
- cockpit 
- openstack
- portainer

# cool stuff
- typst
- automa
- evilimiter
- airgeddon
- PlantUML
- fedora / alpine / k3s / microk8s / k0s
- bookmarklets



---
---

# Python

## <span style="color :#946969">Basics : 
code examples : https://www.programcreek.com/python/
- OOP (classes)
- generators (yield)
- Decorators (@something)
- Type hint
- List comprehention
- Lambda functions


## <span style="color :#93a315">Base Libraries : 

- inspect
- dis
- toolz
- functools
- random/secret
- operator
- datetime/time
- os/sys


## <span style="color :#960b7d">Web : 
- Django / DRF / Celery
- flask (quart)/ fastapi / django / muffin / blacksheep /aiohttp
- requests
- lxml
- json
- bs4
- sockets
- urlib3
- jwt


## <span style="color: #405787;">Usefull libs :</span>

- tabulate
- sqlalchemy
- tinydb
- scrapy
- pyautogui
- opencv2
- pynput
- pyinstaller
- pillow
- networkx
- jwt
- sqlacodegen generating auto models from a database
- furl manip url



## <span style="color: #bf4c0d;">Functional programming :

- map
- reduce
- filter
- toolz
- functools
- lambda
https://www.youtube.com/playlist?list=PLP8GkvaIxJP1z5bu4NX_bFrEInBkAgTMr


## <span style="color: #d4ac28;">Data science / analytics :</span>

- numpy
- pandas
- matplotlib
- plotly
- scipy
- sklearn
- pytorch
- pands-profiling   https://github.com/ydataai/pandas-profiling


## <span style="color :#449174">CLI : 

- fire






