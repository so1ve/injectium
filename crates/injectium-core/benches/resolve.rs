use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use injectium_core::Container;

#[derive(Clone)]
struct Payload {
    a: u64,
    b: u64,
}

fn bench_resolve(c: &mut Criterion) {
    let non_capturing = Container::builder().factory(|_| 7_u64).build();

    c.bench_function("resolve/non_capturing_u64", |b| {
        b.iter(|| black_box(non_capturing.resolve::<u64>()))
    });

    let seed = 11_u64;
    let capturing = Container::builder().factory(move |_| seed + 3).build();

    c.bench_function("resolve/capturing_u64", |b| {
        b.iter(|| black_box(capturing.resolve::<u64>()))
    });

    let singletons = Container::builder()
        .singleton(Payload { a: 1, b: 2 })
        .build();

    c.bench_function("get/singleton_payload", |b| {
        b.iter(|| {
            let p = singletons.get::<Payload>();
            black_box(p.a.wrapping_add(p.b))
        })
    });
}

criterion_group!(benches, bench_resolve);
criterion_main!(benches);
