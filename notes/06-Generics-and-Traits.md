# Lecture 06: Generics and Traits

**Video:** https://www.youtube.com/watch?v=cT_9ua-cr9w
**Uploader:** DigiKey  **Duration:** ~27 min  **Published:** 2026-02-26

## Table of Contents

- [Overview](#overview)
- [Motivation: Strong Typing and Boilerplate](#motivation-strong-typing-and-boilerplate)
- [Generic Functions](#generic-functions)
- [Monomorphisation](#monomorphisation)
- [Defining Traits](#defining-traits)
- [Implementing a Trait for a Type](#implementing-a-trait-for-a-type)
- [Trait Bounds](#trait-bounds)
- [Where Clauses vs Inline Bounds](#where-clauses-vs-inline-bounds)
- [Generic Structs](#generic-structs)
- [Generic Enums](#generic-enums)
- [Static Dispatch vs Dynamic Dispatch](#static-dispatch-vs-dynamic-dispatch)
- [Embedded Example: An LED Wrapper](#embedded-example-an-led-wrapper)
- [embedded-hal: The Canonical Embedded Trait Set](#embedded-hal-the-canonical-embedded-trait-set)
- [Source Code](#source-code)
- [Quick Reference](#quick-reference)

---

## Overview

Rust provides two complementary features for reducing boilerplate while preserving compile-time type safety:

- **Generics** -- parameterise functions, structs and enums over types (analogous to C++ templates).
- **Traits** -- declare a contract of shared behaviour that types may opt-in to implement (analogous to interfaces in Java, C# or Go).

Together they enable code that is both polymorphic and zero-cost: the compiler resolves every use to a concrete type at compile time, with no runtime indirection by default. This lecture introduces both features without touching embedded code, then closes with a small embedded wrapper around an LED pin to ground the concepts in hardware.

> [!NOTE]
> This episode deliberately works in a host project (run with `cargo run` on the operating system) so that the language features can be examined in isolation. The final section returns to a Pico-based blinky example to apply what was learned.

---

## Motivation: Strong Typing and Boilerplate

Rust is strongly typed. Every variable, parameter and return value has a fixed concrete type known at compile time, even when inference removes the syntactic burden of writing the type out. This safety has a cost: code written for one concrete type does not automatically apply to another.

Consider a function that swaps the two elements of a pair:

```rust
fn swap(pair: (i32, &str)) -> (&str, i32) {
    (pair.1, pair.0)
}

fn main() {
    let original = (42, "hello");
    let swapped = swap(original);
    println!("{:?}", swapped);
}
```

This works, but it is locked to the pair `(i32, &str)`. Passing `(42, 4.3)` triggers a compile error because `4.3` is an `f64`, not a `&str`. Writing a fresh `swap_f64`, `swap_u8` and so on is the boilerplate trap that generics solve.

---

## Generic Functions

Generics introduce type parameters in angle brackets after the function name. By convention, Rust uses single uppercase letters starting at `T`:

```rust
// Generic function
fn swap<T, U>(pair: (T, U)) -> (U, T) {
    (pair.1, pair.0)
}

fn demo_swap() {
    let original = (42, "hello");
    let swapped = swap(original);

    println!("{:?}", swapped);
}
```

At each call site the compiler infers the concrete types of `T` and `U`. Type inference is bidirectional: hovering with rust-analyzer (Ctrl+Alt) reveals `T = i32, U = f64` in the first call and `T = i32, U = &str` in the second.

```mermaid
flowchart LR
    A["fn swap<T, U>(pair: (T, U)) -> (U, T)"] --> B{Compiler sees call site}
    B -->|"swap((42, 4.3))"| C["Generate swap_i32_f64"]
    B -->|"swap((42, \"hello\"))"| D["Generate swap_i32_str"]
    B -->|"swap((true, 1u8))"| E["Generate swap_bool_u8"]
    C --> F[Final binary contains one specialised copy per used pair]
    D --> F
    E --> F
```

---

## Monomorphisation

When the compiler encounters a generic, it does **not** emit a single polymorphic function. Instead, for each distinct set of concrete type arguments actually used in the program, it emits a separate fully specialised copy. This process is called **monomorphisation** and is the reason generics in Rust are described as "zero-cost": dispatch is fully resolved at compile time and the call sites are identical to those of hand-written non-generic code.

> [!IMPORTANT]
> Monomorphisation has a flash-size cost. Every distinct instantiation produces an additional copy of the function in the final binary. On a microcontroller with limited flash, generously generic libraries called with many concrete types can noticeably inflate code size. When binary size matters more than per-call performance, prefer dynamic dispatch via `dyn Trait` (covered below).

Mathematically, if a generic function `f<T>` is used at concrete types $T \in \{T_1, T_2, \ldots, T_n\}$, the binary size contribution is

$$
\text{Size}(f) \;=\; \sum_{i=1}^{n} \text{size}\big(f_{T_i}\big)
$$

whereas a `dyn Trait` version pays a single copy plus per-instance vtable overhead:

$$
\text{Size}(f_{\text{dyn}}) \;=\; \text{size}(f) \;+\; \sum_{i=1}^{n} \text{size}(\text{vtable}_{T_i}).
$$

---

## Defining Traits

A trait declares the methods a conforming type must provide. The trait alone does not contain any data or behaviour for any concrete type; it is purely a contract.

```rust
trait Hello {
    fn say_hello(&self);
}
```

By convention trait names use UpperCamelCase. The method signature here takes `&self` and returns nothing. Trait bodies may also include **default method implementations** that implementors may override:

```rust
trait Greeter {
    fn name(&self) -> &str;

    // Default method -- implementors get this for free.
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}
```

---

## Implementing a Trait for a Type

Given a struct, the trait is attached using `impl Trait for Type`:

```rust
// Define a type
struct Person {
    name: String,
}

// Implement the trait for the type
impl Hello for Person {
    fn say_hello(&self) {
        println!("Hello, {}", self.name);
    }
}

// Use the struct
fn demo_trait() {
    let me = Person {
        name: String::from("Shawn"),
    };

    me.say_hello();
}
```

The function signature inside `impl` must match the one in the trait exactly (same parameters, same return type). The body replaces the semicolon of the declaration with `{ ... }`.

This is precisely the pattern documented throughout the embedded-hal ecosystem: HAL crates implement standard traits (`OutputPin`, `I2c`, `SpiBus`, `Read`, `Write`, ...) for their peripheral types, so client code written against the trait runs unchanged across vendors.

---

## Trait Bounds

A naive generic `add` fails to compile:

```rust
fn add<T>(a: T, b: T) -> T {
    a + b   // error: cannot add T to T
}
```

Not every type supports `+`. Rust refuses to compile until we restrict `T` to types for which `+` is meaningful. This restriction is expressed as a **trait bound** -- a requirement that `T` implements a particular trait. The `+` operator desugars to a call to `core::ops::Add`, so the fix is:

```rust
// Fix: add trait bound
fn add<T>(a: T, b: T) -> T
where
    T: std::ops::Add<Output = T>,
{
    a + b
}
```

Now `add(-3, 10)` and `add(12.345, 2.86)` compile, while `add(true, false)` and `add("Hello, ", "world!")` are rejected at compile time -- because `bool` and `&str` do not implement `Add<Output = Self>`.

| Language    | Strategy                       | When errors surface                                    |
|-------------|--------------------------------|--------------------------------------------------------|
| Python      | Duck typing (dynamic)          | Runtime: `TypeError` when the operator is invoked      |
| JavaScript  | Duck typing (dynamic)          | Runtime: silent coercion or `NaN`                      |
| C++         | Compile-time duck typing       | Compile time, but only on template instantiation       |
| Rust        | Explicit trait bounds          | Compile time, at the generic's definition site         |

---

## Where Clauses vs Inline Bounds

There are two equivalent syntaxes for trait bounds:

```rust
// Inline form -- concise, good for one or two bounds.
fn add<T: Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

// `where` clause -- preferred when there are several bounds
// or when bounds are long enough to clutter the signature.
fn add<T>(a: T, b: T) -> T
where
    T: Add<Output = T>,
{
    a + b
}
```

Multiple bounds combine with `+`:

```rust
fn debug_add<T>(a: T, b: T) -> T
where
    T: Add<Output = T> + Copy + core::fmt::Debug,
{
    println!("{:?} + {:?}", a, b);
    a + b
}
```

---

## Generic Structs

Generics work on structs and enums as well as functions. A pair holding two arbitrary types:

```rust
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {

    // Acts as a manual constructor
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }

    // Swap first and second
    fn swap(self) -> Pair<U, T> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }
}

fn demo_struct() {
    let collection = Pair::new(42, "hello");
    let swapped = collection.swap();

    println!("{}, {}", swapped.first, swapped.second);
}
```

Note the angle brackets on the `impl` block: `impl<T, U> Pair<T, U>` declares the type parameters that the implementation is generic over. Without the leading `<T, U>` the compiler would search for concrete types named `T` and `U`.

---

## Generic Enums

The same machinery powers `Option<T>` and `Result<T, E>` in the core library. As an exercise the episode rebuilds `Option` from scratch:

```rust
// Similar to Option<T> { Some(T), None }
enum Maybe<T> {
    Something(T),
    Nothing,
}

impl<T> Maybe<T> {

    // Panic if nothing, otherwise return the value in Something
    fn unwrap(self) -> T {
        match self {
            Maybe::Something(value) => value,
            Maybe::Nothing => panic!("Called unwrap on Nothing"),
        }
    }
}

fn demo_enum() {
    let no_value: Maybe<String> = Maybe::Nothing;
    let some_number: Maybe<f64> = Maybe::Something(1.2345);

    // Check manually
    match no_value {
        Maybe::Something(value) => println!("Value: {}", value),
        Maybe::Nothing => println!("No value found"),
    }

    // If we know that it's not nothing, we can use unwrap
    println!("Unwrapped value: {}", some_number.unwrap());
}
```

Rust has no built-in `null`. `Option<T>` (and our `Maybe<T>`) forces the absence-of-value case to be represented in the type system, eliminating an entire class of dereference-of-null bugs. Enums must exhaustively enumerate their variants; the compiler will reject a `match` that omits any.

---

## Static Dispatch vs Dynamic Dispatch

Generics with trait bounds give **static dispatch**: every call site is resolved at compile time to a concrete monomorphised function. The alternative is **dynamic dispatch** via `dyn Trait`, where the concrete type is hidden behind a pointer plus a vtable, and the correct method is selected at runtime by following the vtable pointer.

```rust
trait Shape {
    fn area(&self) -> f32;
}

// Static dispatch -- monomorphised per concrete T.
fn print_area_static<T: Shape>(s: &T) {
    println!("{}", s.area());
}

// Equivalent shorthand using `impl Trait`.
fn print_area_impl(s: &impl Shape) {
    println!("{}", s.area());
}

// Dynamic dispatch -- one function body, vtable lookup per call.
fn print_area_dyn(s: &dyn Shape) {
    println!("{}", s.area());
}
```

| Aspect                  | Generics / `impl Trait` (static)               | `dyn Trait` (dynamic)                      |
|-------------------------|------------------------------------------------|--------------------------------------------|
| Dispatch resolved at    | Compile time                                   | Run time (vtable lookup)                   |
| Per-call cost           | Direct call, inlinable                         | Indirect call through vtable, no inlining  |
| Binary size             | Grows with each concrete instantiation         | One copy regardless of implementor count   |
| Heterogeneous collections | Not directly (each `T` is a distinct type)   | Yes -- `Vec<Box<dyn Shape>>` works         |
| Object safety required  | No                                             | Yes (no generic methods, no `Self` return) |
| Typical embedded usage  | Default -- zero-cost abstraction               | When flash budget or heterogeneity demands |

`impl Trait` in argument position is sugar for an anonymous generic parameter: `fn f(x: impl Trait)` is identical to `fn f<T: Trait>(x: T)`. In return position it is different: `-> impl Trait` returns a single concrete (but unnamed) type, while `-> Box<dyn Trait>` returns any boxed implementor.

---

## Embedded Example: An LED Wrapper

The closing example wraps a GPIO pin into a friendlier `Led` driver. The pin type on the rp2040-hal is itself generic over the pin number, function, pull, and so on, so we accept *any* pin that implements the `embedded_hal::digital::OutputPin` trait.

```rust
use embedded_hal::digital::OutputPin;

// Generic LED struct
struct Led<P> {
    pin: P,
    active_high: bool,
}

// Implementation of LED struct
// Alternative inline trait bound: `impl<P: OutputPin> Led<P> {`
impl<P> Led<P>
where
    P: OutputPin,
{

    // Instantiate a new Led
    fn new(pin: P, active_high: bool) -> Self {
        Self { pin, active_high }
    }

    // Turn the LED on
    fn on(&mut self) {
        if self.active_high {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }

    // Turn the LED off
    fn off(&mut self) {
        if self.active_high {
            self.pin.set_low().unwrap();
        } else {
            self.pin.set_high().unwrap();
        }
    }
}
```

The inline shorthand `impl<P: OutputPin> Led<P>` is equivalent to the `where` form above. Both are widely used; the `where` form is recommended once bounds grow beyond a single trait.

In the main blinky loop, the generic type parameter `P` is filled in automatically by inference based on the pin that is moved in:

```rust
let led_pin = pins.gpio15.into_push_pull_output();
let mut led = Led::new(led_pin, true);

loop {
    led.on();
    timer.delay_ms(500);
    led.off();
    timer.delay_ms(500);
}
```

Changing GPIO number changes the concrete type of `P` because rp235x-hal encodes the pin number in the type, but the surrounding code does not change because everything is expressed through the `OutputPin` trait.

> [!IMPORTANT]
> Each distinct pin used with `Led<P>` causes monomorphisation to emit a fresh copy of `new`, `on` and `off`. For a handful of LEDs this is negligible; for a driver library used with dozens of pin types it can matter. Profile with `cargo size` or `cargo bloat` before reaching for `dyn`.

---

## embedded-hal: The Canonical Embedded Trait Set

The `embedded-hal` crate is the working example of trait-driven abstraction in the Rust embedded ecosystem. It defines vendor-neutral traits such as:

| Trait                                     | Purpose                                       |
|-------------------------------------------|-----------------------------------------------|
| `digital::OutputPin` / `InputPin`         | GPIO direction-specific access                |
| `digital::StatefulOutputPin`              | Output pins that can be queried and toggled   |
| `i2c::I2c`                                | I2C bus controller                            |
| `spi::SpiBus` / `SpiDevice`               | SPI bus and bus-shared device abstractions    |
| `delay::DelayNs`                          | Blocking nanosecond/microsecond/ms delays     |
| `pwm::SetDutyCycle`                       | PWM output duty-cycle control                 |

Vendor HAL crates (rp2040-hal, stm32f4xx-hal, nrf-hal, esp-hal, ...) implement these traits for their peripheral types. Generic driver crates (for displays, sensors, radios, ...) are then written against the traits and run unmodified across every supported microcontroller. This is the same pattern as the `Led<P: OutputPin>` wrapper, scaled up to a whole ecosystem.

The next episode applies all of the above to write a real driver for the TMP102 temperature sensor over I2C.

---

## Source Code

The companion code for this lecture lives in two workspace apps:

- [`workspace/apps/generics-examples`](../workspace/apps/generics-examples) -- host-side demos for the generic `swap`, the `Hello` trait, the `add` trait bound, the `Pair<T, U>` generic struct, and the `Maybe<T>` generic enum.
- [`workspace/apps/led-wrapper`](../workspace/apps/led-wrapper) -- the embedded `Led<P: OutputPin>` wrapper running on the rp235x (Pico 2) with a blinky main loop.
- [`workspace/apps/rp2040-led-wrapper`](../workspace/apps/rp2040-led-wrapper) -- equivalent port for the original Raspberry Pi Pico (RP2040). The generic `Led<P>` struct and its `impl` block are byte-for-byte identical to the rp235x version; only the chip boilerplate around them (HAL crate, boot artefact, `Timer` constructor, target triple) differs — the canonical demonstration of `embedded-hal` trait-driven portability that the lecture argues for.

---

## Quick Reference

```rust
// Generic function with a `where` clause.
fn add<T>(a: T, b: T) -> T
where
    T: core::ops::Add<Output = T>,
{
    a + b
}

// Equivalent inline-bound form.
fn add<T: core::ops::Add<Output = T>>(a: T, b: T) -> T { a + b }

// Trait definition with a default method.
trait Greeter {
    fn name(&self) -> &str;
    fn greet(&self) { println!("Hi, {}", self.name()); }
}

// Trait implementation for a custom type.
struct Person { name: String }
impl Greeter for Person {
    fn name(&self) -> &str { &self.name }
}

// Generic struct + impl block.
struct Pair<T, U> { first: T, second: U }
impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self { Pair { first, second } }
    fn swap(self) -> Pair<U, T> { Pair { first: self.second, second: self.first } }
}

// Generic enum -- the shape of `Option<T>`.
enum Maybe<T> { Something(T), Nothing }

// Static vs dynamic dispatch.
fn s<T: Greeter>(g: &T)   { g.greet(); }   // monomorphised per T
fn i(g: &impl Greeter)    { g.greet(); }   // same, shorthand
fn d(g: &dyn Greeter)     { g.greet(); }   // vtable lookup at runtime

// Heterogeneous collection requires `dyn`.
let speakers: alloc::vec::Vec<alloc::boxed::Box<dyn Greeter>> = alloc::vec![];
```

| Concept                     | Syntax                                  | Cost model                       |
|-----------------------------|-----------------------------------------|----------------------------------|
| Generic function            | `fn f<T>(x: T)`                         | Monomorphised per concrete `T`   |
| Trait bound (inline)        | `fn f<T: Trait>(x: T)`                  | Monomorphised, compile-time only |
| Trait bound (`where`)       | `fn f<T>(x: T) where T: Trait`          | Same                             |
| `impl Trait` argument       | `fn f(x: impl Trait)`                   | Same as `<T: Trait>`             |
| `impl Trait` return         | `-> impl Trait`                         | Single hidden concrete type      |
| `dyn Trait`                 | `&dyn Trait`, `Box<dyn Trait>`          | Vtable, runtime dispatch         |
| Generic struct              | `struct S<T> { x: T }`                  | Monomorphised per use            |
| Generic enum                | `enum E<T> { A(T), B }`                 | Monomorphised per use            |
| Default trait method        | inside `trait { fn m() { ... } }`       | Override-optional shared body    |
| Multiple bounds             | `T: A + B`, or `where T: A, T: B`       | All must hold for instantiation  |
