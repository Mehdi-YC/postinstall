+++
title = "Python — The Full Picture"
description = "Basics, Pythonic patterns, OOP, async, type hints, data, tooling, and Rust extensions"
[extra]
date = 2024-01-15
+++

## Basics

### Variables, types, and truthiness

Python is dynamically typed — variables hold references, not values.
```python,linenos
    # Primitives
    name  : str   = "alice"
    age   : int   = 30
    score : float = 9.5
    active: bool  = True
    nothing         = None

    # Multiple assignment
    x = y = z = 0
    a, b, c = 1, 2, 3
    first, *rest = [1, 2, 3, 4]   # first=1, rest=[2,3,4]

    # Falsy values — memorise these
    # False, None, 0, 0.0, "", [], {}, set(), ()
    if not []:    print("empty list is falsy")
    if not "":   print("empty string is falsy")

    # type() vs isinstance()
    isinstance(3, (int, float))    # True — prefer this
    type(3) is int                  # True — exact type only
```
### Strings
```python,linenos
    s = "hello world"
    s.upper()           # "HELLO WORLD"
    s.split(" ")         # ["hello", "world"]
    s.strip()            # remove leading/trailing whitespace
    s.replace("l","L")  # "heLLo worLd"
    s.startswith("he") # True
    s.count("l")         # 3
    ", ".join(["a","b","c"])  # "a, b, c"

    # f-strings (use these always)
    name = "Alice"
    print(f"Hello, {name}!")
    print(f"Pi ≈ {3.14159:.2f}")  # "Pi ≈ 3.14"
    print(f"{score = }")           # "score = 9.5"  (debug format, 3.8+)

    # Multiline
    text = """
      Line one
      Line two
    """.strip()

    # Raw strings (no escape processing)
    pattern = r"\d+\.\d+"
```
### Control flow
```python,linenos
    # if / elif / else
    if score >= 90:      grade = "A"
    elif score >= 70:   grade = "B"
    else:               grade = "C"

    # One-liner ternary
    label = "pass" if score >= 50 else "fail"

    # match / case (Python 3.10+) — structural pattern matching
    match command:
        case "quit":             quit()
        case "hello":            print("Hi!")
        case {"action": action}: print(action)
        case [x, y]:             print(f"coords {x},{y}")
        case _:                  print("unknown")

    # for loops
    for i, val in enumerate(["a","b","c"]):
        print(i, val)   # 0 a, 1 b, 2 c

    for a, b in zip([1,2,3], ["x","y","z"]):
        print(a, b)     # 1 x, 2 y, 3 z

    # for...else (runs if loop didn't break)
    for item in items:
        if item.found: break
    else:
        print("nothing found")

    # while
    while queue:
        task = queue.pop(0)
        process(task)
```
### Exception handling
```python,linenos
    try:
        result = risky()
    except (ValueError, TypeError) as e:
        print(f"bad input: {e}")
    except Exception as e:
        logger.error(f"unexpected: {e}")
        raise               # re-raise
    else:
        save(result)         # runs only if no exception
    finally:
        cleanup()            # always runs

    # Custom exception
    class AppError(Exception):
        def __init__(self, message: str, code: int = 400):
            super().__init__(message)
            self.code = code

    raise AppError("not found", code=404)
```
## Collections

### Lists
```python,linenos
    nums = [3, 1, 4, 1, 5, 9]

    # Slicing: [start:stop:step]  — stop is exclusive
    nums[1:4]      # [1, 4, 1]
    nums[:-1]     # reversed copy
    nums[::2]      # every 2nd: [3, 4, 5]

    # Mutations
    nums.append(2)
    nums.extend([6, 5])
    nums.insert(0, 0)
    nums.remove(1)         # removes first occurrence
    popped = nums.pop()    # removes & returns last
    nums.sort(reverse=True)
    nums.sort(key=lambda x: -x)

    # Non-mutating
    sorted(nums)                  # returns new list
    sorted(people, key=lambda p: p.age)
    list(reversed(nums))          # returns iterator

    # Useful builtins on lists
    sum(nums); min(nums); max(nums); len(nums)
    any(x > 5 for x in nums)   # True if at least one
    all(x > 0 for x in nums)   # True if all
```
### Dictionaries
```python,linenos
    user = {"name": "Alice", "age": 30, "role": "admin"}

    # Access
    user["name"]              # "Alice" — raises KeyError if missing
    user.get("city", "N/A")   # "N/A" — safe default

    # Mutate
    user["email"] = "alice@x.com"
    user.update({"age": 31, "city": "Paris"})
    user.pop("role", None)    # remove, return None if missing

    # Merge dicts (3.9+)
    merged = base | overrides     # new dict
    base |= overrides              # in-place

    # Iterate
    for key in user:              # iterates keys
    for key, val in user.items():
    for val in user.values():

    # setdefault — initialise if key missing
    groups.setdefault("admin", []).append(user)

    # defaultdict    from collections import defaultdict
    groups = defaultdict(list)
    groups["admin"].append(user)   # no KeyError

    # Counter
    from collections import Counter
    c = Counter("banana")     # Counter({'a':3,'n':2,'b':1})
    c.most_common(2)          # [('a', 3), ('n', 2)]

    # OrderedDict (dicts are ordered since 3.7, but OrderedDict has .move_to_end)
    from collections import OrderedDict
```
### Sets and Tuples
```python,linenos
    # Set — unordered, unique, mutable
    tags = {"python", "web", "api"}
    tags.add("async")
    tags.discard("missing")   # no error if absent

    a = {1,2,3};  b = {2,3,4}
    a & b   # intersection: {2,3}
    a | b   # union:        {1,2,3,4}
    a - b   # difference:   {1}
    a ^ b   # symmetric diff:{1,4}
    a.issubset(b)   # False

    # Tuple — ordered, immutable
    point = (3, 4)
    x, y = point          # unpack
    single = (42,)        # note the trailing comma

    # namedtuple
    from collections import namedtuple
    Point = namedtuple("Point", ["x", "y"])
    p = Point(3, 4)
    print(p.x, p.y)   # 3 4
```
## Pythonic Patterns

### List, dict, and set comprehensions
```python,linenos
Comprehensions are faster than equivalent for-loops and far more readable when kept short.

    nums = [1, 2, 3, 4, 5, 6]

    # List comprehension:  [expression for item in iterable if condition]
    squares  = [x**2 for x in nums]
    evens_sq = [x**2 for x in nums if x % 2 == 0]
    flat     = [item for sub in matrix for item in sub]

    # Dict comprehension
    sq_map  = {x: x**2 for x in range(5)}         # {0:0,1:1,2:4,...}
    inverted = {v: k for k, v in original.items()}  # flip keys/values
    filtered = {k: v for k, v in d.items() if v is not None}

    # Set comprehension
    unique_domains = {email.split("@")[1] for email in emails}

    # Generator expression — lazy, no brackets
    total = sum(x**2 for x in range(1_000_000))   # no list in memory
```
### Walrus operator := (Python 3.8+)
```python,linenos
    # Assign and use in same expression
    if (n := len(data)) > 10:
        print(f"Too much data: {n} items")

    # Avoid calling the same function twice
    while chunk := file.read(8192):
        process(chunk)

    # Filter and transform in one pass
    results = [y for x in data if (y := process(x)) is not None]
```
### Unpacking
```python,linenos
    a, b = b, a                   # swap — no temp variable
    first, *middle, last = items  # star unpacking
    _, important, *_ = record     # ignore parts with _

    # Unpack into function args
    args   = [3, 14]
    kwargs = {"sep": ",", "end": "\n"}
    print(*args, **kwargs)

    # Dict merge via unpacking (< 3.9 compat)
    merged = {**base, **overrides}
```
### Lambdas

Lambdas are anonymous functions — use them for short, throwaway operations. For anything with logic, name it.
```python,linenos
    # lambda arguments: expression
    square = lambda x: x**2
    add    = lambda x, y: x + y

    # Common use: as sort key
    people.sort(key=lambda p: (p.age, p.name))
    sorted(words, key=lambda w: w.lower())

    # With map/filter (generator-based, lazy)
    doubled  = list(map(lambda x: x*2, nums))
    positive = list(filter(lambda x: x > 0, nums))
    # ↑ Prefer list comprehensions: [x*2 for x in nums]

    # With max/min/sorted key
    most_exp = max(products, key=lambda p: p.price)
```
### Context managers

```python,linenos
    # with statement — guarantees cleanup
    with open("file.txt", "r") as f:
        data = f.read()           # auto-closes even on exception

    # Multiple at once
    with open("in.txt") as src, open("out.txt", "w") as dst:
        dst.write(src.read())

    # Custom context manager via class
    class Timer:
        def __enter__(self):
            import time
            self.start = time.monotonic()
            return self
        def __exit__(self, *args):
            self.elapsed = time.monotonic() - self.start
            print(f"Elapsed: {self.elapsed:.3f}s")

    # Custom via @contextmanager (simpler)
    from contextlib import contextmanager

    @contextmanager
    def managed_resource(name):
        print(f"Opening {name}")
        try:
            yield name          # value bound to 'as' variable
        finally:
            print(f"Closing {name}")
```

## Functions in Depth

### Arguments

```python,linenos
    # All argument kinds
    def example(
        pos_only,        # positional only (before /)
        /,
        normal,          # positional or keyword
        *,               # everything after is keyword-only
        kw_only,
        default = 10,
    ):
        ...

    # *args and **kwargs
    def variadic(*args: int, **kwargs: str):
        # args = tuple, kwargs = dict
        for a in args:    print(a)
        for k, v in kwargs.items(): print(k, v)

    # Mutable defaults — the classic gotcha
    def bad(items=[]):           # ← shared across all calls!
        items.append(1)
        return items

    def good(items=None):        # ← correct pattern
        if items is None:
            items = []
        items.append(1)
        return items
```

### Closures and decorators

```python,linenos
    # Closure — inner function captures outer scope
    def make_counter(start=0):
        count = [start]                 # list to allow mutation
        def counter():
            count[0] += 1
            return count[0]
        return counter

    c = make_counter()
    c()   # 1
    c()   # 2

    # Decorator — a function that wraps another
    import functools

    def timing(func):
        @functools.wraps(func)          # preserves __name__, __doc__
        def wrapper(*args, **kwargs):
            import time
            t = time.perf_counter()
            result = func(*args, **kwargs)
            print(f"{func.__name__} took {time.perf_counter()-t:.4f}s")
            return result
        return wrapper

    @timing
    def slow_function():
        import time; time.sleep(0.1)

    # Decorator with arguments
    def retry(times=3, exceptions=(Exception,)):
        def decorator(func):
            @functools.wraps(func)
            def wrapper(*args, **kwargs):
                for attempt in range(times):
                    try:
                        return func(*args, **kwargs)
                    except exceptions as e:
                        if attempt == times - 1: raise
                        print(f"Retry {attempt+1}: {e}")
            return wrapper
        return decorator

    @retry(times=3, exceptions=(ConnectionError,))
    def fetch(url): ...
```

### Generators

```python,linenos
    # Generator function — yields values lazily
    def fibonacci():
        a, b = 0, 1
        while True:
            yield a
            a, b = b, a + b

    fib = fibonacci()
    next(fib)    # 0
    next(fib)    # 1
    next(fib)    # 1

    # Consume with for or itertools
    import itertools
    first10 = list(itertools.islice(fibonacci(), 10))

    # Generator for file processing (memory-efficient)
    def read_chunks(path, size=8192):
        with open(path, "rb") as f:
            while chunk := f.read(size):
                yield chunk

    # yield from — delegate to sub-generator
    def chain(*iterables):
        for it in iterables:
            yield from it
```

## Object-Oriented Programming

### Class anatomy

```python,linenos
    class Animal:
        species_count: int = 0    # class attribute

        def __init__(self, name: str, sound: str) -> None:
            self.name  = name       # instance attribute
            self._sound = sound     # convention: "private"
            Animal.species_count += 1

        def speak(self) -> str:
            return f"{self.name} says {self._sound}"

        @property
        def sound(self) -> str:
            return self._sound

        @sound.setter
        def sound(self, value: str):
            if not value:
                raise ValueError("sound can't be empty")
            self._sound = value

        @classmethod
        def from_dict(cls, data: dict) -> "Animal":
            return cls(data["name"], data["sound"])

        @staticmethod
        def validate_name(name: str) -> bool:
            return bool(name.strip())

        def __repr__(self) -> str:   # for developers
            return f"Animal({self.name!r})"

        def __str__(self) -> str:    # for users
            return self.name
```

### Dunder (magic) methods

```python,linenos
    class Vector:
        def __init__(self, x, y): self.x, self.y = x, y
        def __repr__(self):   return f"Vector({self.x}, {self.y})"
        def __add__(self, o):  return Vector(self.x+o.x, self.y+o.y)
        def __sub__(self, o):  return Vector(self.x-o.x, self.y-o.y)
        def __mul__(self, n):  return Vector(self.x*n, self.y*n)
        def __abs__(self):    return (self.x**2 + self.y**2)**0.5
        def __bool__(self):   return bool(abs(self))
        def __eq__(self, o):  return self.x==o.x and self.y==o.y
        def __hash__(self):   return hash((self.x, self.y))  # needed if __eq__
        def __len__(self):    return 2
        def __iter__(self):   yield self.x; yield self.y
        def __getitem__(self, i): return (self.x, self.y)[i]
        def __contains__(self, v): return v in (self.x, self.y)
```

### Inheritance and MRO

```python,linenos
    class Dog(Animal):
        def __init__(self, name: str, breed: str):
            super().__init__(name, "woof")   # call parent __init__
            self.breed = breed

        def speak(self) -> str:            # override
            base = super().speak()
            return f"{base}! ({self.breed})"

    # Multiple inheritance — MRO (Method Resolution Order)
    class A:
        def hello(self): print("A")

    class B(A):
        def hello(self): print("B"); super().hello()

    class C(A):
        def hello(self): print("C"); super().hello()

    class D(B, C): ...   # MRO: D → B → C → A
    D().hello()  # prints B C A

    # Abstract classes
    from abc import ABC, abstractmethod

    class Shape(ABC):
        @abstractmethod
        def area(self) -> float: ...

        @abstractmethod
        def perimeter(self) -> float: ...

    class Circle(Shape):
        def __init__(self, r): self.r = r
        def area(self): return 3.14159 * self.r**2
        def perimeter(self): return 2 * 3.14159 * self.r
```

### Dataclasses

```python,linenos
    from dataclasses import dataclass, field, asdict, astuple

    @dataclass
    class User:
        name:  str
        email: str
        age:   int = 0
        tags:  list[str] = field(default_factory=list)

        def __post_init__(self):
            self.email = self.email.lower()

    # Auto-generates __init__, __repr__, __eq__
    u = User("Alice", "ALICE@EX.COM", age=30)
    asdict(u)   # {'name': 'Alice', 'email': 'alice@ex.com', ...}

    @dataclass(frozen=True)   # immutable — usable as dict key
    class Point:
        x: float; y: float

    @dataclass(order=True)    # adds __lt__, __le__, etc.
    class Task:
        priority: int
        name:     str
```

## Type Hints

### Basic annotations

```python,linenos
    # Variables
    name: str = "Alice"
    count: int = 0
    ratio: float = 0.5
    active: bool = True
    data: bytes = b""

    # Collections (use built-in types, Python 3.9+)
    names: list[str]
    mapping: dict[str, int]
    pair: tuple[int, str]
    unique: set[float]

    # Optional — None or a value
    from typing import Optional
    maybe: Optional[str] = None
    maybe: str | None = None    # 3.10+ union syntax

    # Union types
    result: int | str | None

    # Function signatures
    def greet(name: str, times: int = 1) -> str:
        return (f"Hello {name}!\n") * times

    def nothing() -> None: ...   # explicit None return
```

### Advanced types

```python,linenos
    from typing import (
        Any, Callable, Generator, Iterator,
        TypeVar, Generic, Literal, Final,
        TypedDict, Protocol, overload
    )

    # Callable
    handler: Callable[[str, int], bool]   # (str, int) -> bool
    callback: Callable[..., None]            # any args, returns None

    # TypeVar — generic programming
    T = TypeVar("T")
    K = TypeVar("K")
    V = TypeVar("V")

    def first(items: list[T]) -> T | None:
        return items[0] if items else None

    # Generic class
    class Stack(Generic[T]):
        def __init__(self): self._items: list[T] = []
        def push(self, item: T) -> None: self._items.append(item)
        def pop(self) -> T: return self._items.pop()

    # Literal — restrict to specific values
    Mode = Literal["read", "write", "append"]
    def open_file(path: str, mode: Mode) -> ...: ...

    # TypedDict — typed dict structure
    class UserDict(TypedDict):
        name: str
        email: str
        age: int

    # Protocol — structural subtyping ("duck typing" formalized)
    class Drawable(Protocol):
        def draw(self) -> None: ...

    # Any class with a draw() method satisfies Drawable — no inheritance needed
    def render(item: Drawable) -> None:
        item.draw()

    # Final — constant that cannot be reassigned
    MAX_SIZE: Final = 100
```

> **Run mypy for static checks:** `mypy --strict mymodule.py`. Type hints are not enforced at runtime — they're for tools and documentation only.

## Async Python

### The basics

Python's `asyncio` uses a single-threaded event loop. Use it for I/O-bound tasks (HTTP, DB, files). For CPU-bound work, use `ProcessPoolExecutor`.

```python,linenos
    import asyncio

    # Basic coroutine
    async def fetch(url: str) -> str:
        await asyncio.sleep(1)   # non-blocking wait
        return f"data from {url}"

    # Run it
    result = asyncio.run(fetch("http://example.com"))

    # Run concurrently with gather
    async def main():
        results = await asyncio.gather(
            fetch("http://a.com"),
            fetch("http://b.com"),
            fetch("http://c.com"),
        )   # all 3 run concurrently, total ~1s not 3s
        return results

    # Tasks — schedule and forget
    async def main():
        task = asyncio.create_task(fetch("http://a.com"))
        # do other work here...
        result = await task    # retrieve when ready
```

### HTTP with httpx

```python,linenos
    import httpx

    async def get_users() -> list:
        async with httpx.AsyncClient() as client:
            resp = await client.get("https://api.example.com/users")
            resp.raise_for_status()
            return resp.json()

    # Concurrent requests with rate limiting
    async def fetch_all(urls: list[str]) -> list:
        sem = asyncio.Semaphore(10)    # max 10 concurrent
        async def bounded(url):
            async with sem:
                async with httpx.AsyncClient() as c:
                    return (await c.get(url)).json()
        return await asyncio.gather(*[bounded(u) for u in urls])
```

### Queues and producer/consumer

```python,linenos
    async def producer(queue: asyncio.Queue):
        for i in range(10):
            await queue.put(i)
        await queue.put(None)     # sentinel

    async def consumer(queue: asyncio.Queue):
        while True:
            item = await queue.get()
            if item is None: break
            print(f"Consuming {item}")
            queue.task_done()

    async def main():
        q = asyncio.Queue(maxsize=5)
        await asyncio.gather(producer(q), consumer(q))
```

### asyncio patterns and gotchas

```python,linenos
    # Timeout
    try:
        result = await asyncio.wait_for(fetch(url), timeout=5.0)
    except asyncio.TimeoutError:
        print("timed out")

    # gather with exception handling per task
    results = await asyncio.gather(*tasks, return_exceptions=True)
    for r in results:
        if isinstance(r, Exception): handle_error(r)

    # Run sync code in executor (thread or process pool)
    import time
    result = await asyncio.get_event_loop().run_in_executor(
        None, time.sleep, 1   # None = default ThreadPoolExecutor
    )

    # CPU-bound: ProcessPoolExecutor
    from concurrent.futures import ProcessPoolExecutor
    async def cpu_work():
        loop = asyncio.get_event_loop()
        with ProcessPoolExecutor() as pool:
            return await loop.run_in_executor(pool, heavy_computation)
```

## Standard Library Gems

### pathlib

```python,linenos
    from pathlib import Path

    p = Path("data/reports")
    p.mkdir(parents=True, exist_ok=True)

    # Navigate
    base = Path.home() / "projects" / "myapp"
    cfg  = base / "config.toml"
    cfg.exists(); cfg.is_file(); cfg.is_dir()

    # Read / write
    text = cfg.read_text(encoding="utf-8")
    cfg.write_text("content", encoding="utf-8")
    data = cfg.read_bytes()

    # Glob
    py_files = list(base.rglob("*.py"))   # recursive
    configs   = list(base.glob("*.toml")) # non-recursive

    # Info
    cfg.name       # "config.toml"
    cfg.stem       # "config"
    cfg.suffix     # ".toml"
    cfg.parent     # Path("~/projects/myapp")
    cfg.stat().st_size
```

### itertools and functools

```python,linenos
    import itertools as it
    import functools

    # itertools
    it.chain([1,2], [3,4])        # 1 2 3 4
    it.chain.from_iterable([[1,2],[3]])  # flatten one level
    it.islice(gen, 5)             # first 5 items from generator
    it.groupby(sorted_items, key=lambda x: x.status)
    it.combinations([1,2,3], 2)  # (1,2)(1,3)(2,3)
    it.permutations([1,2,3], 2)  # (1,2)(1,3)(2,1)...
    it.product("AB", repeat=2)    # AA AB BA BB
    it.accumulate([1,2,3,4])     # 1 3 6 10 (running sum)
    it.takewhile(lambda x: x<5, nums)
    it.dropwhile(lambda x: x<5, nums)
    it.repeat(0, times=3)          # 0 0 0

    # functools
    functools.lru_cache(maxsize=128)   # memoize
    functools.cache                      # unbounded cache (3.9+)
    functools.partial(pow, exp=2)      # partial application
    functools.reduce(lambda a, b: a+b, [1,2,3])  # fold left: 6

    @functools.lru_cache(maxsize=None)
    def fib(n): return n if n < 2 else fib(n-1) + fib(n-2)
```

## Pandas & NumPy

### NumPy

```python,linenos
    import numpy as np

    # Create arrays
    a = np.array([1, 2, 3, 4, 5], dtype=np.float64)
    z = np.zeros((3, 4))         # 3×4 of zeros
    o = np.ones((2, 3))          # 2×3 of ones
    r = np.random.rand(5, 5)    # 5×5 uniform [0,1)
    n = np.random.randn(1000)   # normal distribution
    l = np.linspace(0, 1, 100)  # 100 evenly spaced points
    e = np.arange(0, 10, 0.5)   # 0, 0.5, 1.0, ..., 9.5

    # Vectorised operations (NO loops needed)
    a * 2            # element-wise multiply
    a + b            # element-wise add
    np.sqrt(a)       # element-wise sqrt
    a[a > 3]        # boolean indexing: [4, 5]

    # Shape operations
    m = np.arange(12).reshape(3, 4)
    m.T              # transpose
    m.flatten()     # → 1D
    np.concatenate([a, b])
    np.vstack([row1, row2])
    np.hstack([col1, col2])

    # Stats
    a.mean(); a.std(); a.min(); a.max()
    np.median(a); np.percentile(a, 75)
    np.dot(A, B)     # matrix multiplication
    A @ B            # same (3.5+)
```

### Pandas

```python,linenos
    import pandas as pd

    # Create
    df = pd.read_csv("data.csv", parse_dates=["created_at"])
    df = pd.read_json("data.json")
    df = pd.DataFrame({
        "name":  ["Alice", "Bob", "Carol"],
        "score": [90, 75, 88],
        "dept":  ["eng", "eng", "hr"],
    })

    # Inspect
    df.head(10); df.tail(5)
    df.info()         # dtypes, nulls, memory
    df.describe()     # count, mean, std, quartiles
    df.shape          # (rows, cols)
    df.dtypes
    df.isnull().sum() # null counts per column

    # Select
    df["name"]               # Series
    df[["name", "score"]]    # DataFrame subset
    df.iloc[0]              # row by integer position
    df.loc[0]               # row by label
    df.loc[df["score"] > 80]    # filter rows
    df.query("score > 80 and dept == 'eng'")

    # Transform
    df["grade"] = df["score"].apply(lambda s: "A" if s>=90 else "B")
    df["score"] = df["score"] * 1.1           # vectorised
    df.rename(columns={"name": "full_name"})
    df.drop(columns=["dept"])
    df.fillna(0)
    df.dropna(subset=["score"])
    df.astype({"score": int})

    # Aggregate
    df.groupby("dept")["score"].mean()
    df.groupby("dept").agg({"score": ["mean","max","count"]})
    df.pivot_table(values="score", index="dept", aggfunc="mean")

    # Join / merge
    pd.merge(df, other, on="id", how="left")
    pd.concat([df1, df2], ignore_index=True)

    # Export
    df.to_csv("out.csv", index=False)
    df.to_json("out.json", orient="records")
    df.to_parquet("out.parquet")   # columnar, fast
```

## Python Tooling

| Tool | What it does | Command |
|------|--------------|---------|
| uv | Blazing-fast package manager + virtual env (written in Rust, replaces pip + venv) | `uv init / uv add / uv run` |
| ruff | Ultra-fast linter + formatter (replaces flake8, isort, black, in one tool) | `ruff check . / ruff format .` |
| mypy | Static type checker. Reads type hints, catches bugs before runtime | `mypy --strict src/` |
| pytest | The test framework. Fixtures, parametrize, plugins, coverage | `pytest -v --cov` |
| pre-commit | Run checks (ruff, mypy, tests) automatically on every git commit | `pre-commit install` |
| pyproject.toml | Single config file for all tools (replaces setup.py, setup.cfg, tox.ini) | — |
| pydantic | Data validation with type hints. Used in FastAPI, config loading | `uv add pydantic` |
| loguru | Structured, beautiful logging that replaces the std logging module | `from loguru import logger` |
| rich | Pretty terminal output: tables, progress bars, syntax highlighting | `from rich import print` |
| click / typer | Build CLI tools with arguments, options, help text (typer uses type hints) | `uv add typer` |

### pyproject.toml — the modern Python config

```toml
    [project]
    name = "myapp"
    version = "0.1.0"
    requires-python = ">=3.12"
    dependencies = [
        "fastapi>=0.111",
        "pydantic>=2",
    ]

    [project.optional-dependencies]
    dev = ["pytest", "mypy", "ruff"]

    [tool.ruff]
    line-length = 100
    target-version = "py312"

    [tool.ruff.lint]
    select = ["E", "F", "I", "UP", "N"]   # pycodestyle, pyflakes, isort, pyupgrade, pep8-naming

    [tool.mypy]
    strict = true
    python_version = "3.12"

    [tool.pytest.ini_options]
    testpaths = ["tests"]
    addopts = "--tb=short -q"
```

### pytest patterns

```python,linenos
    # tests/test_example.py
    import pytest

    def test_basic():
        assert 1 + 1 == 2

    # Parametrize
    @pytest.mark.parametrize("n,expected", [(2,4), (3,9), (5,25)])
    def test_square(n, expected):
        assert n**2 == expected

    # Fixtures
    @pytest.fixture
    def db_session():
        db = create_test_db()
        yield db           # setup before / teardown after
        db.rollback()
        db.close()

    def test_create_user(db_session):
        user = db_session.create_user("alice")
        assert user.id is not None

    # Expect exception
    def test_raises():
        with pytest.raises(ValueError, match="too short"):
            validate("")

    # Mock
    from unittest.mock import patch, MagicMock

    def test_email():
        with patch("myapp.email.send") as mock_send:
            notify_user("alice@ex.com")
            mock_send.assert_called_once_with("alice@ex.com", subject="Welcome")
```

## Rust Extensions for Python (PyO3 + maturin)

PyO3 lets you write Python extension modules in Rust. Use this when Python is too slow for a specific function — image processing, parsers, math-heavy code. The Rust function becomes a regular importable Python module.

### Setup

```bash
    # Install maturin (the build tool)
    uv add maturin --dev
    # or:
    pip install maturin

    # Create a new Rust extension project
    maturin new mylib --bindings pyo3
    # Creates: mylib/Cargo.toml + src/lib.rs + pyproject.toml

    # Install in current virtualenv (dev mode)
    cd mylib
    maturin develop

    # Build a release wheel
    maturin build --release
```

### Writing the Rust extension

```rust
    // src/lib.rs
    use pyo3::prelude::*;
    use pyo3::wrap_pyfunction;

    /// A simple function exposed to Python
    #[pyfunction]
    fn add(a: i64, b: i64) -> i64 {
        a + b
    }

    /// Compute sum of squares — vectorised over a Python list
    #[pyfunction]
    fn sum_of_squares(nums: Vec<f64>) -> f64 {
        nums.iter().map(|x| x * x).sum()
    }

    /// A Rust class exposed to Python
    #[pyclass]
    struct Counter {
        count: u64,
    }

    #[pymethods]
    impl Counter {
        #[new]
        fn new() -> Self {
            Counter { count: 0 }
        }

        fn increment(&mut self) {
            self.count += 1;
        }

        fn value(&self) -> u64 {
            self.count
        }
    }

    /// The module name must match the package name in pyproject.toml
    #[pymodule]
    fn mylib(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(add, m)?)?;
        m.add_function(wrap_pyfunction!(sum_of_squares, m)?)?;
        m.add_class::<Counter>()?;
        Ok(())
    }
```

### Using the extension in Python

```python,linenos
    import mylib

    result = mylib.add(3, 4)          # 7
    total  = mylib.sum_of_squares([1.0, 2.0, 3.0])  # 14.0

    c = mylib.Counter()
    c.increment()
    c.increment()
    print(c.value())  # 2
```

### Cargo.toml

```toml
    [package]
    name = "mylib"
    version = "0.1.0"
    edition = "2021"

    [lib]
    name = "mylib"
    crate-type = ["cdylib"]   # dynamic lib loadable by Python

    [dependencies]
    pyo3 = { version = "0.21", features = ["extension-module"] }
```

> **When to use Rust extensions:** JSON parsers, image processing, graph algorithms, cryptography, any hot loop that runs millions of iterations. For most business logic, pure Python is fine.

> **NumPy interop:** Use the `numpy` crate in Rust (`pyo3-numpy`) to pass NumPy arrays directly to Rust with zero copies. Enables massively fast numerical code with a clean Python interface.