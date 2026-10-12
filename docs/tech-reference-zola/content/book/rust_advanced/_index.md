+++
title = "Rust — Beyond the Basics"
description = "The mental models, decisions, and hard-won details that separate working Rust from idiomatic Rust"
[extra]
date = 2026-10-12
+++

> **Read this after Vol. 07.** That book teaches the syntax: what a `Vec` is, how `match` works, what `?` does. This one teaches the **judgement** — which of six ways to hold data to reach for, when a clone is honest versus lazy, why the borrow checker is fighting you, and which crates will still be here in 2030.
>
> Every code block in this book was compiled and run against **rustc 1.98.1**. Every crate version and download number was read from the crates.io API on **October 2026**. Nothing here is from memory.

## How to Read This Book

The chapters are ordered roughly by how much they change how you write code. If you only read six, read these:

| Chapter | Why it matters |
|---------|-----------------|
| [Data Structure Selection](#data-structure-selection) | The single biggest lever on whether your Rust is idiomatic or fighting you |
| [Ownership: The Parts That Surprise](#ownership-the-parts-that-surprise) | Partial moves, drop order, and why `Copy` is not what you think |
| [Borrowing & Lifetimes: Surviving the Checker](#borrowing-lifetimes-surviving-the-checker) | The four legitimate escape hatches, and when each is correct |
| [Clone, Copy, and When to Stop Fighting](#clone-copy-and-when-to-stop-fighting) | Clone is a design tool. Here is when it is one. |
| [Traits & Generics](#traits-generics) | Generic vs `dyn`, associated types, sealed traits |
| [Errors That Carry Information](#errors-that-carry-information) | Library vs application error design |

## The Core Mental Model

Most Rust confusion comes from one misreading. Rust does **not** manage memory. It manages **ownership**, and memory follows from that.

Three consequences generate nearly every rule in the language:

| Principle | What it produces |
|-----------|-----------------|
| Every value has exactly one owner | Moves; no accidental double-free |
| Multiple readers **or** one writer, never both | The borrow checker; no data races |
| Values are dropped in reverse declaration order | Deterministic cleanup; RAII |

Everything else — lifetimes, traits, generics, `Send`/`Sync`, the `?` operator — is a consequence of these three. When a rule surprises you, ask which of the three it protects.

> **Rust is not about performance.** It is about making illegal states unrepresentable, and performance falls out of that. Code that is correct-by-construction usually needs no `unsafe`, no caching, and no premature optimization.

## Data Structure Selection

This is the chapter that most changes how your code looks. Rust gives you six ways to hold data, and choosing wrong is the root of most "the borrow checker hates me" situations.

### The Selection Table

| You need | Use | Why | Avoid |
|----------|-----|-----|-------|
| Fixed set of named fields | `struct` | Field access, no indirection | `HashMap<String, Box<dyn Any>>` |
| One of several distinct variants | `enum` | Compiler enforces every case | `Option`-soup, sentinel values |
| 2–3 fields passed around together | tuple `struct` | Grouping, named type | loose tuples |
| A distinct semantic meaning | newtype | **Cannot** mix up with the base type | bare `type Meters = f64` |
| Invalid-order-proof state machine | typestate | States are distinct types | a `valid: bool` field |
| Lookup by key | `HashMap<K, V>` | O(1) average | linear scans |
| Ordered / ranged lookup | `BTreeMap<K, V>` | O(log n), sorted iteration, range queries | sorting a `Vec` each time |
| Insertion order matters | `IndexMap<K, V>` | Hash speed + stable order | `HashMap` + re-sorting |
| Known, small, fixed count | array `[T; N]` | Stack-allocated, inline | `Vec` for 3 items |
| View into someone else's data | slice `&[T]` | Borrow, no copy, no alloc | `&Vec<T>` (hides intent) |
| Recursive or large type | `Box<T>` | Fixed-size handle to heap | stack overflow |
| Shared ownership, single thread | `Rc<T>` | Counted pointer | `Arc` (unnecessary atomics) |
| Shared ownership, threads | `Arc<T>` | Atomic refcount | `Rc` (won't compile cross-thread) |

### Struct vs Enum: The Actual Rule

The question is not "how many fields" but **"how many valid shapes does this thing have?"**

```rust,linenos
    // ONE shape → struct
    struct Point { x: f64, y: f64 }

    // MANY shapes, each carrying different data → enum
    enum Shape {
        Circle { radius: f64 },
        Rect { w: f64, h: f64 },
        Empty,              // a variant with no data is legal and meaningful
    }
```

The enum version is strictly better because the compiler forces you to handle `Empty`. A struct with an `Option` for each field produces states like "circle with a width of 3.0" that your domain forbids.

**Reach for `Option` before reaching for the new struct.** Only introduce a new type when you genuinely need *two or more* variants with *different* payloads.

### Struct Update Syntax and `..Default`

`..Default::default()` is the cleanest way to have optional fields. Combine it with `#[derive(Default)]` and you never write a manual constructor again:

```rust,linenos
    #[derive(Debug, Clone, Default, PartialEq)]
    struct ServerConfig {
        host: String,          // "" — caller almost always sets this
        port: u16,             // 0 — invalid, must be set
        tls: bool,             // false — sane default
        timeout_secs: Option<u64>,
    }

    impl ServerConfig {
        // Named constructor for the common case
        fn dev() -> Self {
            Self { host: "localhost".into(), port: 8080, tls: false, ..Default::default() }
        }
    }

    // Patch a single field, keep the rest — the real workhorse
    let prod = ServerConfig { tls: true, ..ServerConfig::dev() };
    assert!(prod.tls && prod.port == 8080);
```

> **Warning:** struct update syntax **moves** non-`Copy` fields out of the base value. `..cfg` consumes `cfg`. If you need the base afterwards, derive `Clone` or operate on a reference.

### The Newtype: Zero-Cost Type Safety

A newtype is a one-field tuple struct. It costs **zero bytes and zero runtime** — the compiler optimizes it to the inner representation — while making illegal programs uncompilable.

```rust,linenos
    // `type` aliases are NOT types — this compiles but buys nothing:
    type UserId = u64;
    fn f(a: UserId, b: UserId) {}  // you can now pass any u64. Useless.

    // A newtype IS a real, distinct type:
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct UserId(u64);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct OrderId(u64);

    fn process(user: UserId, order: OrderId) { /* ... */ }

    // process(5, 6);   // COMPILE ERROR: expected UserId, found OrderId
    // process(OrderId(5), UserId(6));  // COMPILE ERROR: arguments swapped
```

Newtypes are the single best answer to "the compiler can't help me here." They cost nothing and they eliminate whole classes of bugs. Pair one with `TryFrom` for parsing:

```rust,linenos
    use std::num::ParseIntError;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct UserId(u64);

    impl std::str::FromStr for UserId {
        type Err = ParseIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u64>().map(UserId)   // maps u64 -> UserId
        }
    }

    fn lookup(raw: &str) -> Result<UserId, ParseIntError> {
        raw.parse()   // returns UserId, no map needed
    }
```

**Real uses:** domain identifiers (`UserId` vs `OrderId`), units (`Meters` vs `Feet`), validated strings (`Email` that can only be constructed post-validation), and phantom type parameters that make a unit visible in the signature.

```rust,linenos
    #[derive(Debug, Clone, Copy)]
    struct Meters(f64);

    impl Meters {
        fn to_feet(self) -> Feet { Feet(self.0 * 3.28084) }
    }

    #[derive(Debug, Clone, Copy)]
    struct Feet(f64);

    // Now this is impossible: you cannot pass Feet where Meters is expected.
    fn draw(d: Meters) { /* ... */ }
```

### Typestate: Encoding Valid Order in the Type System

Instead of a runtime `Option`/`Result`, make each state a **distinct type**. Then the compiler rejects out-of-order calls entirely.

```rust,linenos
    use std::marker::PhantomData;

    struct Tcp;           // raw socket
    struct Handshaked;    // TLS negotiated
    struct Authed;        // credentials accepted

    struct Conn<S> {
        inner: Tcp,              // the payload stays one type
        _state: PhantomData<S>,  // the type param is the state marker
    }

    impl Tcp {
        fn connect(self) -> Conn<Tcp> {
            Conn { inner: self, _state: PhantomData }
        }
    }

    // Each method is only available in the right state — note `self` is consumed
    // and a new, differently-typed value is returned.
    impl Conn<Tcp> {
        fn handshake(self) -> Conn<Handshaked> {
            Conn { inner: self.inner, _state: PhantomData }
        }
    }

    impl Conn<Handshaked> {
        fn auth(self) -> Conn<Authed> {
            Conn { inner: self.inner, _state: PhantomData }
        }
    }

    // This method exists ONLY on Conn<Authed>.
    impl Conn<Authed> {
        fn query(&self, _sql: &str) -> &'static str { "rows" }
    }

    // Correct order: compiles.
    let conn = Tcp.connect().handshake().auth();
    assert_eq!(conn.query("SELECT 1"), "rows");

    // conn.query("...") on a fresh Conn<Tcp> does not exist. The compiler stops you.
```

> **The `PhantomData` gotcha that costs everyone an hour:** the type parameter `S` and the field `inner` must be *different* types. If you write `struct Conn<S> { inner: S }` and try to transition `Conn<Tcp> -> Conn<Handshaked>`, you hit `expected Handshaked, found Tcp` — because you're trying to store a `Tcp` in a field that must hold a `Handshaked`. Decouple them: store the real payload in a concrete field and use `PhantomData<S>` purely as the state tag. This compiles cleanly, as shown above.

Typestate is genuinely worth it for protocol clients, parsers, and anything with a "must do X before Y" rule. Do **not** use it for simple "optional value" — that's what `Option` is for.

### HashMap vs BTreeMap vs IndexMap

| | `HashMap` | `BTreeMap` | `IndexMap` |
|-|-----------|------------|------------|
| Lookup | O(1) average | O(log n) | O(1) average |
| Sorted iteration | No | **Yes** | No (insertion order) |
| Range queries (`.range(a..b)`) | No | **Yes** | No |
| Ordering requirement on key | `Hash + Eq` | `Ord` | `Hash + Eq` |
| Extra memory | ~0 | Node overhead | Index array |
| Worst case | Hash flooding (DoS risk) | Deterministic | Hash flooding |

**Rule of thumb:** default to `HashMap`. Switch to `BTreeMap` when you need sorted output or range queries. Use `IndexMap` when the **order** is semantically meaningful (e.g. preserving a config file's order, rendering a UI list) and you still want O(1) lookup.

> **HashMap has no stable iteration order** and does not guarantee it between runs. Never rely on it. If you need determinism, collect and sort, or use `BTreeMap`.

The `entry` API is the idiomatic way to avoid the "check then insert" double-lookup:

```rust,linenos
    use std::collections::HashMap;

    let mut scores: HashMap<String, Vec<i32>> = HashMap::new();
    scores.entry("ana".into()).or_default().push(10);
    scores.entry("ana".into()).or_default().push(20);
    // entry().or_insert(v)  -> insert v only if vacant
    // entry().or_insert_with(|| expensive())  -> lazy, only runs if vacant
    // entry().or_default() -> for Vec/HashMap/String/Box (implements Default)
    assert_eq!(scores["ana"], vec![10, 20]);
```

### Vec vs Array vs Slice

These three differ only in **ownership and location**, which makes the choice mechanical once you see it:

```rust,linenos
    let arr: [i32; 3] = [1, 2, 3];   // fixed size, inline on the stack, Copy if i32
    let vec: Vec<i32> = vec![1, 2, 3]; // growable, heap buffer + (ptr, len, cap)
    let sl: &[i32] = &arr;           // borrowed view, fat pointer (ptr + len), no copy
    let sl2: &[i32] = &vec;          // borrows the Vec, no copy

    assert_eq!(sl.len(), 3);          // slices know their length at runtime
    // arr.len() is a compile-time constant; arrays are length-checked by the type system
```

**When each one:**

- **Array `[T; N]`** — the count is known at compile time and small. `[u8; 32]` for a hash, `[i32; 3]` for a vertex. No allocation.
- **`Vec<T>`** — the default. Growable, heap-allocated, three-word size (`ptr`, `len`, `cap`).
- **Slice `&[T]`** — you have data someone else owns and you only want to read or transform it. The single most under-used type in beginner Rust because people pass `&Vec<T>` out of habit.

**Never write `&Vec<T>`.** Write `&[T]`. A `&Vec<T>` is a pointer to a *specific growable buffer* and it leaks the implementation detail into your API, preventing callers from passing arrays, slices from other sources, or sub-ranges. `&[T]` accepts all of those.

> `&mut Vec<T>` is a different story — there you sometimes *do* want `&mut Vec<T>` specifically, because `push` is not on slices. Use `&mut [T]` when you only need to mutate existing elements.

## Ownership: The Parts That Surprise

The basics are in Vol. 07. These are the details that bite.

### Partial Moves

You can move *out* of a struct field without moving the whole struct — and you can still use the parts you didn't move.

```rust,linenos
    struct User { name: String, age: u32 }

    let u = User { name: "ana".to_string(), age: 30 };

    let name = u.name;       // partial move: String moved out
    // println!("{}", u.name);  // ERROR: u.name was moved
    println!("{}", u.age);      // FINE: u.age is u32 (Copy), untouched
```

But the *whole* struct is not usable as a value after a partial move, because the compiler no longer knows it's fully initialized:

```rust,linenos
    let u2 = User { name: "bob".to_string(), age: 25 };
    let n2 = u2.name;
    // let u3 = u2;               // ERROR: u2 partially moved
    // u2.age = 40;               // ERROR: cannot assign to u2, it's partially moved
```

**Partial move out of a collection** works the same way, and it's how you drain without `clone`:

```rust,linenos
    let v = vec![String::from("a"), String::from("b")];
    // `into_iter()` moves every String out — no clones, zero allocation.
    let owned: Vec<String> = v.into_iter().collect();
    // Or, to keep the Vec and take one item:
    let mut v2 = vec![String::from("x"), String::from("y")];
    let first = v2.remove(0);      // moves the String out, shifts the rest
    assert_eq!(first, "x");
```

### Drop Order: Reverse Declaration

Rust drops values in **reverse declaration order**, and *struct fields* drop in declaration order. This matters when values depend on each other.

```rust,linenos
    struct Guard(&'static str);
    impl Drop for Guard {
        fn drop(&mut self) { println!("drop {}", self.0); }
    }

    {
        let _a = Guard("a");
        let _b = Guard("b");
        let _c = Guard("c");
        // Dropped in reverse: c, then b, then a
    }
```

Why reverse? Because a later-declared value often *depends* on an earlier one (a guard that must drop before the lock it holds). Reverse order makes that the safe default.

**Manually controlling order** with `std::mem::drop` when you need an early release:

```rust,linenos
    let lock = std::sync::Mutex::new(0);
    {
        let _guard = lock.lock().unwrap();
        // ... critical section ...
    }   // guard released here

    // Equivalent, when you need to release mid-scope:
    let big = vec![0u8; 1024];
    drop(big);              // free memory NOW, before allocating more
    let other = vec![0u8; 1024];
```

> **RAII is the whole point.** A `MutexGuard` releases the lock on drop. A file handle closes on drop. A database transaction commits or rolls back on drop. You almost never write explicit cleanup code in Rust — you attach cleanup to a value's lifetime.

### `Copy` Is Not "Cheap"

`Copy` means "bitwise copy and the original stays usable." It requires **all** fields to be `Copy`. It is *not* about performance and it is *not* a marker of "simple type."

```rust,linenos
    // Derived Copy works automatically for plain data:
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Point { x: f64, y: f64 }

    let a = Point { x: 1.0, y: 2.0 };
    let b = a;            // copied, `a` still usable
    let c = a;            // copied again
    assert_eq!(a, b);

    // Arrays of Copy types are Copy up to length 32 (historically). Modern Rust:
    let arr = [1u8; 64];
    let arr2 = arr;       // copies the whole array (64 bytes)
```

**What is `Copy`:** integers, floats, `bool`, `char`, raw pointers, shared references `&T`, and tuples/arrays of those.

**What is not `Copy`:** `String`, `Vec`, `Box`, `Rc`, `Arc`, anything with a `Drop` impl, and `&mut T`.

**Do not derive `Copy` on a struct that contains a `String`, `Vec`, or a `Drop` type** — the compiler will refuse, and rightly so.

### `Clone` vs Deep Copy

`clone()` is *not* necessarily a deep copy. It calls your type's `Clone` impl, which you control.

```rust,linenos
    // Clone for Rc is a refcount bump, NOT a deep copy:
    let data = Rc::new(vec![1, 2, 3]);
    let copy = data.clone();
    assert_eq!(Rc::strong_count(&data), 2);
    // Both point to ONE heap allocation.

    // For plain owned data, Clone is a real deep copy:
    let v = vec![1, 2, 3];
    let w = v.clone();    // new allocation
    assert_eq!(v, w);
```

This matters for the "when to clone" question: cloning a handle (`Rc`, `Arc`, `&`-backed smart pointer) is cheap and often correct. Cloning a `Vec<String>` of a thousand strings is expensive, and usually a design smell — see below.

## Clone, Copy, and When to Stop Fighting

Cloning has a reputation as "giving up on the borrow checker." That is wrong, but the nuance is real. Here is the decision framework.

### The Four Legitimate Uses

**1. Escaping a borrow — usually correct, usually cheap.**

```rust,linenos
    // You need to store data beyond the lifetime of a borrow you were handed.
    fn parse(input: &str) -> Vec<u32> {
        input.split(',').map(|s| s.parse().unwrap()).collect()
    }   // `input` is borrowed; the Vec<u32> is owned. No clone needed here —
        // because parse() produces owned numbers.

    // When you DO need a clone: transforming borrowed data into owned storage.
    fn normalize(s: &str) -> String {
        s.trim().to_lowercase()   // to_lowercase() returns a NEW String, not a clone
    }
```

**2. Keeping the API ergonomic for the caller.**

```rust,linenos
    struct Server { name: String }

    impl Server {
        // Accept &self but return owned, so the caller can keep the result
        // around without fighting the borrow checker. Cost: one small alloc.
        fn name(&self) -> String { self.name.clone() }

        // Better when the caller doesn't need ownership:
        fn name_ref(&self) -> &str { &self.name }
    }
```

Rule of thumb: a `String` clone is a single heap allocation of a small buffer. For anything but the hottest inner loop, **it is free** and the ergonomic win is large. Do not contort your API to avoid a small `String` clone.

**3. Small, cheap, and the borrow would otherwise be awkward.**

```rust,linenos
    #[derive(Clone, Copy)]
    struct Point { x: f64, y: f64 }

    let p = Point { x: 1.0, y: 2.0 };
    let snapshot = p;    // Copy — no heap, no clone needed
    use_it(p);
    use_it(snapshot);    // still works
```

**4. Thread/task boundaries require ownership.**

```rust,linenos
    // `thread::spawn` requires `'static`, so anything borrowed can't go in.
    let data = vec![1, 2, 3];
    let handle = std::thread::spawn({
        let data = data.clone();       // clone here is CORRECT — the thread
        move || data.iter().sum::<i32>()   // needs owned data
    });
    let _ = handle.join().unwrap();
```

### When Clone Is a Smell

Cloning to escape the borrow checker is fine for **small** types. It is a design signal for **large** collections or in **loops**.

```rust,linenos
    // BAD: clone inside a loop — O(n) allocations for no reason
    fn bad(rows: &[String]) -> Vec<Vec<String>> {
        rows.iter().map(|r| {
            let owned = r.clone();      // unnecessary: we could return borrowed refs
            owned.chars().rev().collect()
        }).collect()
    }

    // BETTER: return references when the output is a view of the input
    fn better<'a>(rows: &'a [String]) -> Vec<&'a String> {
        rows.iter().collect()          // just copies the references, not the data
    }
```

**Better still:** restructure so there is no intermediate collection at all.

```rust,linenos
    // BEST: fuse the operations, return one owned result
    fn best(rows: &[String]) -> Vec<String> {
        rows.iter()
            .filter(|r| !r.is_empty())
            .map(|r| r.to_uppercase())      // new value, no clone of r
            .collect()                       // one Vec built directly
    }
```

### The `Cow` Escape Hatch

`Cow` (clone-on-write) is `std`'s built-in answer to "this might need to allocate, but often won't." It is an enum that is either a borrowed view or an owned value.

```rust,linenos
    use std::borrow::Cow;

    fn normalize(s: &str) -> Cow<'_, str> {
        if s.contains(' ') {
            Cow::Owned(s.replace(' ', "-"))   // allocate, because we must
        } else {
            Cow::Borrowed(s)                  // zero-cost, just a reference
        }
    }

    assert_eq!(normalize("a b"), "a-b");
    assert_eq!(normalize("ab"), "ab");
```

`Cow<'a, T>` means "a borrowed `&'a T` **or** an owned `T`." It shows up in real APIs: `String::from_utf8_lossy` returns `Cow<str>`, and many parsers return `Cow<'a, str>`.

### When to Stop Fighting and What to Reach For

If you find yourself adding `.clone()` three times in one function, do not add a fourth. Change the design:

| Symptom | Do this instead |
|---------|-----------------|
| Cloning a `Vec`/`String` to escape a borrow | Change the function to return owned, or accept `&str` |
| Cloning a field to return it | Add `name_ref(&self) -> &str`, or store an `Arc<str>` |
| `Rc::clone` everywhere | Consider whether the data is truly shared — a plain reference may do |
| Borrow conflict in a struct field | Use `Box`, `Rc`, or `Arc` to break the cycle |
| Can't return a reference from a loop | Collect into a `Vec` and return owned, or restructure |

**The test:** clone is acceptable if the cloned value is small (a few dozen bytes, one allocation) and not in a hot loop. It's a smell if you clone collections, clone repeatedly, or clone to work around a function signature you control.

## Borrowing & Lifetimes: Surviving the Checker

### Reading Borrow Checker Errors

The error message is a specification, not a complaint. It tells you the two conflicting borrows and their spans. Work through it in this order:

1. **Which two borrows conflict?** One is usually shared (`&`), one mutable (`&mut`).
2. **Where does the first borrow end?** (NLL = last *use*, not end of scope.)
3. **Does the value need to outlive the borrow?** Then it needs ownership, not a borrow.

### Non-Lexical Lifetimes in Practice

Since Rust 2018, a borrow ends at its **last use**, not the end of the block. This unblocks a lot:

```rust,linenos
    let mut v = vec![1, 2, 3];
    let first = &v[0];     // shared borrow
    println!("{}", first); // last use of `first` — borrow ENDS here
    v.push(4);             // MUTABLE borrow now allowed (would not compile pre-NLL)
    assert_eq!(v.len(), 4);
```

But the borrow is still alive if you hold a reference *past* the mutation:

```rust,linenos
    let mut v = vec![1, 2, 3];
    let r = &v[0];
    v.push(4);             // ERROR: v is mutably borrowed because `r` is still live
    println!("{}", r);
```

### Reborrowing: The Automatic Fix

When you pass `&mut T` to a function that takes `&T`, the compiler **reborrows** automatically — you don't need `&*`:

```rust,linenos
    fn read(s: &[i32]) -> usize { s.len() }

    let mut v = vec![1, 2, 3];
    let n = read(&v);      // &v is coerced to &[i32]; reborrow, not a move
    v.push(4);             // fine — the borrow ended at `read`
    assert_eq!(n, 3);
```

### The Four Legitimate Escape Hatches

When borrowing genuinely can't work, these are the tools, in order of preference:

**1. Change the signature to take ownership (best).**

```rust,linenos
    // Instead of fighting over a borrow, consume what you need.
    fn process(data: Vec<u32>) -> u32 { data.iter().sum() }

    let owned = vec![1, 2, 3];
    let total = process(owned);   // moved in; caller doesn't need it after
```

**2. `Box` — break the size cycle.**

Useful when a struct needs to hold itself (recursive) or when a large variant bloats every instance:

```rust,linenos
    enum Node {
        Leaf(i32),
        Branch { left: Box<Node>, right: Box<Node> },   // Box makes this finite
    }
```

**3. `Rc` / `Arc` — shared ownership.**

When two owners genuinely need the same data:

```rust,linenos
    use std::rc::Rc;
    let shared = Rc::new(vec![1, 2, 3]);
    let a = Rc::clone(&shared);       // refcount++, same allocation
    assert_eq!(Rc::strong_count(&shared), 2);
```

**4. Lifetimes — tie the output to the input (the right tool, used last).**

See the next section. This is *not* an escape hatch — it's the correct solution when a function returns a view into its argument.

### When You Actually Need a Lifetime Annotation

Lifetimes are elided in the overwhelming majority of cases. You write one only when the compiler **cannot** infer the relationship. There are exactly three situations:

**1. Returning a reference derived from an input.**

```rust,linenos
    // Elision can't tell which input `out` refers to — there are two.
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() >= y.len() { x } else { y }
    }

    // If you only need the output to live as long as ONE input,
    // name them separately:
    fn first_word<'a>(s: &'a str) -> &'a str {
        s.split(' ').next().unwrap_or("")
    }
```

**2. A struct holds a reference.**

```rust,linenos
    #[derive(Debug)]
    struct Excerpt<'a> {
        part: &'a str,     // must not outlive the data it points to
    }

    impl<'a> Excerpt<'a> {
        fn first_sentence(text: &'a str) -> Self {
            Excerpt { part: text.split('.').next().unwrap_or("") }
        }
        fn level(&self) -> i32 { 3 }   // 'a elided here
    }

    let novel = String::from("Call me Ishmael. ...");
    let ex = Excerpt::first_sentence(&novel);
    assert!(ex.part.starts_with("Call"));
```

**3. Higher-ranked trait bounds** (`for<'a>`), when a closure must work for any lifetime:

```rust,linenos
    fn apply<F>(f: F, xs: &[i32]) -> Vec<i32>
    where
        F: for<'a> Fn(&'a i32) -> i32,   // accepts references of ANY lifetime
    {
        xs.iter().map(|x| f(x)).collect()
    }

    assert_eq!(apply(|x| x * 2, &[1, 2, 3]), vec![2, 4, 6]);
```

### Lifetime Elision Rules

These three rules cover ~99% of signatures without you writing anything:

| Rule | Condition | Effect |
|------|-----------|--------|
| 1 | Every reference is a parameter | Each gets its own lifetime: `fn f(x: &T, y: &U)` → `fn f<'a,'b>(x: &'a T, y: &'b U)` |
| 2 | Exactly one input lifetime | It is assigned to all output lifetimes |
| 3 | There's a `&self` or `&mut self` | That lifetime is assigned to the return value |

Rule 3 is why this works with no annotation:

```rust,linenos
    struct Parser<'a> { input: &'a str, pos: usize }

    impl<'a> Parser<'a> {
        fn new(input: &'a str) -> Self { Parser { input, pos: 0 } }

        // Rule 3: return lifetime == &self's lifetime. No annotation needed.
        fn rest(&self) -> &'a str { &self.input[self.pos..] }
    }
```

> **Design heuristic:** if a struct needs a lifetime, it probably wants to **own** the data instead (`String` not `&str`, `Vec<u8>` not `&[u8]`). Owning removes the lifetime entirely, and the allocation usually costs less than the ergonomic tax of threading `'a` through every signature. Use lifetimes when you genuinely cannot afford the copy.

### Structurally Validating Lifetimes

A struct with `PhantomData<&'a T>` inherits the lifetime even if it stores no actual reference — useful for carrying ownership semantics:

```rust,linenos
    use std::marker::PhantomData;

    struct Parser<'a> {
        input: &'a str,
        pos: usize,
    }

    struct OwnedWithLifetime<'a> {
        data: String,          // actually owns the data
        _marker: PhantomData<&'a ()>,   // but still tracks a lifetime
    }
```

The `_marker` field tells the compiler "this struct behaves as if it borrows for `'a`," which matters for variance and drop-check. Name it with a leading underscore so it's not flagged as unused.
## Traits & Generics

A trait is a contract. Rust has no inheritance; traits compose instead. This chapter covers the parts you need beyond "define a trait and implement it."

### Generic vs `dyn`: Choosing Dispatch

This is the decision that shapes every library API. There are exactly two kinds of dispatch in Rust:

**Static dispatch (generics, `impl Trait`)** — monomorphized at compile time. One copy of the function per concrete type. Zero runtime cost, but code bloat and compile-time cost.

**Dynamic dispatch (`dyn Trait`, `Box<dyn Trait>`)** — one copy of the function, a vtable pointer at runtime. Flexible, accepts heterogeneous collections, but one indirection.

```rust,linenos
    trait Render { fn render(&self) -> String; }
    struct Html; struct Md;
    impl Render for Html { fn render(&self) -> String { "<p/>".into() } }
    impl Render for Md   { fn render(&self) -> String { "**p**".into() } }

    // Generic: one binary copy per type, inlined, no indirection.
    fn show_one(r: &impl Render) -> String { r.render() }

    // Dynamic: one copy, works with a heterogeneous Vec.
    fn show_any(r: &dyn Render) -> String { r.render() }

    let docs = vec![Box::new(Html) as Box<dyn Render>, Box::new(Md)];
    let parts: Vec<String> = docs.iter().map(|d| show_any(d.as_ref())).collect();
    assert_eq!(parts.join(","), "<p/>,**p**");
```

**Decision rule:**

| Situation | Use | Why |
|-----------|-----|-----|
| Function takes one concrete type | `impl Trait` or generic | Cleanest signature |
| You need a **collection** of mixed types | `Box<dyn Trait>` / `&dyn Trait` | Generics can't express a heterogeneous `Vec<T>` |
| Performance-critical inner loop | Generic (monomorphized) | Inlined, no vtable |
| Plugin architecture / runtime selection | `dyn Trait` | Types aren't known until runtime |
| Library API you want zero-cost | Generic | Callers get inlining |

> **Performance reality:** `dyn` dispatch costs one pointer dereference per call — a few nanoseconds. This is almost always irrelevant. **Compile times and binary size** are the real costs of generics, and those are real. Do not contort your API for runtime speed you cannot measure.

### Associated Types vs Generic Type Parameters

This trips up nearly everyone. The rule:

- **Generic type parameter (`fn f<T: Trait>`)** → the caller picks one type for the whole function. Use when the caller has exactly one choice.
- **Associated type (`trait Repo { type Item; }`)** → each implementor picks its own. Use when the type is tied to the implementor.

```rust,linenos
    // Generic param: ONE T for the whole function.
    fn show_all<T: Display>(items: &[T]) { for i in items { println!("{i}"); } }
    // Can't hold a Vec of mixed Display types.

    // Associated type: each impl chooses its own Item.
    trait Repo {
        type Item;                     // e.g. User, or Row, or Document
        fn all(&self) -> Vec<Self::Item>;
    }

    struct Users;
    struct User { id: u64 }
    impl Repo for Users { type Item = User; fn all(&self) -> Vec<User> { vec![User{id:1}] } }

    struct Posts;
    struct Post { slug: String }
    impl Repo for Posts { type Item = Post; fn all(&self) -> Vec<Post> { vec![] } }

    // Now one function handles both:
    fn count<R: Repo>(r: &R) -> usize { r.all().len() }
    assert_eq!(count(&Users), 1);
```

**When you need both**, use an associated type with a `where` clause:

```rust,linenos
    trait Repo { type Item; fn all(&self) -> Vec<Self::Item>; }

    // Associated type constrained to Display:
    fn print_all<R>(r: &R) -> String
    where
        R: Repo,
        R::Item: std::fmt::Display,   // now we know Item is printable
    {
        r.all().iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
    }
```

### Blanket Implementations

You can implement a trait for *all* types satisfying a bound. This is how `From<T>` gives you `Into<T>` for free, and how `Display` implies `ToString`.

```rust,linenos
    use std::fmt::Display;

    // Blanket impl: every T: Display automatically gets ToString.
    // (This is what actually exists in std — you don't write it yourself.)
    // impl<T: Display> ToString for T { ... }

    // A useful custom blanket impl:
    trait Describe { fn describe(&self) -> String; }

    impl<T: Display> Describe for T {
        fn describe(&self) -> String { format!("<{self}>") }
    }

    assert_eq!(42u32.describe(), "<42>");
    assert_eq!("hi".describe(), "<hi>");
```

> **Blanket impls are powerful and dangerous.** They apply everywhere, so you cannot later add a specific impl for one concrete type without a coherence conflict. Use them when the implementation is genuinely universal.

### Marker and Sealed Traits

A **marker trait** has no methods — it just tags a type as having a property.

```rust,linenos
    trait JsonSerialize {}          // no methods; just a capability tag

    struct Money;  impl JsonSerialize for Money {}
    struct User;   impl JsonSerialize for User {}
    struct Blob;                            // NOT serializable

    fn to_json<T: JsonSerialize>(_v: &T) -> String { "{}".into() }

    to_json(&Money);   // ok
    // to_json(&Blob); // ERROR: Blob does not implement JsonSerialize
```

A **sealed trait** is a trait you can implement, but only inside your own crate. It prevents downstream code from implementing it, which lets you add methods later without breaking them.

```rust,linenos
    mod sealed {
        pub trait Sealed {}   // private module = downstream can't impl
        impl Sealed for u32 {}
        impl Sealed for String {}
    }

    // Public trait, but bound to a private one: nobody outside can implement it.
    pub trait Shiny: sealed::Sealed {
        fn shine(&self) -> String;
    }

    impl Shiny for u32    { fn shine(&self) -> String { format!("{}✨", self) } }
    impl Shiny for String { fn shine(&self) -> String { format!("{self}✨") } }

    assert_eq!(7u32.shine(), "7✨");
    // External crates cannot impl Shiny for their own types. You can add
    // methods to Shiny in a minor release without breaking anyone.
```

### Supertraits

A supertrait is a trait bound that another trait requires. It guarantees a trait always has its dependency's methods.

```rust,linenos
    use std::fmt::Debug;

    trait Summary: Debug {          // any Summary must also be Debug
        fn summarize(&self) -> String;
    }

    #[derive(Debug)]
    struct Article;

    impl Summary for Article {
        fn summarize(&self) -> String { "article".into() }
    }

    // We get .summarize() AND .debug() because Debug is a supertrait.
    fn render(s: &impl Summary) -> String {
        format!("{} ({:?})", s.summarize(), s)
    }
```

### The Orphan Rule

You can only implement a trait for a type if **either** the trait or the type is defined in your crate. This prevents two crates from both adding `Display` to `Vec<T>` and conflicting.

**Legal:** you define `MyTrait`, `Vec<T>` is foreign → you may implement `MyTrait for Vec<T>`.
**Illegal:** both `Display` and `Vec` are foreign → forbidden.

**The workaround** — newtype wrapper (covered in [Data Structure Selection](#data-structure-selection)):

```rust,linenos
    struct MyVec(Vec<u8>);       // local type unlocks impls on it
    impl std::fmt::Display for MyVec {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "MyVec({} bytes)", self.0.len())
        }
    }
    assert_eq!(MyVec(vec![1,2,3]).to_string(), "MyVec(3 bytes)");
```

## Errors That Carry Information

Rust has no exceptions. `Result<T, E>` makes failure part of the type. The design question is **what shape should `E` have?**

### Library vs Application

| | Library | Application (binary) |
|-|---------|----------------------|
| Error type | Concrete enum (`thiserror`) | Dynamic (`anyhow::Error`) |
| Caller can match on variants? | **Yes** — that's the point | No |
| Add a dependency? | `thiserror` | `anyhow` |
| Use case | Reusable crate | CLI, server, prototype |

**Rule: libraries define typed errors; applications erase them.** Never put `anyhow::Error` in a public library API — you are forcing your callers to `Box::dyn` their way through, and they cannot handle your variants.

### thiserror for Libraries

```rust,linenos
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum ConfigError {
        #[error("could not read {path}")]
        Io {
            path: String,
            #[source]                     // marks the underlying cause
            source: std::io::Error,
        },

        #[error("invalid value for `{key}`: {reason}")]
        Invalid { key: String, reason: String },

        #[error("missing required field `{0}`")]
        Missing(String),
    }

    fn load(path: &str) -> Result<Config, ConfigError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|source| ConfigError::Io { path: path.into(), source })?;
        Ok(Config { name: raw })
    }

    struct Config { name: String }
```

The `#[from]` attribute generates a `From` impl automatically:

```rust,linenos
    #[derive(Debug, thiserror::Error)]
    enum StoreError {
        #[error("io failure")]
        Io(#[from] std::io::Error),      // #[from] == #[source] + From impl

        #[error("db: {0}")]
        Db(String),
    }

    fn save() -> Result<(), StoreError> {
        std::fs::write("x", b"y")?;       // io::Error converts automatically
        Ok(())
    }
```

### anyhow for Applications

`anyhow` gives you a type-erased error with `.context()` for adding a human-readable message as it propagates.

```rust,linenos
    use anyhow::{Context, Result, bail, ensure};

    fn load_config(path: &str) -> Result<String> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config at {path}"))?;   // adds context
        ensure!(!raw.trim().is_empty(), "config file {path} is empty");     // early return
        Ok(raw)
    }

    // bail! is `return Err(anyhow!("..."))`
    fn validate(s: &str) -> Result<()> {
        if s.len() > 100 { bail!("input too long: {} bytes", s.len()); }
        Ok(())
    }
```

The **context chain** is the real value. A bare `No such file or directory` is useless; `failed to read config at ./prod.yaml` tells you what failed and where.

```rust,linenos
    // Printing an anyhow error shows the full chain:
    // Error: failed to read config at ./prod.yaml
    //
    // Caused by:
    //     No such file or directory (os error 2)
```

### The `?` Operator and Conversion

`?` does three things: unwrap `Ok`, return the `Err`, and **convert the error type** if `From` exists.

```rust,linenos
    fn parse_and_save() -> Result<(), StoreError> {
        let n: u32 = "42".parse().map_err(|_| StoreError::Db("bad int".into()))?;
        std::fs::write("f", n.to_string())?;     // io::Error -> StoreError via #[from]
        Ok(())
    }

    // In a function returning Option, ? works too — it returns None early.
    fn first_word_opt(s: &str) -> Option<&str> {
        let t = s.split(' ').next()?;     // returns None if no space
        Some(t)
    }
```

> **The `#` operator** (`Err(#err_expr)`) inside a `Result`/`Option` block lets you match and immediately return a transformed error. Rare, occasionally clearer than a full `match`.

### Panic vs Error

| | `panic!` / `unwrap()` | `Result` / `?` |
|-|----------------------|----------------|
| Meaning | Programmer bug / broken invariant | Expected, recoverable failure |
| Example | Array index assumes a boundary that a test already proved | File not found, network down |
| Recovery | Unwind, or abort; caller can't handle | Caller matches and handles |
| Use in | `main`, tests, internal invariants | Almost everything else |

```rust,linenos
    // Unwrap is FINE for values that cannot fail by construction:
    let x: i32 = "42".parse().unwrap();         // literal — always succeeds
    let v: Vec<i32> = Vec::new();
    let first = v.first();                      // Option, handled below

    // Unwrap is WRONG for runtime input:
    fn port_from_env() -> u16 {
        std::env::var("PORT").unwrap().parse().unwrap()   // two panics waiting to happen
    }
```

**Use `expect` instead of `unwrap` in library code.** The message documents the invariant:

```rust,linenos
    let config = std::fs::read_to_string("app.toml")
        .expect("app.toml must exist next to the binary; run from the project root");
```

### Never let `unwrap` Reach Production

The practical rules:

1. **Clippy is your friend:** `#![deny(clippy::unwrap_used, clippy::expect_used)]` at the crate root turns every one into a compile error in library code.
2. **In tests and examples,** `unwrap()` is encouraged — it is clearer than `.expect("...")` for a value you constructed yourself.
3. **In `main`,** returning `Result` from main is fine: `fn main() -> Result<(), Box<dyn Error>>`.

## Iterators & Closures

Iterators are Rust's main tool for expressing transformations without loops. They are **lazy** by default: nothing runs until you consume the iterator.

### Lazy vs Eager

```rust,linenos
    let v = vec![1, 2, 3, 4, 5];

    // Lazy — adaptor chain, nothing executed yet
    let squares = v.iter().map(|x| x * x).filter(|x| *x % 2 == 1);

    // Consumed now. collect() forces evaluation.
    let out: Vec<i32> = squares.collect();
    assert_eq!(out, vec![1, 9, 25]);

    // Nothing ran above except building the chain. Forgetting to consume
    // an iterator is a (silent) no-op — use `.for_each()` or `collect()`
    // to actually execute it.
```

### The Adaptor Catalogue

The 20 adaptors that cover ~95% of real use:

| Category | Adaptors |
|----------|----------|
| **Transform** | `map`, `filter`, `filter_map`, `flat_map`, `take_while`, `skip_while`, `step_by`, `zip`, `chain`, `scan`, `flatten` |
| **Consume** | `collect`, `sum`, `product`, `count`, `fold`, `reduce`, `for_each`, `last`, `nth`, `all`, `any`, `position`, `find`, `min_by`, `max_by`, `unzip` |
| **Adapt** | `take`, `skip`, `rev`, `cloned`, `copied`, `inspect`, `by_ref`, `peekable`, `dedup`, `unique`, `chunks`, `windows` |
| **Combine** | `zip` |

```rust,linenos
    let words = vec!["apple", "banana", "cherry", "date", "elderberry"];

    // Long words, sorted by length descending, first three:
    let top: Vec<&str> = words.iter()
        .filter(|w| w.len() > 4)
        .sorted_by_key(|w| std::cmp::Reverse(w.len()))  // reverse sort
        .take(3)
        .copied()
        .collect();
    assert_eq!(top, vec!["elderberry", "banana", "cherry"]);

    // flat_map: one-to-many expansion (iter + collect::&lt;Vec&lt;_&gt;&gt; chained)
    let letters: Vec<char> = words[0].chars().collect();
    let total_len: usize = words.iter().map(|w| w.chars().count()).sum();
    assert_eq!(total_len, 5 + 6 + 6 + 4 + 11);
```

### Grouping Without itertools

The "group consecutive equal elements" problem is common enough that `itertools` (`Itertools::chunk_by`) exists. Here is the plain-std version:

```rust,linenos
    fn group_consecutive<T: PartialEq, I: IntoIterator<Item = T>>(it: I) -> Vec<Vec<T>> {
        let mut out: Vec<Vec<T>> = vec![];
        for v in it {
            match out.last_mut() {
                Some(g) if g.last() == Some(&v) => g.push(v),   // extends current run
                _ => out.push(vec![v]),                        // starts a new one
            }
        }
        out
    }

    assert_eq!(group_consecutive(vec![1,1,2,2,2,3]), vec![vec![1,1], vec![2,2,2], vec![3]]);
```

> **A note on `itertools`:** it's a mature, ubiquitous crate (1.6B+ downloads) that fills real gaps in std — `chunk_by`, `exactly_one`, `Itertools::join`, `multipeek`, `Itertools::into_group_map`. But for the top 40% of cases, std is enough. Reach for `itertools` when you need `chunk_by`, `izip!`, or `Either`; otherwise prefer std so your code has fewer dependencies.

### `fold` vs `for`

Use `for` when you just want to iterate. Use `fold` when you're building an accumulator.

```rust,linenos
    // for: clearest for a simple loop
    let mut total = 0;
    for n in [1, 2, 3] { total += n; }

    // fold: clearest when accumulating into a non-numeric type
    let joined = ["a", "b", "c"]
        .iter()
        .fold(String::new(), |mut acc, s| { acc.push_str(s); acc });
    assert_eq!(joined, "abc");
```

### Custom Iterators

Implement `Iterator` to build reusable abstractions. Only `next()` is required.

```rust,linenos
    struct Evens { n: u64 }
    impl Iterator for Evens {
        type Item = u64;
        fn next(&mut self) -> Option<u64> {
            let r = self.n;
            self.n += 2;
            (r < 10).then_some(r)     // then_some instead of if/else
        }
    }

    assert_eq!(Evens { n: 0 }.collect::<Vec<_>>(), vec![0, 2, 4, 6, 8]);
```

Now every adaptor works on it, for free:

```rust,linenos
    let sum: u64 = Evens { n: 0 }.filter(|x| x % 4 == 0).sum();
    assert_eq!(sum, 12);      // 4 + 8
```

### `.iter()` vs `.into_iter()` vs `.iter_mut()`

| Method | Yields | Moves the collection? |
|--------|--------|----------------------|
| `.iter()` | `&T` | No |
| `.iter_mut()` | `&mut T` | No |
| `.into_iter()` | `T` | **Yes** — consumes |

```rust,linenos
    let v = vec![String::from("a"), String::from("b")];

    let lens: Vec<usize> = v.iter().map(|s| s.len()).collect();      // borrows
    let owned: Vec<String> = v.clone().into_iter().map(|s| s + "!").collect(); // moves the clone
    let mut upper = vec![String::from("a")];
    for s in upper.iter_mut() { *s = s.to_uppercase(); }             // mutates in place
    assert_eq!(upper, vec!["A"]);
    assert_eq!(lens, vec![1, 1]);
```

> **`v.into_iter()` on a `Vec<T>` moves the elements** — a genuinely cheap operation (it's just taking the buffer), no cloning involved. This is the idiomatic way to consume a collection.

## Smart Pointers & Interior Mutability

A smart pointer is a struct that owns data **and** adds semantics. Each one answers a different question.

### The Pointer Family

| Type | Question it answers | Cost |
|------|-------------------|------|
| `Box<T>` | "I need a fixed-size handle to heap data" | one alloc |
| `Rc<T>` | "Several owners, one thread" | refcount (non-atomic) |
| `Arc<T>` | "Several owners, many threads" | refcount (atomic) |
| `RefCell<T>` | "Mutate through a shared reference" | runtime borrow check |
| `Cell<T>` | "Mutate through a shared reference, move-only data" | no check, just `get`/`set` |
| `Cow<T>` | "Usually borrowed, sometimes owned" | allocation only when needed |
| `Pin<Box<T>>` | "This value must not move (self-referential)" | heap box |
| `Weak<T>` | "A non-owning reference to `Rc`/`Arc` data" | — |

### `Box<T>` — The Default Indirection

`Box` is a heap allocation with exactly one owner. You reach for it in three cases: recursive types, very large values moved around, or trait objects.

```rust,linenos
    // 1. Recursive — impossible without indirection (infinite size)
    enum Tree {
        Leaf,
        Node(Box<Tree>, i32, Box<Tree>),
    }

    // 2. Trait objects
    let handlers: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),
        Box::new(|x| x * 2),
    ];
    assert_eq!(handlers[0](10), 11);

    // 3. Deliberately-large, move-rarely values (e.g. a big buffer you pass by &)
    let big = Box::new([0u8; 1_000_000]);
    assert_eq!(big.len(), 1_000_000);
```

> `Vec<T>` is already a `Box`-like handle (it owns a heap buffer). `Box<Vec<T>>` is almost always a mistake — it adds a pointer hop for no reason.

### `Rc` vs `Arc`

Identical APIs; the difference is atomic vs non-atomic refcounting. `Rc` is cheaper but **cannot cross threads**. `Arc` can, at the cost of atomic operations on every clone/drop.

```rust,linenos
    use std::rc::Rc;
    use std::sync::Arc;

    // Single-threaded shared ownership
    let data = Rc::new(vec![1, 2, 3]);
    let a = Rc::clone(&data);        // refcount++, SAME allocation
    assert_eq!(Rc::strong_count(&data), 2);
    drop(a);
    assert_eq!(Rc::strong_count(&data), 1);

    // Cross-thread shared ownership
    let shared = Arc::new(String::from("hello"));
    let handles: Vec<_> = (0..4).map(|_| {
        let s = Arc::clone(&shared);
        std::thread::spawn(move || s.len())
    }).collect();
    let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    assert_eq!(total, 20);
```

**Always use `Arc::clone(&x)` rather than `x.clone()`** — same result, but the explicit form signals to readers (and to clippy) that you're bumping a refcount, not deep-copying.

### `Rc<RefCell<T>>` vs `Arc<Mutex<T>>`

This is the classic shared-mutable-state question. Both give shared ownership plus interior mutability; the runtime mechanism differs.

```rust,linenos
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    // SINGLE THREAD: RefCell — borrow checked at RUNTIME, no OS locking
    let data = RefCell::new(vec![1, 2, 3]);
    data.borrow_mut().push(4);            // runtime-checked mutable borrow
    assert_eq!(data.borrow().len(), 4);

    // MULTIPLE THREADS: Mutex — OS-level lock, poisons on panic
    let counter = Arc::new(Mutex::new(0u64));
    {
        let mut c = counter.lock().unwrap();     // guard releases on drop
        *c += 1;
    }   // lock released HERE automatically
    assert_eq!(*counter.lock().unwrap(), 1);
```

| | `RefCell` | `Mutex` |
|-|-----------|---------|
| Threads | One | Many |
| Enforcement | Runtime borrow check | OS lock |
| Cost | Almost zero | Syscall (uncontended: ~20ns) |
| Deadlock possible | Can panic (double borrow) | **Yes — can hang** |
| Poisoning | No | Yes (a panic while holding it) |

> **`RefCell` panics on overlapping borrows at runtime.** The compiler checked the types but not the timing. If you see `already borrowed` at runtime, a reference is held across a mutation. Restructure to shorten the borrow, not by using `try_borrow_mut` in a loop.

### `Cell<T>` — Mutate-Only, No References

`Cell` skips the runtime borrow check by requiring `get`/`set` (a move in, a move out) rather than handing out references. Faster and simpler for Copy types and small values.

```rust,linenos
    use std::cell::Cell;

    let count = Cell::new(0);
    count.set(count.get() + 1);      // read + write, no borrow held
    count.set(count.get() + 1);
    assert_eq!(count.get(), 2);

    // Works inside a shared reference, so a struct with &self methods can count:
    struct Hits(Cell<u64>);
    impl Hits {
        fn hit(&self) { self.0.set(self.0.get() + 1); }   // &self, mutates anyway
    }
    let h = Hits::default();
    h.hit();
    assert_eq!(h.0.get(), 1);
```

| | `Cell<T>` | `RefCell<T>` |
|-|----------|-------------|
| Access | `get()` / `set()` (moves) | `borrow()` / `borrow_mut()` (references) |
| Runtime check | None | Yes — panics on conflict |
| Requires | `T: Copy` (for `get`) | nothing |
| Use for | Counters, flags, small Copy values | Anything you need a reference to |

### `Weak<T>` — Breaking Cycles

`Rc`/`Arc` counts strong references. A cycle (`A` holds `Arc` to `B`, `B` holds `Arc` to `A`) means the count never hits zero and the memory leaks. `Weak` doesn't count, so it breaks cycles.

```rust,linenos
    use std::rc::{Rc, Weak};

    let strong = Rc::new(1);
    let weak = Rc::downgrade(&strong);       // Weak<i32>, no ownership

    assert_eq!(weak.upgrade().map(|r| *r), Some(1));   // upgrade() -> Option<Rc>
    drop(strong);
    assert!(weak.upgrade().is_none());                 // data is gone, no leak
```

**Canonical use:** a parent with many children. The parent owns the children (strong), the children reference the parent (weak), so dropping the parent actually frees everything.

### `Pin` — When Self-Reference Forces the Heap

`Pin<T>` promises "this value will not be moved again." Most of the time you only care about the type `Pin<Box<T>>`, because the `Box` gives a **stable address** — the heap location never changes, so `self-referential` structs become possible.

```rust,linenos
    use std::pin::Pin;
    use std::boxed::Box;

    // A self-referential struct: borrows from its own field.
    // Impossible on the stack, possible in a Box because the address is stable.
    struct SelfRef {
        data: String,
        slice_ptr: *const str,       // would dangle if `data` ever moved
    }

    impl SelfRef {
        fn new(s: &str) -> Pin<Box<SelfRef>> {
            let mut boxed = Box::new(SelfRef { data: s.to_string(), slice_ptr: std::ptr::null() });
            let ptr = boxed.data.as_str();
            unsafe { boxed.slice_ptr = ptr; }     // set after the Box is stable
            Box::pin(boxed)                        // Pin<Box<SelfRef>>
        }
    }

    let sr = SelfRef::new("hello");
    assert_eq!(sr.data, "hello");
```

> **Do not write `unsafe` pointers yourself.** `Pin` exists because async state machines and self-referential types genuinely need it. `Box::pin` is the safe entry point. If you find yourself hand-rolling `*const` fields, you almost certainly want `self_cell`, `ouroboros`, or just a different design.

## Concurrency

Rust makes **data races impossible to compile**. It does not prevent deadlocks, logic races, or deadlocks — those remain your problem.

### `Send` and `Sync`

Two auto-trait markers the compiler checks:

- **`Send`** — safe to move to another thread. (`String`, `Vec`, most types: yes. `Rc`: no.)
- **`Sync`** — safe to share by reference across threads. (`&T: Send` when `T: Sync`.)

```rust,linenos
    use std::sync::Arc;
    use std::thread;

    // Rc is NOT Send — this would not compile:
    // let rc = std::rc::Rc::new(5);
    // thread::spawn(move || println!("{}", rc));   // Rc<Vec<i32>> cannot be sent

    // Arc is Send + Sync:
    let data = Arc::new(vec![1, 2, 3]);
    let handle = {
        let d = Arc::clone(&data);
        thread::spawn(move || d.iter().sum::<i32>())    // sends an Arc clone
    };
    assert_eq!(handle.join().unwrap(), 6);
```

### Threads: `join` and Scoping

```rust,linenos
    use std::thread;

    // Spawn with a join handle to collect the result
    let handle = thread::spawn(|| 2 + 2);
    assert_eq!(handle.join().unwrap(), 4);

    // Spawn MANY, collect handles, then join in order
    let handles: Vec<_> = (0..8)
        .map(|i| thread::spawn(move || i * i))
        .collect();
    let squares: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(squares, vec![0, 1, 4, 9, 16, 25, 36, 49]);
```

### Scoped Threads: Borrow Without `Arc`

`std::thread::scope` (stable since 1.63) lets spawned threads **borrow** stack data. No `Arc`, no `'static` requirement, no clone.

```rust,linenos
    fn parallel_sum(v: &[u32]) -> u32 {
        std::thread::scope(|s| {
            let mid = v.len() / 2;
            let (a, b) = v.split_at(mid);
            let h1 = s.spawn(|| a.iter().sum::<u32>());   // borrows `a`
            let h2 = s.spawn(|| b.iter().sum::<u32>());
            h1.join().unwrap() + h2.join().unwrap()      // scope guarantees completion
        })
    }

    assert_eq!(parallel_sum(&[1, 2, 3, 4]), 10);
```

**This is the idiomatic way to parallelize a function over a slice** with zero ceremony. The scope guarantees all threads finish before returning.

### Rayon: Data Parallelism Without Threads

`rayon` turns sequential iterator chains into parallel ones with one trait: `.par_iter()` instead of `.iter()`.

```rust,linenos
    use rayon::prelude::*;

    let v: Vec<u64> = (0..100_000u64).collect();

    // Sequential
    let total: u64 = v.iter().sum();

    // Parallel — same chain shape, ~all cores
    let ptotal: u64 = v.par_iter().sum();

    // Parallel map + filter + collect
    let evens: Vec<u64> = v.par_iter().copied().filter(|x| x % 2 == 0).collect();

    assert_eq!(total, ptotal);
    assert_eq!(evens.len(), 50_000);
```

**When Rayon is the right tool:** CPU-bound work over a large collection where the per-item cost is nontrivial. For tiny collections, Rayon's overhead exceeds the gain — use sequential.

### Channels: Message Passing

```rust,linenos
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel::<u32>();       // unbounded
    let tx2 = tx.clone();                        // multi-producer

    thread::spawn(move || tx.send(1).unwrap());
    thread::spawn(move || tx2.send(2).unwrap());
    drop(tx);                                    // drop original sender

    let sum: u32 = rx.iter().sum();              // ends when all senders drop
    assert_eq!(sum, 3);

    // Bounded (backpressure) — send blocks when full
    let (btx, brx) = mpsc::sync_channel::<u32>(2);
    btx.send(10).unwrap();
    assert_eq!(brx.recv().unwrap(), 10);
```

**Modern alternative:** `crossbeam-channel` (3x more downloads, better API: `select`, `after`, MPMC). For simple cases std `mpsc` is fine.

### Shared State: `Arc<Mutex<T>>`

```rust,linenos
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0u64));
    let handles: Vec<_> = (0..10).map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || {
            // Lock for as short a scope as possible
            let mut n = c.lock().unwrap();
            *n += 1;
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
    assert_eq!(*counter.lock().unwrap(), 10);
```

> **Lock as little as possible.** Hold the guard across only the mutation, never across `.await`, never across an expensive computation, never while calling out to code that might re-lock. A common bug is holding a `MutexGuard` across an `.await` in async code — that blocks the whole executor thread and is a deadlock waiting to happen.

### Locking Rules (Deadlock Prevention)

- **One lock at a time.** Never hold two mutexes at once in a fixed order; it deadlocks if another thread takes them in reverse.
- **Keep guards tiny.** Acquire, mutate, drop.
- **Never hold across `.await`.** Copy what you need out of the guard, drop the guard, *then* await.

```rust,linenos
    use std::sync::Mutex;

    let m = Mutex::new(vec![1, 2, 3]);
    let snapshot: Vec<i32> = { let g = m.lock().unwrap(); g.clone() };   // copy, then guard drops
    drop(snapshot);   // safe to await here — no guard held
```

## Async Rust

Async is Rust's model for **I/O-bound** concurrency: thousands of connections, few threads. It is not about making code run in parallel on multiple cores.

### Async Is Not Multithreading

| | Async | Threads |
|-|-------|---------|
| Concurrency | Many tasks, interleaved | Many OS threads |
| Parallelism | Only if the runtime has multiple worker threads | Yes, inherently |
| Best for | I/O wait (network, DB, disk) | CPU-bound work |
| Cost per unit | Very cheap (~few hundred bytes) | Expensive (~8MB stack) |
| Runtime needed | Yes (`tokio`) | No |

> **The rule:** `async` for I/O, threads for CPU. Mixing them is fine (`spawn_blocking` bridges them), but don't use async for pure computation — you'll just make it slower and more complex.

### Futures Are Lazy

```rust,linenos
    async fn fetch(id: u32) -> String { format!("item-{id}") }

    // Nothing runs until you await or spawn it:
    let fut = fetch(1);          // just a value describing future work
    let s = fut.await;           // NOW it runs
    assert_eq!(s, "item-1");

    // tokio::join! runs two futures concurrently on ONE thread:
    let (a, b) = tokio::join!(fetch(2), fetch(3));
    assert_eq!((a.as_str(), b.as_str()), ("item-2", "item-3"));
```

### Spawning Tasks

```rust,linenos
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[tokio::main]
    async fn main() {
        let counter = Arc::new(AtomicU64::new(0));

        let handles: Vec<_> = (0..10u64).map(|i| {
            let c = Arc::clone(&counter);
            tokio::spawn(async move {
                c.fetch_add(1, Ordering::SeqCst);
                i * 2
            })                       // runs on the runtime's thread pool
        }).collect();

        let results: Vec<u64> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(counter.load(Ordering::SeqCst), 10);
        assert_eq!(results.iter().sum::<u64>(), 90);
    }
```

### `select!` and Cancellation Safety

`select!` races multiple futures and takes the first to complete. The critical property: **cancellation safety** — whether dropping the losing futures loses data.

```rust,linenos
    use std::time::Duration;

    async fn with_timeout() -> Option<&'static str> {
        let work = async { "done" };
        tokio::select! {
            r = work => Some(r),                              // work finished first
            _ = tokio::time::sleep(Duration::from_secs(5)) => None,   // timed out
        }
    }

    #[tokio::main]
    async fn main() {
        assert_eq!(with_timeout().await, Some("done"));
    }
```

| | Cancellation safe | Notes |
|-|------------------|-------|
| `sleep`, `sleep_until` | **Yes** | Always safe |
| `tokio::io::read`/`write` | Depends | Partial reads can be lost |
| `mpsc::recv` | **No** | A received message is consumed even if the branch loses |
| `Mutex::lock` | **Yes** | Never held after drop |

**Rule: if losing a `select!` branch could lose data, that branch is not cancellation-safe — restructure.**

### The Blocking Trap

**Never block a tokio worker thread.** A blocking call inside an async task stalls the entire executor.

```rust,linenos
    #[tokio::main]
    async fn main() {
        // BAD: std::thread::sleep and std::fs::read block the worker thread.
        // GOOD: tokio's async equivalents
        let _ = tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        // For unavoidable blocking work, offload it:
        let result = tokio::task::spawn_blocking(|| {
            std::fs::read_to_string("/etc/hostname")   // blocking I/O, on a blocking pool
        }).await.unwrap();
        let _ = result;
    }
```

### Structured Concurrency: Scope Your Concurrency

The `async-scoped` / `tokio::task::scope` pattern makes concurrency structured and avoids `'static` requirements:

```rust,linenos
    #[tokio::main]
    async fn main() {
        // Sequential but structured — simple and correct:
        let mut total = 0;
        for id in 1..=5u64 {
            total += fetch_double(id).await;
        }
        assert_eq!(total, 30);
    }

    async fn fetch_double(id: u64) -> u64 { id * 2 }
```

**Practical guidance:** start with sequential `await`. Add `join!`/`spawn` only when you've measured that concurrency helps and you understand the cancellation semantics.

## The Top 40% of `std`

The standard library is enormous. These are the modules you will reach for in roughly 80% of real code. Everything else is a search away.

### Module Map

| Module | What it covers | Frequency |
|--------|---------------|-----------|
| `std::collections` | `HashMap`, `BTreeMap`, `Vec`, `VecDeque`, `BTreeSet`, `HashSet`, `BinaryHeap`, `LinkedList` | ★★★★★ |
| `std::io` | Reading/writing bytes, `BufReader`, `BufWriter` | ★★★★★ |
| `std::fs` | Files, dirs, paths, metadata | ★★★★★ |
| `std::path` | `Path`, `PathBuf` (filesystem-safe path handling) | ★★★★★ |
| `std::fmt` | `format!`, `Display`, `Debug`, `write!` | ★★★★★ |
| `std::iter` | `Iterator` and all its adaptors | ★★★★★ |
| `std::str`, `std::string` | `&str`, `String`, parsing | ★★★★★ |
| `std::sync` | `Arc`, `Mutex`, `RwLock`, `atomic`, `mpsc`, `OnceLock`, `Condvar` | ★★★★☆ |
| `std::time` | `Duration`, `Instant`, `SystemTime` | ★★★★☆ |
| `std::thread` | `spawn`, `scope`, `JoinHandle` | ★★★★☆ |
| `std::cmp` | `Ordering`, `min`, `max`, `Reverse`, `Ord` | ★★★★☆ |
| `std::convert` | `From`, `Into`, `TryFrom`, `TryInto`, `AsRef`, `AsMut` | ★★★★☆ |
| `std::borrow` | `Cow`, `Borrow`, `ToOwned` | ★★★★☆ |
| `std::boxed` | `Box` | ★★★★☆ |
| `std::rc` | `Rc`, `Weak` | ★★★☆☆ |
| `std::mem` | `swap`, `replace`, `size_of`, `take` | ★★★☆☆ |
| `std::process` | `Command`, `exit`, env vars | ★★★☆☆ |
| `std::env` | `args`, `var`, `vars` | ★★★☆☆ |
| `std::error` | `Error` trait, custom errors | ★★★☆☆ |
| `std::num` | `NonZero*`, `Wrapping`, `Saturating`, checked arithmetic | ★★★☆☆ |
| `std::char`, `std::ascii` | Unicode-aware classification | ★★☆☆☆ |
| `std::net` | `TcpListener`, `TcpStream`, `UdpSocket` | ★★☆☆☆ |
| `std::cell` | `Cell`, `RefCell`, `OnceCell` | ★★☆☆☆ |
| `std::slice` | `sort`, `binary_search`, `split_at`, `chunks`, `windows` | ★★☆☆☆ |
| `std::task` | `Poll`, `Context`, `Waker` | ★★☆☆☆ |

### Collections Cheatsheet

```rust,linenos
    use std::collections::{HashMap, BTreeMap, HashSet, VecDeque, BinaryHeap};

    // Vec: the default. push/pop O(1) amortized, index O(1), insert middle O(n)
    let mut v: Vec<i32> = vec![3, 1, 2];
    v.push(4); v.pop(); v.insert(0, 0); v.remove(0);
    v.sort(); v.sort_by(|a, b| b.cmp(a));            // descending
    v.sort_unstable();                              // faster, no stability, less memory
    v.dedup();                                      // removes CONSECUTIVE dups (sort first)
    v.retain(|x| *x > 1);                           // in-place filter
    assert_eq!(v, vec![1, 2, 3]);

    // HashMap: O(1) average get/insert/remove
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("a".into(), 1);
    m.entry("b".into()).or_insert(2);
    let keys: Vec<&String> = m.keys().collect();
    assert_eq!(m.len(), 2);

    // BTreeMap: sorted, O(log n), supports range queries
    let mut b: BTreeMap<i32, &str> = BTreeMap::new();
    b.insert(3, "c"); b.insert(1, "a"); b.insert(2, "b");
    let in_range: Vec<(&i32, &&str)> = b.range(1..=2).collect();
    assert_eq!(in_range.len(), 2);

    // HashSet: membership
    let s: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();
    assert_eq!(s.len(), 3);

    // VecDeque: FIFO queue, O(1) at both ends
    let mut d = VecDeque::new();
    d.push_back(1); d.push_front(0);
    assert_eq!(d.pop_front(), Some(0));

    // BinaryHeap: max-heap
    let mut h = BinaryHeap::new();
    h.push(3); h.push(5); h.push(1);
    assert_eq!(h.pop(), Some(5));
```

### `Vec` Operations You Probably Don't Know

```rust,linenos
    let v = vec![1, 2, 3, 4, 5];

    // Slicing without copying
    assert_eq!(&v[1..3], &[2, 3]);           // [i32]
    assert_eq!(&v[..2], &[1, 2]);
    assert_eq!(&v[3..], &[4, 5]);

    // Split into two
    let (a, b) = v.split_at(2);
    assert_eq!((a, b), (&[1, 2], &[3, 4, 5]));

    // Consecutive windows (overlapping) — pairs, triples, etc.
    let pairs: Vec<(i32, i32)> = v.windows(2).map(|w| (w[0], w[1])).collect();
    assert_eq!(pairs, vec![(1,2),(2,3),(3,4),(4,5)]);

    // Non-overlapping chunks
    let chunks: Vec<&[i32]> = v.chunks(2).collect();
    assert_eq!(chunks, vec![&[1,2], &[3,4], &[5]]);

    // Binary search — requires SORTED
    let mut sorted = vec![1, 3, 5, 7];
    assert_eq!(sorted.binary_search(&5), Ok(2));

    // Flatten nested vec
    let nested = vec![vec![1, 2], vec![3]];
    let flat: Vec<i32> = nested.into_iter().flatten().collect();
    assert_eq!(flat, vec![1, 2, 3]);

    // Swap two elements
    let mut s = vec![1, 2];
    s.swap(0, 1);
    assert_eq!(s, vec![2, 1]);
```

### `String` vs `&str` — Function Signature Guide

| You are... | Take | Because |
|-----------|------|---------|
| Reading only | `&str` | Works with literals, `String`, and slices; zero alloc |
| Reading, keeping it around | `&str` + a lifetime | Same |
| Transforming | `&str -> String` | The result is new; the input is just read |
| Storing in a struct | `String` | Needs to own |
| Generic over both | `impl AsRef<str>` or `&impl AsRef<str>` | Accepts `String`, `&str`, `Path`, `OsStr` |

```rust,linenos
    // Accept anything string-like, including String and &str:
    fn shout<S: AsRef<str>>(s: S) -> String { s.as_ref().to_uppercase() }

    assert_eq!(shout("hi"), "HI");
    assert_eq!(shout(String::from("hi")), "HI");

    // Generic over AsRef<[u8]> works with String, &str, Vec<u8>:
    fn hash<S: AsRef<[u8]>>(s: S) -> usize { s.as_ref().len() }
    assert_eq!(hash("abc"), 3);
```

### String Iteration and Slicing

```rust,linenos
    let s = "héllo";

    // .chars() — Unicode scalar values
    assert_eq!(s.chars().count(), 5);

    // .bytes() — UTF-8 bytes
    assert_eq!(s.bytes().count(), 6);      // é is 2 bytes

    // String indexing is BYTE-based and will panic on a non-boundary:
    // &s[0..1] is fine (h is 1 byte)
    // &s[0..2] PANICS — that would split é in half

    // Safe char-based slicing:
    let first_two: String = s.chars().take(2).collect();
    assert_eq!(first_two, "hé");

    // Or by grapheme (user-perceived character) — needs unicode-segmentation
```

### `Option` and `Result` — Full Toolkit

The combinators that eliminate almost all manual matching:

```rust,linenos
    let maybe: Option<i32> = Some(5);

    // Transform the contained value
    assert_eq!(maybe.map(|x| x * 2), Some(10));
    // Chain a fallible operation
    assert_eq!(maybe.and_then(|x| Some(x + 1)), Some(6));
    // Provide a default
    assert_eq!(maybe.unwrap_or(0), 5);
    assert_eq!(None::<i32>.unwrap_or_default(), 0);
    // Flatten nested options
    assert_eq!(Some(Some(1)).flatten(), Some(1));
    // Filter
    assert_eq!(maybe.filter(|x| *x > 10), None);
    // Get a default reference
    assert_eq!(maybe.as_ref().map(|x| *x), Some(5));

    // take() replaces the value in place — the &mut-Option trick
    let mut m = Some(5);
    let taken = m.take();
    assert_eq!((taken, m), (Some(5), None));

    let r: Result<i32, String> = Ok(5);
    assert_eq!(r.map(|x| x + 1).unwrap_or(0), 6);
    assert_eq!(r.map_err(|e| e.len()).unwrap(), 0);
```

### Numeric Operations

```rust,linenos
    let a: u32 = 10;

    // Checked — returns Option, panics on overflow in debug, wraps in release
    assert_eq!(a.checked_add(1), Some(11));
    assert_eq!(u32::MAX.checked_add(1), None);

    // Saturating — clamps at MIN/MAX, never panics
    assert_eq!(u32::MAX.saturating_add(1), u32::MAX);
    assert_eq!(0u32.saturating_sub(1), 0);

    // Wrapping — wraps modulo, explicit, never panics
    assert_eq!(u32::MAX.wrapping_add(1), 0);

    // NonZero — makes "never zero" part of the type
    use std::num::NonZeroU32;
    let n: NonZeroU32 = NonZeroU32::new(5).unwrap();
    assert_eq!(n.get(), 5);

    // Float comparison: no PartialEq tolerance built in
    fn approx_eq(a: f64, b: f64) -> bool { (a - b).abs() < f64::EPSILON }
    assert!(approx_eq!(0.1 + 0.2, 0.3));
```

### `Option<T>` / `Vec<T>`: Return Early With `?`

```rust,linenos
    // ? on Option: return None if the operand is None
    fn first_char_upper(s: &str) -> Option<char> {
        let c = s.chars().next()?;          // None if empty
        if c.is_alphabetic() { Some(c.to_ascii_uppercase()) } else { None }
    }

    assert_eq!(first_char_upper("apple"), Some('A'));
    assert_eq!(first_char_upper(""), None);
```

### Path Handling (Never Concatenate Strings)

```rust,linenos
    use std::path::{Path, PathBuf};

    // Path is the correct abstraction — handles separators, prefixes, encoding
    let p = Path::new("/tmp").join("file.txt");
    assert_eq!(p.file_name().unwrap(), "file.txt");
    assert_eq!(p.extension().unwrap(), "txt");
    assert_eq!(p.parent().unwrap(), Path::new("/tmp"));
    assert!(p.is_absolute());

    // Join multiple components safely
    let out: PathBuf = ["etc", "config", "app.toml"].iter().collect();
    assert_eq!(out, PathBuf::from("etc/config/app.toml"));

    // Reading files (handle errors, never unwrap):
    fn read(p: &Path) -> std::io::Result<String> { std::fs::read_to_string(p) }
```

### `std::mem` Utilities

```rust,linenos
    use std::mem;

    let mut v = vec![1, 2, 3];
    let old = mem::replace(&mut v, vec![9]);     // swap in new, return old
    assert_eq!(old, vec![1, 2, 3]);

    let taken = mem::take(&mut v);                // replace with Default
    assert_eq!((taken, v), (vec![9], vec![]));

    // size_of — know your layout
    assert_eq!(mem::size_of::<u32>(), 4);
    assert_eq!(mem::size_of::<String>(), 24);     // 3 words on 64-bit
    assert_eq!(mem::size_of::<&[i32]>(), 16);     // fat pointer: ptr + len
    assert_eq!(mem::size_of::<&i32>(), 8);        // thin pointer

    // swap two values in place
    let (mut a, mut b) = (1, 2);
    mem::swap(&mut a, &mut b);
    assert_eq!((a, b), (2, 1));
```

### File I/O with `BufReader`/`BufWriter`

```rust,linenos
    use std::io::{BufRead, BufReader, Write, Read};
    use std::fs::File;

    // Writing: ALWAYS buffer for anything but trivial sizes
    fn save(path: &str, data: &[u8]) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        let mut w = std::io::BufWriter::new(f);
        w.write_all(data)?;        // write_all, not write — write can be partial!
        w.flush()                   // must flush before the file is complete
    }

    // Reading lines with buffering
    fn count_lines(path: &str) -> std::io::Result<usize> {
        let f = File::open(path)?;
        BufReader::new(f).lines().count()
    }
```

> **`write` can do a partial write.** Always use `write_all` for correctness. Unbuffered per-byte writes on a file are catastrophically slow — always `BufWriter`.

## Macros & Derives

### `macro_rules!` — Declarative Macros

Match a pattern in the token stream and expand to code. Good for repeated syntax, bad for logic.

```rust,linenos
    macro_rules! my_vec {
        () => { Vec::new() };
        ($($x:expr),+ $(,)?) => { {
            let v = vec![$($x),+];
            v
        } };
    }

    let a = my_vec!();
    let b = my_vec![1, 2, 3,];      // trailing comma allowed
    assert_eq!(a.len(), 0);
    assert_eq!(b, vec![1, 2, 3]);
```

**When to use `macro_rules`:** small, compile-time, pattern-based. Not for anything with real control flow — that's what proc macros are for.

### Derive Macros You Should Use Constantly

```rust,linenos
    // The default derive set for any data type:
    #[derive(Debug, Clone, PartialEq, Default)]
    struct Point { x: i32, y: i32 }

    let a = Point { x: 1, y: 2 };
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(Point::default(), Point { x: 0, y: 0 });

    // Derive Ord for BTreeMap keys, sorting, min/max:
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Version(u32, u32);
    let mut versions = vec![Version(1, 2), Version(1, 0), Version(0, 9)];
    versions.sort();
    assert_eq!(versions[0], Version(0, 9));

    // Custom Debug (bypasses derive for this type)
    #[derive(Debug)]
    struct Secret(String);
    impl std::fmt::Debug for Secret {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Secret(<redacted>)")     // don't leak secrets in logs
        }
    }
```

**`#[derive]` order matters:** `Copy` requires `Clone` to also be derived. `Ord` requires `PartialOrd` and `Eq`. `Eq` requires `PartialEq`.

## Cargo & Project Structure

### The Commands That Matter

```bash,linenos
    cargo new myapp              # binary crate
    cargo new mylib --lib        # library crate

    cargo build                  # debug build (fast compile, slow binary)
    cargo build --release        # optimized (slow compile, fast binary)
    cargo run                    # build + run
    cargo run --release -- arg1  # pass args

    cargo check                  # type-check only, NO codegen — fastest feedback loop
    cargo clippy --all-targets   # lint (run this constantly)
    cargo fmt                    # format

    cargo test                   # run tests
    cargo test -- --nocapture    # show println! in passing tests
    cargo nextest run            # faster test runner (cargo-nextest)

    cargo add serde --features derive    # add a dependency
    cargo tree                    # visualize the dependency graph
    cargo tree -d                 # show DUPLICATED dependencies
    cargo outdated                # what has newer versions
    cargo audit                   # known security advisories (cargo-audit)
    cargo bloat                   # what is making the binary big
    cargo doc --open             # generate and open docs

    cargo install cargo-nextest  # recommended dev tools
    cargo install cargo-audit
    cargo install cargo-outdated
```

### Build Profiles

```toml,linenos
    [profile.release]
    opt-level = 3        # 0=none 1=basic 2=moderate 3=aggressive (default)
    lto = "thin"         # link-time optimization — smaller/faster, slower build
    codegen-units = 1    # single codegen unit — slower build, better code
    panic = "abort"      # smaller binary; no unwinding, no catch_unwind
    strip = true         # remove debug symbols
```

```toml,linenos
    [profile.dev]
    opt-level = 1        # compile slower, but dev builds run MUCH faster
                           # great for macro-heavy code (serde, diesel)
    debug = 1            # 0=none 1=line tables 2=full (default)
    overflow-checks = true   # catch integer overflow in debug (default on)
```

> **`debug-assertions` and `overflow-checks` are ON in dev, OFF in release.** Arithmetic overflow panics in dev but silently wraps in release. This is the single most common "works in dev, wrong in prod" Rust bug. Test with `--release`, or enable overflow checks in your release profile.

### Feature Flags

```toml,linenos
    [features]
    default = ["json"]           # what `cargo add` pulls in by default
    json = ["dep:serde_json"]    # dep: prefix = don't create an implicit feature
    postgres = ["dep:tokio-postgres", "dep:deadpool-postgres"]
    full = ["json", "postgres"]  # feature bundles
```

**Best practice:** prefix optional-dependency features with `dep:` so the implicit feature isn't created (this became the default in the 2021 edition / resolver v2).

### Workspace Layout

```bash,linenos
    my-project/
    ├── Cargo.toml              # [workspace] members = ["crates/*"]
    ├── crates/
    │   ├── myapp/              # the binary
    │   │   ├── Cargo.toml
    │   │   └── src/main.rs
    │   ├── mylib/              # reusable logic
    │   │   ├── Cargo.toml
    │   │   └── src/lib.rs
    │   └── mylib-derive/       # proc macro crate (must be a separate crate)
    └── Cargo.lock
```

A workspace shares one `Cargo.lock` and one `target/` — faster builds, one version of each dependency.

### Dependency Management Rules

- **Commit `Cargo.lock`** for binaries. For libraries, it's optional (but most do commit it now).
- **Use `[workspace.dependencies]`** to declare a version once and inherit it:
  ```toml,linenos
    # root Cargo.toml
    [workspace.dependencies]
    tokio = { version = "1", features = ["full"] }
    
    # crate Cargo.toml
    [dependencies]
    tokio.workspace = true
    ```
- **Run `cargo tree -d`** regularly — duplicate dependency versions bloat binaries.
- **`cargo audit`** in CI — pulls in known CVEs.

## Testing

### The Three Layers

```rust,linenos
    // 1. UNIT TESTS — in the same file, test private functions
    pub fn add(a: i32, b: i32) -> i32 { a + b }

    #[cfg(test)]
    mod tests {
        use super::*;              // import the parent module

        #[test]
        fn adds() { assert_eq!(add(2, 2), 4); }

        #[test]
        #[should_panic(expected = "overflow")]   // expect a specific panic message
        fn panics_on_overflow() { add(i32::MAX, 1); }
    }

    // 2. INTEGRATION TESTS — in tests/, use only the PUBLIC API
    // tests/api.rs
    #[test]
    fn public_api_works() { assert_eq!(mylib::add(1, 1), 2); }

    // 3. DOC TESTS — examples in /// comments are compiled and run
    /// Adds two numbers.
    ///
    /// ```
    /// let x = mylib::add(1, 2);
    /// assert_eq!(x, 3);
    /// ```
    pub fn documented(a: i32, b: i32) -> i32 { a + b }
```

**Test private functions with unit tests; test behavior with integration tests.** Integration tests prove your crate works for *users*, who can only touch the public API.

### Async Tests

```rust,linenos
    #[tokio::test]
    async fn test_async() {
        let v = async { 42 }.await;
        assert_eq!(v, 42);
    }
```

### Snapshot Testing

`insta` is the standard — it stores expected output and shows a diff when it changes.

```rust,linenos
    #[test]
    fn snapshot() {
        insta::assert_snapshot!(render_to_string(input));
        // First run: writes .snap files and fails.
        // Review with `cargo insta review`.
    }
```

Snapshot tests are ideal for anything with unstable output (formatted strings, JSON, rendered HTML) where writing assertions by hand is tedious.

### Property-Based Testing

`proptest` generates thousands of inputs and checks invariants. This finds bugs you would never think to test.

```rust,linenos
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_roundtrip(s in "\\PC{0,100}") {
            // If parsing succeeds, re-serializing must give the original back.
            if let Ok(parsed) = s.parse::<MyType>() {
                prop_assert_eq!(parsed.to_string(), s);
            }
        }

        #[test]
        fn sort_is_sorted(v in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut sorted = v.clone();
            sorted.sort();
            prop_assert!(sorted.windows(2).all(|w| w[0] <= w[1]));
        }
    }
```

## The Ecosystem: Crates That Will Still Be Here

**Data source:** crates.io API, retrieved **October 2026**. Versions are the current stable release at time of writing. "90d" is recent downloads — a proxy for current adoption.

### How to Judge Whether a Crate Will Survive

Do not trust star counts. Apply these five checks, in order:

| Check | Why it matters | Red flag |
|-------|---------------|----------|
| **1. Is it at 1.0+?** | Pre-1.0 crates make breaking releases freely. A 0.x crate used in production is a liability. | `0.x` with no path to 1.0 |
| **2. Download volume** | Millions of 90-day downloads means a large dependent base that *forces* maintenance. | < 50k recent downloads |
| **3. Maintainer count** | Single-maintainer crates can vanish when that person gets bored. | 1 contributor, no org backing |
| **4. Release cadence** | A crate last updated 2+ years ago in an active domain is unmaintained. | stale |
| **5. Ownership** | A Google/Microsoft/AWS/tokio-rs crate has institutional backing. A hobby crate may not. | personal repo, no org |

> **The honest truth about Rust crates:** there is no `1.0` guarantee like Python's backward-compat promise. SemVer is enforced *by the maintainers*, not by law. The only real protection is **adoption** — millions of dependents means breaking you breaks them, so they upgrade carefully.

### Tier 1: Foundations (Adopt Without Hesitation)

These have enormous dependent bases and long histories. Treat them as extensions of `std`.

| Crate | Version | 90d downloads | What it does |
|-------|---------|---------------|-------------|
| `serde` | 1.0 | **354M** | Serialization framework — the backbone of all Rust I/O |
| `serde_json` | 1.0 | **354M** | JSON format for serde |
| `tokio` | 1.53 | **254M** | The async runtime. If you do async, this is it |
| `clap` | 4.6 | **252M** | CLI argument parsing with derive macros |
| `anyhow` | 1.0 | **239M** | Application error handling |
| `futures` | 0.3 | **189M** | Future/stream combinators (`join_all`, `stream::iter`, …) |
| `regex` | 1.13 | **275M** | Regular expressions — no built-in alternative |
| `bytes` | 1.12 | **277M** | Zero-copy byte buffers — used by every network stack |
| `hyper` | 1.12 | **230M** | The HTTP/1+2 implementation under axum, reqwest, tonic |
| `itertools` | 0.15 | **372M** | Extra iterator adaptors (`chunk_by`, `exactly_one`, …) |
| `uuid` | 1.28 | **208M** | UUID v4/v7 generation and parsing |
| `chrono` | 0.4 | **197M** | Date/time. Mature, but see `time` below |
| `rand` | 0.10 | **502M** | Random number generation |

```toml,linenos
    [dependencies]
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    tokio = { version = "1", features = ["full"] }
    clap = { version = "4", features = ["derive"] }
    anyhow = "1"
    thiserror = "2"
    futures = "0.3"
```

### Tier 2: Error Handling and Logging

| Crate | Version | 90d downloads | Notes |
|-------|---------|---------------|-------|
| `thiserror` | 2.0 | **418M** | Derive error enums — **the standard for libraries** |
| `tracing` | 0.1 | **217M** | Structured, span-aware logging. Replaces `log` for anything serious |
| `tracing-subscriber` | 0.3 | **159M** | The subscriber that formats and emits `tracing` events |
| `log` | 0.4 | **318M** | Older, simpler. Fine for libraries that shouldn't pick a subscriber |

**`tracing` vs `log`:** `log` is level-based and global. `tracing` understands *spans* — it knows a request is one unit of work across many nested calls. For a web service or anything multi-threaded, `tracing` is strictly better.

```rust,linenos
    use tracing::{info, warn, error, instrument};

    #[instrument(skip(self))]        // auto-creates a span; skip avoids logging secrets
    fn process(&self, id: u64) {
        info!(user_id = id, "processing");     // structured fields, not string concat
        if id == 0 { warn!("suspicious id"); }
        error!(code = 500, "failed");
    }
```

### Tier 2: Web and HTTP

| Crate | Version | 90d downloads | Notes |
|-------|---------|---------------|-------|
| `axum` | 0.8 | **131M** | The mainstream web framework. Built on tokio + hyper + tower |
| `reqwest` | 0.13 | **211M** | HTTP client — the de facto standard |
| `tower` | 0.5 | **193M** | Service abstraction: middleware, retries, timeouts |
| `tower-http` | 0.7 | **171M** | Ready-made middleware (tracing, CORS, compression, auth) |
| `hyper` | 1.12 | **230M** | Low-level HTTP — you rarely touch this directly |

**`reqwest` version note:** the ecosystem has been migrating from 0.11 → 0.12 → 0.13. Check which major your transitive deps require before pinning.

```toml,linenos
    [dependencies]
    axum = { version = "0.8", features = ["macros", "multipart"] }
    tower = { version = "0.5", features = ["util"] }
    tower-http = { version = "0.7", features = ["trace", "cors", "limit"] }
    reqwest = { version = "0.13", features = ["json", "rustls-tls"], default-features = false }
```

> **`default-features = false`** on `reqwest` is worth knowing: the default pulls in OpenSSL bindings. Using `rustls-tls` instead gives you a pure-Rust TLS stack with no system dependency, which is usually what you want for portable deployments.

### The Tokio Stack: Toasty and Topcoat

The **tokio-rs** organization — Carl Lerche, author of Hyper, Tokio, and now Toasty — also maintains the most ambitious new application framework in Rust. This is the highest-conviction bet in the ecosystem.

| Crate | Version | 90d downloads | Status |
|-------|---------|---------------|--------|
| `topcoat` | 0.10 | **45k** | Experimental. Full-stack web framework |
| `toasty` | 0.11 | **44k** | Experimental. Async ORM |

**The case for them:** institutional backing from the team that built Tokio, Hyper, and the entire async Rust foundation. When they stabilize, they become the default. The author's stated goal is explicitly to replace the current fragmented web/ORM landscape.

**The case against — stated plainly:** both are **0.x and self-described as experimental**, with the maintainer writing that he expects "breaking changes" and "churn." 45k downloads is 4 orders of magnitude below `axum`. There is no compatibility promise at all.

**Recommendation:** treat both as the **future**, not the present.
- **For production code today:** `axum` + explicit SQL. Both are proven.
- **For learning / new projects with no deadline:** building on the Tokio stack is a reasonable bet on where the ecosystem converges.
- **Do not** build a system whose correctness depends on a 0.x API with no stability guarantee.

#### Toasty — the ORM

```rust,linenos
    use toasty::Model;

    #[derive(Debug, Model)]
    struct User {
        #[key]          // primary key
        #[auto]         // auto-increment
        id: u64,
        name: String,
        #[unique]
        email: String,
        #[has_many]
        todos: toasty::Deferred<Vec<Todo>>,
    }

    #[derive(Debug, Model)]
    struct Todo {
        #[key] #[auto]
        id: u64,
        #[index]         // database index on this column
        user_id: u64,
        #[belongs_to]
        user: toasty::Deferred<User>,
        title: String,
    }

    // Create with a nested graph in one round trip:
    async fn create(db: &mut Db) -> anyhow::Result<()> {
        let user = toasty::create!(User {
            name: "John Doe",
            email: "john@example.com",
            todos: [
                { title: "Make pizza" },
                { title: "Ship it" },
            ],
        }).exec(db).await?;
        Ok(())
    }

    // Query — only generates methods for indexed access patterns:
    async fn load(db: &mut Db, id: u64) -> anyhow::Result<User> {
        let user = User::get_by_id(db, &id).await?;
        let todos = user.todos().exec(db).await?;
        Ok(user)
    }
```

Toasty supports PostgreSQL, MySQL, SQLite, and DynamoDB, and explicitly does **not** try to abstract away the database's capabilities.

#### Topcoat — the full-stack framework

```rust,linenos
    use topcoat::{Result, router::{Router, RouterBuilderDiscoverExt, page}, view::{View, component, view}};

    #[tokio::main]
    async fn main() {
        topcoat::start(Router::builder().discover().build()).await.unwrap();
    }

    #[page("/")]
    async fn home() -> Result<impl View> {
        Ok(view! {
            <!DOCTYPE html>
            <html><body>
                hello(name: "World")
            </body></html>
        })
    }

    #[component]
    async fn hello(name: &str) -> Result<impl View> {
        Ok(view! { <h1>"Hello, " (name) "!"</h1> })
    }
```

**The design idea:** server-side rendering where a `$(...)` expression is ordinary type-checked Rust that also compiles to JavaScript. Client reactivity without a WASM bundle or a Node build step.

```rust,linenos
    #[component]
    async fn faq(cx: &Cx) -> Result<impl View> {
        let open = signal(cx, || false);        // state lives in the browser
        Ok(view! {
            <button @click=$(|_e| open.set(!open.get()))>"What is Topcoat?"</button>
            <p :hidden=$(!open.get())>"A full-stack Rust framework."</p>
        })
    }
```

It ships sessions with cookie-based auth, an asset pipeline with content-hashed URLs, Tailwind integration without Node, mail, WebSockets, and server-sent events.

### Tier 3: Specialized

| Crate | Version | 90d downloads | Use for |
|-------|---------|---------------|---------|
| `rayon` | 1.12 | **137M** | CPU data parallelism (`.par_iter()`) |
| `parking_lot` | 0.12 | **237M** | Faster mutexes than std (no poisoning) |
| `num_enum` | 0.7 | **60M** | Safe integer↔enum conversions |
| `strum` | 0.28 | **154M** | Enum → string, CLI parsing, iteration macros |
| `derive_more` | 2.1 | **130M** | A batteries-included alternative to `thiserror`/`strum` |
| `once_cell` | 1.21 | **311M** | Lazily-initialized statics (note: `std::sync::LazyLock` now covers this) |
| `indexmap` | 2.14 | **399M** | HashMap with insertion order |
| `arc-swap` | 1.9 | **94M** | Lock-free reads of an `Arc`-shared value |
| `criterion` | 0.8 | **64M** | Benchmarking |
| `proptest` | 1.12 | **56M** | Property-based testing |
| `insta` | 1.49 | **31M** | Snapshot testing |
| `mockall` | 0.15 | **30M** | Mocking trait impls for tests |
| `wiremock` | 0.6 | **26M** | HTTP mocking in tests |
| `tracing-subscriber` | 0.3 | **159M** | Logging setup |
| `console` | 0.16 | **88M** | Terminal colors and formatting |
| `indicatif` | 0.18 | **51M** | Progress bars |
| `dialoguer` | 0.12 | **18M** | Interactive CLI prompts |
| `dotenvy` | 0.15 | **53M** | Load `.env` files |
| `tempfile` | 3.27 | **205M** | Safe temporary files and directories |
| `sha2` / `blake3` | 0.11 / 1.8 | **297M / 50M** | Hashing. `blake3` is much faster |
| `secrecy` | 0.10 | **46M** | Prevent secrets from leaking into logs |
| `zeroize` | 1.9 | **201M** | Zero memory on drop (for key material) |

### Crates to Avoid or Use Carefully

| Crate | Why |
|-------|-----|
| **Pre-1.0 heavyweights** (`tokio-postgres` 0.7, `argon2` 0.6, `sha2` 0.11) | These *are* the standard, but note they are still 0.x — pin exact versions |
| `dotenvy` | Stale (last release 2023). `std::env` + a config crate is often better |
| `async-std` | Effectively unmaintained. tokio is the standard |
| `failure` | Deprecated. Use `anyhow` or `thiserror` |
| `structopt` | Deprecated. Use `clap` v4 directly |
| `rocket` | Stalled (last meaningful release 2024) |
| Any crate with < 10k recent downloads and no org backing | High abandonment risk |

### `time` vs `chrono`

Both are stable and heavily used. `chrono` has more downloads and more features; `time` has a cleaner API and better type safety. Neither is going away. Pick one and stay consistent — don't have both in your dependency tree.

## Best Practices: Do and Don't

### Always

| Practice | Why |
|----------|-----|
| `cargo clippy --all-targets -- -D warnings` | Catches real bugs, not just style |
| `cargo fmt` | Removes all formatting debates from code review |
| Run `cargo check` in a tight loop | 10x faster than `build` — no codegen |
| Prefer `&str` over `&String` in function params | More flexible, accepts literals too |
| Prefer `impl Trait` over `<T: Trait>` in params | Cleaner, equivalent |
| Return `impl Iterator<Item = T>` from helpers | Hides the concrete type, enables chaining |
| Use `?` with context | Errors should say *what* and *where* failed |
| `thiserror` in libraries, `anyhow` in binaries | Right tool for each audience |
| Derive `Debug` on everything | Makes types printable, testable, and debuggable |
| Handle errors at the boundary | `main` is where `?` finally gets resolved |
| Test the public API | That's what your users actually call |
| Pin dependencies in `Cargo.lock` | Reproducible builds |

### Never

| Anti-Pattern | Why It's Bad |
|-------------|-------------|
| `unwrap()` in library code | Turns a recoverable error into a crash for your users |
| `.unwrap()` in a loop | One failure kills the whole batch |
| `&Vec<T>` in a signature | Use `&[T]` — more flexible, same performance |
| `clone()` to silence a borrow error | A design smell in anything but a trivial small type |
| `unsafe` without a documented safety invariant | The compiler trusts you completely |
| Blocking I/O inside an async task | Stalls the entire tokio worker thread |
| Holding a `MutexGuard` across `.await` | Deadlock |
| Panicking in a library | Callers can't handle it |
| `mem::forget` on something you own | Leaks; almost always a bug |
| Trusting `HashMap` iteration order | Not stable across runs |
| `Box<dyn Error>` in a public API | Callers can't match on variants |
| Huge `match` with `_ => unreachable!()` | You added a variant and missed a case; `unreachable!` is a lie |
| `pub` on every function | Only expose what you want to be a stable commitment |

### Clippy Lints Worth Enforcing

```toml,linenos
    # Cargo.toml
    [lints.clippy]
    all = { level = "deny", priority = -1 }
    pedantic = { level = "warn", priority = -1 }

    # For libraries, forbid these outright:
    unwrap_used = "deny"
    expect_used = "deny"
    indexing_slicing = "warn"     # force .get() instead of [i]
    missing_errors_doc = "warn"   # document what can fail
    missing_panics_doc = "warn"   # document what can panic
```

```rust,linenos
    #![warn(clippy::all, clippy::pedantic)]
    #![forbid(unsafe_code)]        // libraries should not use unsafe
```

### Common Clippy Suggestions and What They Mean

| Clippy says | It means | Usually do this |
|-------------|----------|-----------------|
| `redundant_clone` | The clone is unnecessary | Delete it |
| `needless_borrow` | `&x` where `x` is already a reference | Remove the `&` |
| `ptr_arg` | `&Vec<T>` should be `&[T]` | Change the signature |
| `clone_on_copy` | Cloning a `Copy` type does nothing | Just copy it |
| `large_enum_variant` | One enum variant is huge | `Box` the big variant |
| `collapsible_if` | Nested `if` can be one `if a && b` | Combine |
| `manual_map`, `manual_filter` | You wrote a `match` that an adaptor replaces | Use the adaptor |
| `single_char_pattern` | `.find("x")` should be `.find('x')` | Use the char |
| `needless_return` | `return x;` at the end of a block | Just `x` |
| `type_complexity` | The type is unreadable | Introduce a type alias or newtype |

## Unsafe Rust

`unsafe` gives you five things the compiler otherwise forbids: dereferencing raw pointers, calling `unsafe` functions, accessing mutable statics, implementing unsafe traits, and accessing `union` fields.

### The Five Things `unsafe` Unlocks

```rust,linenos
    // 1. Dereference a raw pointer
    let x = 5;
    let p: *const i32 = &x;
    let val = unsafe { *p };               // unsafe: we promise p is valid

    // 2. Call an unsafe function
    let v = Vec::with_capacity(10);
    // std's unsafe internals, used safely all the time

    // 3. Access a mutable static
    static mut COUNTER: u32 = 0;          // why this is deprecated in 2024 edition
    // Prefer: static COUNTER: AtomicU32 = AtomicU32::new(0);

    // 4. Implement an unsafe trait (Send/Sync) — a promise about thread safety

    // 5. Access union fields
```

### What `unsafe` Actually Means

**`unsafe` does not disable the borrow checker, drop checks, or type checking.** It only permits the five operations above. It is still Rust.

### The Safety Invariant Contract

Every `unsafe` block should be preceded by a `// SAFETY:` comment explaining *why* this is sound. This is not decoration — it's the thing a reviewer checks.

```rust,linenos
    /// # Safety
    /// `ptr` must point to a valid, initialized `T` that outlives `'a`.
    unsafe fn read_value<'a, T>(ptr: *const T) -> &'a T {
        // SAFETY: the caller guarantees ptr is valid and aligned for reads,
        // and that the value is initialized. We return a reference with
        // lifetime 'a, which is bounded by the caller's contract.
        unsafe { &*ptr }
    }
```

### When `unsafe` Is Justified

| Legitimate | Illegitimate |
|-------------|-------------|
| Implementing a data structure with proven invariants | Avoiding a borrow checker error |
| FFI to a C library | Optimizing a hot loop "for speed" |
| Hand-writing a `no_std` primitive | Anything in application code |
| SIMD / intrinsics | Accessing a static you could use `AtomicU32` for |

> **In application code, `unsafe` is almost always a bug.** If you find yourself needing it, the answer is nearly always a different data structure or a crate that already solved it safely.

### Testing `unsafe`

```rust,linenos
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn soundness_holds() {
            // Miri (cargo miri test) checks for undefined behavior
            // across your whole unsafe surface automatically.
            let x = 42;
            let p = &x as *const i32;
            assert_eq!(unsafe { read_value(p) }, &42);
        }
    }
```

> **Run `cargo +nightly miri test`** on any crate containing `unsafe`. Miri is an interpreter that detects undefined behavior — out-of-bounds access, use-after-free, invalid alignment, data races. It is the single best tool for validating unsafe code, and it exists because the Rust team built it for exactly this purpose.

## Performance: What Actually Matters

### Measure Before Optimizing

```bash,linenos
    # 1. Benchmark correctly (criterion — statistically sound)
    cargo bench

    # 2. Profile
    cargo install flamegraph
    cargo flamegraph --bin myapp

    # 3. Find the bottleneck
    perf record ./target/release/myapp
    perf report
```

**Never optimize without a profile.** The intuition that "this code is slow" is wrong more often than not. In most Rust programs, the bottleneck is I/O or an algorithm, not memory layout.

### Zero-Cost Abstractions — With Caveats

Rust abstractions like `Option`, `Result`, iterators, and generics compile to what you would have written by hand. But:

- `Result` propagates error checks the compiler can often eliminate.
- `Iterator` chains fuse into tight loops.
- Generics monomorphize to concrete code.

**However:** "zero-cost" does not mean "zero-size." `Option<T>` where `T` has no niche becomes `size_of::<Option<T>>() == size_of::<T>() + 1`. For large enums, this matters.

```rust,linenos
    use std::mem::size_of;

    #[derive(Clone, Copy)]
    enum Small { A, B }

    struct Big { data: [u8; 1000] }

    // Niche optimization: the discriminant fits in padding
    assert_eq!(size_of::<Option<Small>>(), 1);
    // No niche for a 1000-byte struct — costs a full extra byte
    assert_eq!(size_of::<Option<Big>>(), 1001);
```

### Allocation Is Usually the Real Cost

Most "Rust is slow" complaints come down to excessive allocation, not unsafe or abstraction overhead.

```rust,linenos
    // BAD: allocates a new String on every call
    fn normalize(s: &str) -> String { s.to_lowercase().replace(' ', "_") }

    // BETTER: reuse a buffer
    fn normalize_into(s: &str, buf: &mut String) {
        buf.clear();
        buf.extend(s.chars().filter(|c| *c != ' ').flat_map(char::to_lowercase));
    }

    // BEST: often, don't allocate at all — return a borrowed slice
    fn trim_prefix<'a>(s: &'a str) -> &'a str {
        s.strip_prefix("GET ").unwrap_or(s)   // zero copy
    }
```

**Where allocations hide:**
1. `clone()` in a loop
2. `format!` / `.to_string()` in a hot path
3. Collecting iterators into intermediate `Vec`s before a second pass
4. `String` concatenation with `+` (use `push_str`)
5. Sizing `Vec` without `with_capacity`, causing reallocations

```rust,linenos
    // with_capacity avoids log2(n) reallocations
    let mut v = Vec::with_capacity(1024);
    for i in 0..1000 { v.push(i); }

    // push_str avoids a temporary String
    let mut out = String::with_capacity(256);
    out.push_str("prefix");
    out.push_str(&suffix);
```

### Crossbeam / Rayon / std — Picking the Concurrency Tool

| Need | Tool |
|------|------|
| CPU-bound work over a collection | `rayon` (`.par_iter()`) |
| A few independent I/O futures | `tokio::join!` |
| Many independent I/O futures | `tokio::spawn` + `join_all` |
| Producer/consumer between threads | `crossbeam-channel` or `std::sync::mpsc` |
| Shared mutable state | `Arc<Mutex<T>>` (or `Arc<RwLock<T>>` for read-heavy) |
| Blocking work inside async | `tokio::task::spawn_blocking` |

## Performance Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| `&Vec<T>` parameters | None (it's just wrong, not slow) | Use `&[T]` |
| Boxing everything | Double indirection | Box only trait objects and recursive types |
| `clone()` in a hot loop | Allocations dominate profile | Borrow, or reuse a buffer |
| Collecting into intermediate `Vec`s | Extra allocation + pass | Fuse the iterator chain |
| Unbuffered file I/O | Orders of magnitude slower | `BufReader` / `BufWriter` |
| Not using `with_capacity` | Reallocation + copying | Pre-size your `Vec` |
| `sort_unstable` when you need stability | Wrong result | Use `sort` |
| Optimizing before profiling | Wasted effort | Profile first, always |

## Project Structure Best Practice

```bash,linenos
    myapp/
    ├── Cargo.toml
    ├── Cargo.lock            # commit this
    ├── rust-toolchain.toml   # pin the compiler version
    ├── .clippy.toml
    ├── .rustfmt.toml
    ├── deny.toml             # cargo-deny: license + advisory policy
    ├── src/
    │   ├── main.rs           # thin: parse args, call lib, handle errors
    │   ├── lib.rs            # the public API
    │   ├── config.rs         # configuration loading
    │   ├── error.rs          # thiserror definitions
    │   ├── models.rs         # domain types
    │   ├── routes/           # HTTP handlers
    │   │   ├── mod.rs
    │   │   ├── users.rs
    │   │   └── orders.rs
    │   └── db.rs             # database access
    ├── tests/
    │   ├── api.rs            # integration tests
    │   └── common/mod.rs     # shared test helpers
    └── benches/
        └── parse.rs
```

**`main.rs` should be thin.** Put logic in `lib.rs` so it is testable and reusable. `main` should only: parse args, initialize, call into the library, and handle the top-level error.

## Anti-Patterns Worth Naming

### 1. The `Arc<Mutex<Everything>>`

**Bad:** a global shared state bag guarded by one lock. Every request contends on it.

**Better:** pass dependencies explicitly. If you must share, use fine-grained locks or lock-free reads (`ArcSwap`).

### 2. Trait Objects for Static Dispatch

**Bad:** `fn process(x: &Box<dyn Handler>)` when `Handler` has exactly three implementations and you know them at compile time.

**Better:** `fn process<H: Handler>(x: &H)`. Same speed, no allocation, no indirection.

### 3. `String` Everywhere

**Bad:** storing `String` when you only ever read a fixed set of values.

**Better:** an enum, or a `&'static str`, or a small integer id.

### 4. Premature Generalization

**Bad:** a generic framework for a problem you have exactly once.

**Better:** write the concrete version first. Generalize when you have the *second* use case and can see the real shape of the abstraction.

### 5. Blocking in Async

```rust,linenos
    // BAD: blocks the tokio worker thread
    async fn handler() {
        let data = std::fs::read_to_string("file").unwrap();
        std::thread::sleep(Duration::from_secs(5));
        let _ = data;
    }

    // GOOD
    async fn handler() {
        let data = tokio::fs::read_to_string("file").await.unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
        let _ = data;
    }
```

### 6. Trusting `HashMap` Order

```rust,linenos
    use std::collections::HashMap;

    // BAD: output order is undefined and varies between runs
    let m: HashMap<i32, i32> = (0..5).map(|i| (i, i * 2)).collect();
    // for (k, v) in &m { ... }   // nondeterministic!

    // GOOD: sort, or use BTreeMap
    let mut keys: Vec<&i32> = m.keys().collect();
    keys.sort();
    assert_eq!(keys[0], &0);
```

## The Learning Path

If you are working through this book, here is the order that builds the most understanding fastest.

### Phase 1 — Make It Compile (Weeks 1–2)

Read Vol. 07, then this book's chapters on data structures, ownership, borrowing, and lifetimes. Write small programs until the borrow checker stops surprising you. Do not skip to frameworks.

**Practice:** implement `Vec`-like behavior from scratch (a `MyVec<T>` with push/pop/iter), a linked list, and a binary search tree. Each teaches ownership concretely.

### Phase 2 — Make It Ergonomic (Weeks 3–4)

Traits and generics, errors, iterators, smart pointers. Write a library crate and publish it to crates.io (even privately). Learn the iterator combinators until you reach for them by reflex.

**Practice:** a CSV parser, a JSON pretty-printer, a small CLI tool with `clap` and `serde`.

### Phase 3 — Concurrency and Async (Weeks 5–7)

Threads, channels, `Arc<Mutex<T>>`, then tokio. Understand *why* async exists before reaching for it. Learn cancellation safety — it is the thing that bites everyone eventually.

**Practice:** a concurrent web scraper with a semaphore, a TCP proxy, a background job queue.

### Phase 4 — Production (Weeks 8+)

Error design, testing, Cargo profiles, profiling, deployment. Build something with real users, real failure modes, and real uptime requirements. This is where the last 20% of Rust knowledge lives, and it cannot be taught — only encountered.

## Quick Reference: The Rules That Matter Most

| Rule | Why |
|------|-----|
| Borrow before you clone; clone before you contort | Cloning is cheap; contorted code is not |
| `&str` in, `String` out | Most functions only read |
| Return `impl Iterator` from helpers | Hides types, enables chaining |
| Make illegal states unrepresentable | Newtypes and typestates beat runtime checks |
| `thiserror` in libraries, `anyhow` in binaries | Match right for the audience |
| Never `unwrap()` in library code | Your callers deserve to handle failures |
| Profile before optimizing | Your intuition is wrong more often than right |
| `Box` only for trait objects, recursion, or big moves | Everything else adds a pointer hop |
| `Arc::clone` when sharing, `Rc::clone` when not | Cross-thread requires `Arc` |
| Generics for speed, `dyn` for flexibility | And `Box<dyn Trait>` for heterogeneous collections |
| Pin dependencies in `Cargo.lock` | Reproducible builds |
| Run clippy with `-D warnings` | Catches bugs, not just style |
| Read `cargo expand` when debugging macros | See what the macros generate |

## Further Reading

| Resource | What it is |
|----------|-----------|
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | The official book — read it once, end to end |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Learn by reading annotated code |
| [Rustlings](https://github.com/rust-lang/rustlings) | Small compiler-error-fixing exercises — excellent for the borrow checker |
| [The Rustonomicon](https://doc.rust-lang.org/nomicon/) | The `unsafe` and low-level book |
| [Async Book](https://rust-lang.github.io/async-book/) | The best explanation of async Rust |
| [Rust Reference](https://doc.rust-lang.org/reference/) | The precise specification, when the docs are unclear |
| [docs.rs](https://docs.rs) | Searchable docs for every crate, with runnable examples |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | How to design good Rust APIs |
| [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/lints.html) | Every lint and what it means |

---

*Compiled and tested against **rustc 1.98.1**. Crate versions and download counts retrieved from the crates.io API on **October 2026**. The ecosystem moves fast — verify version numbers against [docs.rs](https://docs.rs) before pinning.*
