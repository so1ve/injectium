# injectium-core

[![Crates.io Version](https://img.shields.io/crates/v/injectium-core)](https://crates.io/crates/injectium-core)
[![docs.rs](https://img.shields.io/docsrs/injectium-core)](https://docs.rs/injectium-core)

Core dependency-injection container implementation for Rust.

## Installation

```bash
cargo add injectium-core
```

## Quick Start

```rust
use injectium_core::{Container, container};

// Build a container with singletons and/or factories
let c = container! {
    singletons: [
        42_u32,
        String::from("hello"),
    ],
    providers: [
        |c| format!("value is {}", c.get::<u32>()),
    ],
};

// Retrieve a singleton
assert_eq!(*c.get::<u32>(), 42);

// Resolve a factory (produces a new value each time)
assert_eq!(c.resolve::<String>(), "value is 42");
```

## Documentation

See [docs.rs](https://docs.rs/injectium-core) for full API documentation.

## License

[MIT](../LICENSE). Made with ❤️ by [Ray](https://github.com/so1ve)
