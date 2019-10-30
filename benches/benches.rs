#[macro_use]
extern crate criterion;
extern crate typed_arena;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};

#[derive(Default)]
struct Small(usize);

#[derive(Default)]
struct Big([usize; 32]);

fn allocate<T: Default>(n: usize) {
    let arena = typed_arena::Arena::new();
    for _ in 0..n {
        let val: &mut T = arena.alloc(Default::default());
        criterion::black_box(val);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "allocate",
        ParameterizedBenchmark::new(
            "allocate-small",
            |b, n| b.iter(|| allocate::<Small>(*n)),
            (1..5).map(|n| n * 1000).collect::<Vec<usize>>(),
        )
        .throughput(|n| Throughput::Elements(*n as u64)),
    );

    c.bench(
        "allocate",
        ParameterizedBenchmark::new(
            "allocate-big",
            |b, n| b.iter(|| allocate::<Big>(*n)),
            (1..5).map(|n| n * 1000).collect::<Vec<usize>>(),
        )
        .throughput(|n| Throughput::Elements(*n as u64)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
