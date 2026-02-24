# injectium-salvo

[![Crates.io Version](https://img.shields.io/crates/v/injectium-salvo)](https://crates.io/crates/injectium-salvo)
[![docs.rs](https://img.shields.io/docsrs/injectium-salvo)](https://docs.rs/injectium-salvo)

Salvo integration for Injectium dependency injection.

## Installation

```bash
cargo add injectium-salvo
```

## Quick Start

```rust
use injectium::{container, Injectable};
use injectium_salvo::{inject_container, Injected};
use salvo::prelude::*;
use std::sync::Arc;

// Define your services
#[derive(Clone, Injectable)]
struct DbService {
    connection_string: String,
}

#[derive(Injectable)]
struct UserService {
    db: DbService,
}

// Build the container
let container = Arc::new(container! {
    singletons: [
        DbService { connection_string: "postgres://localhost".into() },
    ],
});

// Handler with dependency injection
#[handler]
async fn hello_user(user: Injected<UserService>) -> String {
    format!("User service DB: {}", user.db.connection_string)
}

let router = Router::new()
    .hoop(inject_container(container))
    .push(Router::with_path("/hello").get(hello_user));
```

## Documentation

See [docs.rs](https://docs.rs/injectium-salvo) for full API documentation.

## Features

- `oapi` – Enable OpenAPI support for `Injected<T>` types

## License

[MIT](../LICENSE)
