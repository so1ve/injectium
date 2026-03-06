use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use injectium_core::{Container, cloned};

#[derive(Clone)]
struct Payload {
    a: u64,
    b: u64,
}

fn bench_get(c: &mut Criterion) {
    let non_capturing = Container::builder().provider(|_: &Container| 7_u64).build();

    c.bench_function("get/closure_non_capturing_u64", |b| {
        b.iter(|| black_box(non_capturing.get::<u64>()))
    });

    let seed = 11_u64;
    let capturing = Container::builder()
        .provider(move |_: &Container| seed + 3)
        .build();

    c.bench_function("get/closure_capturing_u64", |b| {
        b.iter(|| black_box(capturing.get::<u64>()))
    });

    let shared = Container::builder()
        .provider(cloned(Payload { a: 1, b: 2 }))
        .build();

    c.bench_function("get/clone_payload", |b| {
        b.iter(|| {
            let p = shared.get::<Payload>();
            black_box(p.a.wrapping_add(p.b))
        })
    });
}

criterion_group!(benches, bench_get);
criterion_main!(benches);
