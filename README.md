# sly_static

Seamless Rust Static Initialization: Effortless and Efficient

## What is this?
This crate allows you to initialize static variables easily (automaticlly) inside main function.

## How does it work?

Uses the [linkme](https://crates.io/crates/linkme) crate to collect all the static variables and initialize them inside the main function.
It makes use of some code from [ctor](https://crates.io/crates/ctor) to initialize the statics.

## Warning
It uses unsafe code to initialize the statics, but it's safe to use unless you do some weird magic with it.

I say it's safe because all the statics are initialized inside the main function, so there's no way to access them before they are initialized.

## Example
```rust
use sly_static::sly_static;
use sly_static::sly_main;

#[sly_static]
static MY_STATIC: String = String::from("Hello, World!");

#[sly_main]
fn main() {
    println!("{}", *MY_STATIC);
}
```

License: MIT
