+++
title = "Rust Language Reference for Developers"
description = "Ownership, borrowing, lifetimes, traits, and the compiler model"
[extra]
date = 2024-01-15
+++


## The Ownership Model

Rust enforces memory safety at compile time through three rules: **every value has one owner**, **ownership can be moved or borrowed**, and **values are dropped when their owner goes out of scope**. No garbage collector, no runtime overhead.

| Concept | What It Means | Compile Error If You Violate It |
|---------|---------------|--------------------------------|
| **Ownership** | One variable owns the data at any time | Use after move |
| **Move** | Ownership transfers; old variable is gone | Value used after move |
| **Clone** | Deep copy; both variables own separate data | (No error — explicit) |
| **Copy** | Stack-only types (i32, bool, f64) are copied implicitly | (No error — cheap bitwise copy) |
| **Borrow** | Reference to data without taking ownership | Outlives owner, or mutated while borrowed |

```rust,linenos
    // Move: ownership transfers to `b`, `a` is no longer valid
    let a = String::from("hello");
    let b = a;              // a is MOVED into b
    // println!("{}", a);  // COMPILE ERROR: value borrowed after move

    // Clone: explicit deep copy, both live independently
    let a = String::from("hello");
    let b = a.clone();      // b is a new independent String
    println!("{} {}", a, b); // both valid

    // Copy types: i32, bool, char, f64, tuples of Copy types
    let x: i32 = 5;
    let y = x;              // x is COPIED (not moved), both valid
    println!("{} {}", x, y);
```

## Borrowing: Shared and Mutable References

References let you use a value without taking ownership. The borrow checker enforces the rule at compile time: **you can have many shared references OR one mutable reference — never both at the same time.**

```rust,linenos
    // Shared reference (&T): read-only, many allowed simultaneously
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;            // fine — multiple shared refs are OK
    println!("{} {}", r1, r2);

    // Mutable reference (&mut T): exclusive, only one at a time
    let mut s = String::from("hello");
    let r = &mut s;
    r.push_str(", world");
    // let r2 = &mut s;    // COMPILE ERROR: cannot borrow `s` as mutable more than once

    // Cannot mix shared and mutable refs to same data simultaneously
    let r1 = &s;
    // let r2 = &mut s;    // COMPILE ERROR: cannot borrow `s` as mutable because it is also borrowed as immutable
```

> **The borrow checker works at the scope level, not the line level.** In newer Rust (NLL — Non-Lexical Lifetimes), a borrow ends at its last use, not at the end of its scope. This means you can sometimes take a mutable ref after a shared ref as long as the shared ref is no longer used.

## Ownership in Functions

```rust,linenos
    // Passing by value: ownership moves into the function
    fn take(s: String) {
        println!("{}", s);
    } // s is dropped here

    let s = String::from("hi");
    take(s);
    // println!("{}", s); // COMPILE ERROR: s was moved

    // Passing by reference: borrow, caller retains ownership
    fn borrow(s: &String) {
        println!("{}", s);
    } // borrow ends here, s is not dropped

    let s = String::from("hi");
    borrow(&s);
    println!("{}", s); // still valid — we only borrowed

    // Returning ownership: function gives back what it owns
    fn make() -> String {
        String::from("new string") // returned, not dropped
    }
    let s = make(); // s owns the string
```

## The Stack vs the Heap

| | Stack | Heap |
|--|-------|------|
| **Allocation** | Automatic (function call frame) | Explicit (`Box::new`, `String::from`, `Vec::new`) |
| **Size** | Must be known at compile time | Can grow/shrink at runtime |
| **Speed** | Fast (pointer increment) | Slower (allocator call) |
| **Examples** | `i32`, `bool`, `[u8; 4]`, references | `String`, `Vec<T>`, `Box<T>`, `HashMap` |
| **Drop** | Automatic when frame exits | When owning variable is dropped |

> **String vs &str.** `String` is heap-allocated, owned, growable. `&str` is a borrowed reference to string data (often a string literal in the binary, or a slice of a `String`). Functions that only read strings should generally take `&str` — it's more flexible and avoids forcing the caller to allocate.

## Scalar Types

| Type | Size | Notes |
|------|------|-------|
| `i8 / i16 / i32 / i64 / i128` | 1–16 bytes | Signed integers. `i32` is default. |
| `u8 / u16 / u32 / u64 / u128` | 1–16 bytes | Unsigned integers. `u8` is a byte. |
| `isize / usize` | Platform width | Pointer-sized. Used for indexing. |
| `f32 / f64` | 4 / 8 bytes | Floating point. `f64` is default. |
| `bool` | 1 byte | `true` or `false`. |
| `char` | 4 bytes | Unicode scalar value (not a byte). |

## Compound Types

```rust,linenos
    // Tuple: fixed-length, can mix types
    let t: (i32, f64, bool) = (1, 2.0, true);
    println!("{}", t.0); // access by index
    let (x, y, z) = t; // destructure

    // Array: fixed-length, same type, stack-allocated
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let zeros = [0; 10]; // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    // arr[5];           // RUNTIME PANIC: index out of bounds (checked)

    // Vec: heap-allocated, growable array
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    let v2 = vec![1, 2, 3]; // macro shorthand

    // Slice: view into an array or Vec (doesn't own data)
    let slice: &[i32] = &v[0..2]; // first two elements

    // HashMap
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("key", 42);
    let val = map.get("key"); // returns Option<&i32>
```

## Structs

```rust,linenos
    // Named-field struct
    struct User {
        name: String,
        age: u32,
        active: bool,
    }

    let user = User {
        name: String::from("Alice"),
        age: 30,
        active: true,
    };
    println!("{}", user.name);

    // Struct update syntax
    let user2 = User {
        name: String::from("Bob"),
        ..user  // fill remaining fields from `user`
                // NOTE: moves `user` if non-Copy fields are used
    };

    // Tuple struct
    struct Point(f64, f64);
    let p = Point(1.0, 2.0);
    println!("{}", p.0);

    // Unit struct (no fields, used as marker type)
    struct Marker;
```

## impl Blocks: Methods on Structs

```rust,linenos
    struct Rectangle {
        width: f64,
        height: f64,
    }

    impl Rectangle {
        // Associated function (no `self`): called as Rectangle::new()
        fn new(width: f64, height: f64) -> Self {
            Self { width, height }
        }

        // Method: takes `&self` (shared borrow of the instance)
        fn area(&self) -> f64 {
            self.width * self.height
        }

        // Mutable method: takes `&mut self`
        fn scale(&mut self, factor: f64) {
            self.width *= factor;
            self.height *= factor;
        }
    }

    let mut r = Rectangle::new(4.0, 5.0);
    println!("{}", r.area());  // 20.0
    r.scale(2.0);
    println!("{}", r.area());  // 80.0
```

> **`self`, `&self`, `&mut self`.** `self` consumes the value (rare). `&self` borrows immutably (most common — for reads). `&mut self` borrows mutably (for mutations). Pick the least powerful you need.

## Traits

Traits define shared behaviour across types. They're similar to interfaces in other languages, but more powerful — they can have default method implementations, and you can implement them on types you didn't define.

```rust,linenos
    // Define a trait
    trait Summary {
        fn summarize(&self) -> String;

        // Default implementation — types can override or use as-is
        fn preview(&self) -> String {
            format!("{}...", &self.summarize()[..50])
        }
    }

    // Implement for a struct
    struct Article { title: String, content: String }

    impl Summary for Article {
        fn summarize(&self) -> String {
            format!("{}: {}", self.title, self.content)
        }
    }

    // Use trait bound in a function (monomorphized at compile time)
    fn print_summary<T: Summary>(item: &T) {
        println!("{}", item.summarize());
    }

    // Equivalent using `impl Trait` syntax (simpler for single params)
    fn print_summary(item: &impl Summary) {
        println!("{}", item.summarize());
    }

    // Dynamic dispatch with `dyn Trait` (runtime cost, allows mixed types)
    fn print_any(item: &dyn Summary) {
        println!("{}", item.summarize());
    }
```

## Common Standard Traits

| Trait | What It Provides | How to Derive |
|-------|------------------|----------------|
| `Debug` | `{:?}` formatting for debugging | `#[derive(Debug)]` |
| `Display` | `{}` formatting for user output | Must implement manually |
| `Clone` | `.clone()` deep copy | `#[derive(Clone)]` |
| `Copy` | Implicit bitwise copy (stack only) | `#[derive(Copy, Clone)]` |
| `PartialEq / Eq` | `==` and `!=` | `#[derive(PartialEq, Eq)]` |
| `PartialOrd / Ord` | `<`, `>`, sorting | `#[derive(PartialOrd, Ord)]` |
| `Hash` | Use as HashMap key | `#[derive(Hash)]` |
| `Default` | `Type::default()` zero value | `#[derive(Default)]` |
| `Iterator` | Custom iteration with `.next()` | Must implement manually |
| `From / Into` | Type conversions | Implement `From`, get `Into` free |

    // Deriving multiple traits
    #[derive(Debug, Clone, PartialEq)]
    struct Point { x: f64, y: f64 }

    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1.clone();
    println!("{:?}", p1); // Point { x: 1.0, y: 2.0 }
    println!("{}", p1 == p2); // true

    // Implementing Display manually
    use std::fmt;
    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }
    println!("{}", p1); // (1.0, 2.0)

> **Orphan rule:** You can only implement a trait for a type if *either* the trait or the type is defined in your crate. You cannot implement `Display` for `Vec<T>` — neither is yours. This prevents conflicting implementations across crates.

## Enums

Rust enums are algebraic data types — each variant can hold different data. Combined with pattern matching, they're the primary tool for modelling states and outcomes.

```rust,linenos
    // Simple enum
    enum Direction { North, South, East, West }

    // Enum with data in variants
    enum Shape {
        Circle(f64),              // radius
        Rectangle(f64, f64),     // width, height
        Point { x: f64, y: f64 }, // named fields
    }

    let s = Shape::Circle(3.14);

    // Pattern match to extract data
    let area = match s {
        Shape::Circle(r)        => std::f64::consts::PI * r * r,
        Shape::Rectangle(w, h)  => w * h,
        Shape::Point { .. }     => 0.0,
    };
```

## Option<T>

Rust has no `null`. Instead, optional values use `Option<T>`. The compiler forces you to handle the `None` case. No null pointer exceptions.

```rust,linenos
    enum Option<T> {
        Some(T),
        None,
    }

    let maybe: Option<i32> = Some(42);
    let nothing: Option<i32> = None;

    // Pattern match
    match maybe {
        Some(n) => println!("got {}", n),
        None    => println!("nothing"),
    }

    // Convenient methods
    maybe.unwrap();              // panics if None — use only when certain
    maybe.unwrap_or(0);          // default value if None
    maybe.unwrap_or_else(|| compute()); // lazily computed default
    maybe.map(|n| n * 2);        // transform if Some, pass through None
    maybe.and_then(|n| lookup(n)); // chain operations that may fail
    maybe.is_some();              // bool check
    maybe.is_none();              // bool check

    // if let: concise pattern match for one case
    if let Some(n) = maybe {
        println!("got {}", n);
    }
```

## Pattern Matching

```rust,linenos
    // match must be exhaustive — compiler enforces all cases are covered
    let n = 7;
    match n {
        1         => println!("one"),
        2 | 3     => println!("two or three"),
        4..=6     => println!("four to six"),
        x if x < 0 => println!("negative: {}", x),
        _          => println!("something else"), // catch-all
    }

    // Destructure structs in match
    let p = Point { x: 3.0, y: 4.0 };
    match p {
        Point { x: 0.0, y } => println!("on y-axis at {}", y),
        Point { x, y }      => println!("({}, {})", x, y),
    }

    // while let: loop while pattern matches
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
```

## Result<T, E>

Operations that can fail return `Result<T, E>`. The compiler forces you to handle failures — you cannot accidentally ignore an error.

```rust,linenos
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

    // Parsing a string: can succeed (Ok) or fail (Err)
    let r: Result<i32, _> = "42".parse();
    match r {
        Ok(n)  => println!("parsed: {}", n),
        Err(e) => println!("failed: {}", e),
    }

    // Convenient Result methods (mirror Option)
    r.unwrap();              // panic on Err — use in tests or when certain
    r.unwrap_or(0);          // default on Err
    r.expect("parse failed"); // panic with message on Err (better than unwrap)
    r.map(|n| n * 2);        // transform Ok value
    r.map_err(|e| MyErr(e)); // transform Err value
    r.and_then(|n| next(n)); // chain fallible operations
    r.is_ok(); r.is_err();   // bool checks
```

## The ? Operator

The `?` operator is shorthand for: if `Ok`, unwrap and continue; if `Err`, return the error immediately from the current function. It eliminates boilerplate match chains for error propagation.

```rust,linenos
    use std::fs;
    use std::io;

    // Without ?: manual propagation
    fn read_file_verbose() -> Result<String, io::Error> {
        let content = match fs::read_to_string("file.txt") {
            Ok(s)  => s,
            Err(e) => return Err(e),
        };
        Ok(content)
    }

    // With ?: concise propagation
    fn read_file() -> Result<String, io::Error> {
        let content = fs::read_to_string("file.txt")?; // returns Err if it fails
        Ok(content)
    }

    // Chaining with ?
    fn parse_config() -> Result<Config, MyError> {
        let raw = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&raw)?;
        Ok(config)
    }
```

> **? works on both Result and Option.** In a function returning `Option<T>`, `?` returns `None` early if applied to a `None` value. In a function returning `Result`, it returns the `Err`. The function's return type determines behavior.

## Custom Error Types

```rust,linenos
    // Simple custom error with thiserror crate (recommended)
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum AppError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("parse failed: {msg}")]
        Parse { msg: String },

        #[error("not found: {0}")]
        NotFound(String),
    }

    // anyhow crate: for applications where you don't need typed errors
    use anyhow::Result;
    fn run() -> Result<()> {   // Result<(), anyhow::Error>
        let s = fs::read_to_string("file")?; // any error works
        Ok(())
    }
```

> **thiserror for libraries, anyhow for applications.** `thiserror` gives you typed errors callers can match on. `anyhow` is ergonomic for binaries where you just want to propagate and display errors. Use `Box<dyn Error>` if you want no deps but accept the ergonomic cost.

## Lifetimes

Lifetimes are the compiler's way of tracking how long references are valid. Most of the time the compiler infers them (lifetime elision). You only write them explicitly when the compiler can't figure it out — usually when a function returns a reference and the compiler can't determine which input it came from.

```rust,linenos
    // The problem: which input does the returned reference point to?
    // The compiler needs to know to ensure the output doesn't outlive the input.
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }
    // 'a means: "the returned reference lives as long as the shorter of x and y"

    // Lifetime in a struct: struct holds a reference, must not outlive what it points to
    struct Excerpt<'a> {
        text: &'a str,
    }

    let novel = String::from("Call me Ishmael...");
    let first = novel.split('.').next().unwrap();
    let excerpt = Excerpt { text: first };
    // `excerpt` cannot outlive `novel` — compiler enforces this
```

## Lifetime Elision Rules

You don't write lifetimes in most function signatures because the compiler applies three elision rules automatically:

| Rule | What It Says |
|------|---------------|
| Rule 1 | Each reference parameter gets its own lifetime: `fn f(x: &T, y: &U)` → `fn f<'a,'b>(x: &'a T, y: &'b U)` |
| Rule 2 | If there's exactly one input lifetime, all outputs get that lifetime. |
| Rule 3 | If one of the inputs is `&self` or `&mut self`, all output lifetimes get `self`'s lifetime. |

```rust,linenos
    // These three signatures are equivalent after elision:
    fn first_word(s: &str) -> &str
    // is the same as:
    fn first_word<'a>(s: &'a str) -> &'a str

    // Method on a struct: rule 3 applies
    impl Excerpt<'_> {
        fn announce(&self, msg: &str) -> &str {
            println!("Attention: {}", msg);
            self.text  // lifetime of return = lifetime of &self (rule 3)
        }
    }
```

## The 'static Lifetime

```rust,linenos
    // 'static: lives for the entire program duration
    let s: &'static str = "I live forever"; // string literals are 'static

    // Common in trait objects and thread-spawning
    fn spawn_worker(f: impl Fn() + Send + 'static) {
        std::thread::spawn(f);
    }
    // `'static` here means the closure must not borrow from the current stack frame
    // (the thread might outlive the current function call)
```

> **Don't reach for lifetimes first.** If you find yourself writing complex lifetime annotations, consider whether you could own the data (use `String` instead of `&str`, `Vec<T>` instead of `&[T]`) or restructure the code. Explicit lifetimes are sometimes unavoidable but often a design smell.

## Threads

Rust's ownership system makes data races **impossible to compile**. The `Send` and `Sync` traits mark what's safe to share across thread boundaries — the compiler enforces them automatically.

```rust,linenos
    use std::thread;

    // Spawn a thread, move data into it
    let data = String::from("hello");
    let handle = thread::spawn(move || {
        println!("in thread: {}", data); // `move` transfers ownership
    });
    handle.join().unwrap(); // wait for thread to finish

    // Spawn many threads and collect results
    let handles: Vec<_> = (0..10)
        .map(|i| thread::spawn(move || i * i))
        .collect();

    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
```

## Sharing State: Arc and Mutex

```rust,linenos
    use std::sync::{Arc, Mutex};
    use std::thread;

    // Arc = Atomically Reference Counted: shared ownership across threads
    // Mutex = Mutual Exclusion: only one thread can access data at a time
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..10).map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || {
            let mut num = c.lock().unwrap(); // lock — blocks until available
            *num += 1;
        }) // lock released automatically when `num` goes out of scope
    }).collect();

    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("count: {}", *counter.lock().unwrap()); // 10
```

## Channels: Message Passing

```rust,linenos
    use std::sync::mpsc; // multi-producer, single-consumer
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // Sender can be cloned for multiple producers
    let tx2 = tx.clone();

    thread::spawn(move || {
        tx.send(String::from("from thread 1")).unwrap();
    });

    thread::spawn(move || {
        tx2.send(String::from("from thread 2")).unwrap();
    });

    // Receive (blocks until a message arrives)
    for msg in rx {
        println!("{}", msg);
    }
```

## Async / Await

```rust,linenos
    // Async functions return a Future — they don't run until polled
    // Requires a runtime like tokio or async-std

    // Cargo.toml: tokio = { version = "1", features = ["full"] }

    use tokio;

    #[tokio::main]
    async fn main() {
        let result = fetch_data().await;
        println!("{:?}", result);
    }

    async fn fetch_data() -> Result<String, reqwest::Error> {
        let body = reqwest::get("https://httpbin.org/get")
            .await?
            .text()
            .await?;
        Ok(body)
    }

    // Spawn concurrent async tasks
    let (a, b) = tokio::join!(task_a(), task_b()); // run concurrently, wait for both
    let handle = tokio::task::spawn(task_c());        // fire and forget (or .await later)
```

> **Async ≠ multithreaded by default.** `async/await` is cooperative concurrency on one (or a pool of) thread(s). `tokio::task::spawn` can run on a thread pool, but a `Future` alone doesn't create threads. Use threads for CPU-bound work; use async for I/O-bound work.

## Cargo: The Build Tool and Package Manager

```bash,linenos
    # Create a new project
    cargo new my_project         # binary (has src/main.rs)
    cargo new my_lib --lib       # library (has src/lib.rs)

    # Build and run
    cargo build                  # compile (debug mode, fast compile, slow binary)
    cargo build --release        # optimized (slow compile, fast binary)
    cargo run                    # build + run
    cargo run --release
    cargo run -- arg1 arg2       # pass args to the binary

    # Check and test
    cargo check                  # type-check only, no codegen — very fast
    cargo test                   # run all tests
    cargo test test_name         # run tests matching a name
    cargo test -- --nocapture    # show println! output in tests

    # Dependency management
    cargo add serde              # add crate (requires cargo-edit or Rust 1.62+)
    cargo add serde --features derive
    cargo update                 # update Cargo.lock to latest compatible versions
    cargo outdated               # show outdated deps (requires cargo-outdated)

    # Other
    cargo fmt                    # format all code with rustfmt
    cargo clippy                 # lint: catch common mistakes and style issues
    cargo doc --open             # build and open documentation in browser
    cargo publish                # publish crate to crates.io
```

## Cargo.toml

```toml,linenos
    [package]
    name = "my_project"
    version = "0.1.0"
    edition = "2021"         # always use 2021

    [dependencies]
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    tokio = { version = "1", features = ["full"] }
    reqwest = { version = "0.12", features = ["json"] }
    thiserror = "1"
    anyhow = "1"

    [dev-dependencies]           # only for tests and benchmarks
    mockall = "0.12"

    [features]
    default = []
    my-feature = ["some-optional-dep"]

    [[bin]]                      # multiple binaries in one project
    name = "server"
    path = "src/bin/server.rs"
```

## Essential Crates

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | Serialize/deserialize JSON (and many other formats) |
| `tokio` | Async runtime — the standard for async Rust |
| `reqwest` | HTTP client (async, built on tokio) |
| `axum` | Web framework (async, built on tokio) |
| `sqlx` | Async SQL with compile-time query checking |
| `thiserror` | Derive macros for custom error types (libraries) |
| `anyhow` | Ergonomic error handling (applications) |
| `tracing` | Structured logging and diagnostics |
| `clap` | CLI argument parsing with derive macros |
| `rayon` | Data parallelism — parallel iterators |
| `rand` | Random number generation |
| `regex` | Regular expressions |
| `chrono` | Date and time handling |
| `uuid` | UUID generation and parsing |

## Testing

```rust,linenos
    // Unit tests: in the same file, inside a `#[cfg(test)]` module
    #[cfg(test)]
    mod tests {
        use super::*; // bring parent module into scope

        #[test]
        fn test_add() {
            assert_eq!(add(2, 3), 5);
        }

        #[test]
        #[should_panic(expected = "divide by zero")]
        fn test_panic() {
            divide(1, 0);
        }

        #[test]
        fn test_result() -> Result<(), String> {
            let n: i32 = "5".parse().map_err(|e: _| e.to_string())?;
            assert_eq!(n, 5);
            Ok(())
        }
    }

    // Integration tests: in tests/ directory (separate crate, public API only)
    // tests/integration_test.rs
    use my_project;

    #[test]
    fn it_works() {
        assert!(my_project::public_fn());
    }
```
