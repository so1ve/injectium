# injectium

[![Crates.io Version](https://img.shields.io/crates/v/injectium)](https://crates.io/crates/injectium)
[![docs.rs](https://img.shields.io/docsrs/injectium)](https://docs.rs/injectium)

A minimal dependency-injection implementation for Rust.

## Installation

```bash
cargo add injectium
```

## Quick Start

```rust
use injectium::{Injectable, cloned, container};

#[derive(Clone)]
struct Db {
    conn: String,
}

#[derive(Injectable)]
struct Service {
    db: Db,
}

// At startup, build the container
let c = container! {
    providers: [
        cloned(Db { conn: "postgres://localhost".into() }),
    ],
};

// Validate everything is wired up
c.validate();

// Later, resolve services
let svc = Service::from_container(&c);
```

## Documentation

See [docs.rs](https://docs.rs/injectium) for full API documentation.

## Crates

| Crate | Description |
|-------|-------------|
| [`injectium`](https://crates.io/crates/injectium) | Main crate with derive macro |
| [`injectium-core`](https://crates.io/crates/injectium-core) | Core container implementation |
| [`injectium-macro`](https://crates.io/crates/injectium-macro) | Procedural macros |
| [`injectium-salvo`](https://crates.io/crates/injectium-salvo) | Salvo web framework integration |

## License

[MIT](../../LICENSE). Made with ❤️ by [Ray](https://github.com/so1ve)
