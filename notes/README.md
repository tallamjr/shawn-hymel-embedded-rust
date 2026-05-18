# Introduction to Embedded Rust — DigiKey Lecture Notes

A 12-part video series by **DigiKey** covering embedded Rust development on the
Raspberry Pi Pico 2 (RP2350) microcontroller. These notes are generated from
the original video transcripts, then expanded with reference material, code
listings, diagrams, and tables.

**Playlist:** <https://www.youtube.com/playlist?list=PLEBQazB0HUySZGmArV2sazIIXLKUq2ic->
**Uploader:** DigiKey
**Total runtime:** ~6 hours 18 minutes across 12 videos
**Series published:** 2026-01-22 — 2026-04-09

> [!NOTE]
> Lecture 04 (Ownership and Borrowing) was generated from a partially
> corrupted transcript — `mlx_whisper` entered a repeat-loop on the second
> half of the audio. The code blocks have since been reconciled against
> `workspace/apps/ownership-examples/` and now reflect the real source, but
> some surrounding prose was reconstructed from canonical Rust documentation
> consistent with the topic outline. If you re-transcribe that video with
> `--condition_on_previous_text False`, regenerate the lecture note from the
> fresh transcript for a fully faithful narrative.

## Lectures

| #   | Title                                           | Duration | Published  | Notes                                                                                                      |
| --- | ----------------------------------------------- | -------- | ---------- | ---------------------------------------------------------------------------------------------------------- |
| 01  | What is Rust?                                   | ~56 min  | 2026-01-22 | [01-What-is-Rust.md](01-What-is-Rust.md)                                                                   |
| 02  | Blink an LED                                    | ~43 min  | 2026-01-29 | [02-Blink-an-LED.md](02-Blink-an-LED.md)                                                                   |
| 03  | USB Serial Logging and Debugging                | ~27 min  | 2026-02-05 | [03-USB-Serial-Logging-and-Debugging.md](03-USB-Serial-Logging-and-Debugging.md)                           |
| 04  | Ownership and Borrowing                         | ~42 min  | 2026-02-12 | [04-Ownership-and-Borrowing.md](04-Ownership-and-Borrowing.md)                                             |
| 05  | Reading from an I2C Temperature Sensor (TMP102) | ~27 min  | 2026-02-19 | [05-Reading-from-an-I2C-Temperature-Sensor.md](05-Reading-from-an-I2C-Temperature-Sensor.md)               |
| 06  | Generics and Traits                             | ~27 min  | 2026-02-26 | [06-Generics-and-Traits.md](06-Generics-and-Traits.md)                                                     |
| 07  | Creating a TMP102 Driver Library and Crate      | ~25 min  | 2026-03-05 | [07-Creating-a-TMP102-Driver-Library-and-Crate.md](07-Creating-a-TMP102-Driver-Library-and-Crate.md)       |
| 08  | Lifetimes and Lifetime Annotations              | ~24 min  | 2026-03-12 | [08-Lifetimes-and-Lifetime-Annotations.md](08-Lifetimes-and-Lifetime-Annotations.md)                       |
| 09  | Test-Driven Development                         | ~27 min  | 2026-03-19 | [09-Test-Driven-Development.md](09-Test-Driven-Development.md)                                             |
| 10  | Interrupts                                      | ~22 min  | 2026-03-26 | [10-Interrupts.md](10-Interrupts.md)                                                                       |
| 11  | Logging with `defmt` and Step-through Debugging | ~25 min  | 2026-04-02 | [11-Logging-with-defmt-and-Step-through-Debugging.md](11-Logging-with-defmt-and-Step-through-Debugging.md) |
| 12  | Async Programming with Embassy                  | ~39 min  | 2026-04-09 | [12-Async-Programming-with-Embassy.md](12-Async-Programming-with-Embassy.md)                               |

## Learning Path

The series builds incrementally — every concept is reused in later episodes.
The notes can be read in order, or by theme:

### Foundations & Bring-Up

1. **What is Rust?** — language history, the C unsafety problem, the three
   memory-management strategies, Rust's ownership model, ecosystem and
   embedded landscape, hardware bill of materials, Docker dev container,
   `rustc` Hello World, Cargo, and Rustlings setup.
2. **Blink an LED** — `no_std` / `no_main`, panic handlers, `.cargo/config.toml`,
   `memory.x`, the RP2350 HAL clock + GPIO + Timer API, the `picotool` UF2
   flash workflow.
3. **USB Serial Logging and Debugging** — USB CDC ACM with `usb-device` /
   `usbd-serial`, VID/PID/CDC class config, the non-blocking poll/read/write
   super loop, host-side terminal, debug vs release binary size.

### Core Language Semantics

4. **Ownership and Borrowing** — stack/heap, the three ownership rules,
   move semantics, `Copy` vs `Clone`, references (`&T`, `&mut T`), the borrow
   checker, slices, embedded relevance.
5. **Lifetimes and Lifetime Annotations** — why lifetimes exist, `'a` syntax,
   function and struct annotations, the three elision rules, `'static`,
   lifetimes in trait objects, DMA-buffer borrowing patterns.

### Hardware Drivers & Reusable Code

5. **Reading from an I2C Temperature Sensor (TMP102)** — I2C protocol primer
   (START/STOP, ACK/NAK, 7-bit addressing), TMP102 register map, raw read
   via `embedded-hal::I2c::write_read`, conversion to Celsius.
6. **Generics and Traits** — generic functions and types, trait definitions
   and implementations, trait bounds and `where` clauses, static vs dynamic
   dispatch, monomorphisation cost, the `embedded-hal` trait family.
7. **Creating a TMP102 Driver Library and Crate** — `cargo new --lib`, `no_std`,
   generic-over-`embedded-hal::i2c::I2c` driver struct, custom error type,
   doctests, path dependencies, publishing to crates.io.

### Quality & Reliability

9. **Test-Driven Development** — red/green/refactor, `#[cfg(test)]`, unit vs
   integration tests, host-side testing for embedded driver crates,
   hand-rolled mocks against `embedded-hal` traits.
10. **Interrupts** — NVIC, the ARM Cortex-M vector table, the `cortex-m-rt`
    `#[interrupt]` macro, `critical_section`, the
    `Mutex<RefCell<Option<T>>>` shared-state idiom, GPIO EXTI, debouncing.
11. **Logging with `defmt` and Step-through Debugging** — deferred formatting
    and string interning, `probe-rs` / `probe-run`, RTT transport, log levels,
    `.cargo/config.toml` runner, breakpoints and watch via the Cortex-Debug
    VS Code extension.

### Concurrency

12. **Async Programming with Embassy** — cooperative concurrency, `async`/`await`,
    `Future` and `Waker`, the Embassy executor, `#[embassy_executor::main]`,
    `#[embassy_executor::task]`, `embassy_time` timers, async peripherals,
    `embassy_sync::Signal`, comparison to RTIC and FreeRTOS.

## Source material

The full transcripts (produced locally with `mlx_whisper` on Apple Silicon
using the `whisper-large-v3-turbo` model) and cached `.mp3` audio files are
preserved under [`transcripts/`](transcripts/). Keeping both lets you
re-transcribe with a different model later without redownloading from YouTube.

## Hardware referenced throughout the series

- **MCU board:** Raspberry Pi Pico 2 (RP2350, Arm Cortex-M33, `thumbv8m.main-none-eabihf`)
- **Sensor (from Part 5 onwards):** Texas Instruments TMP102 I2C temperature sensor (I2C address `0x48`)
- **Debug probe:** Raspberry Pi Debug Probe (CMSIS-DAP / SWD)
- **Host tooling:** Rust stable, Cargo, `probe-rs`, `picotool`, optional VS Code with the Cortex-Debug or `probe-rs` extension
