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
use std::sync::Arc;

use injectium_core::{Container, container};

// Build a container from providers
let c = container! {
    providers: [
        Arc::new(42_u32),
        |c: &Container| format!("value is {}", c.get::<Arc<u32>>().as_ref()),
    ],
};

assert_eq!(*c.get::<Arc<u32>>(), 42);
assert_eq!(c.get::<String>(), "value is 42");
```

## Documentation

See [docs.rs](https://docs.rs/injectium-core) for full API documentation.

## License

[MIT](../../LICENSE). Made with ❤️ by [Ray](https://github.com/so1ve)
