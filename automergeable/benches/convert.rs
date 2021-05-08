use std::collections::HashMap;

use automerge::Value;
use automergeable::{FromAutomerge, ToAutomerge};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hashmap(h: HashMap<String, String>) {
    #[allow(clippy::unit_arg)]
    black_box({
        let ha = h.to_automerge();
        HashMap::<String, String>::from_automerge(&ha).unwrap();
    });
}

fn bench_hashmap_to(h: HashMap<String, String>) {
    #[allow(clippy::unit_arg)]
    black_box(h.to_automerge());
}

fn bench_hashmap_from(h: Value) {
    #[allow(clippy::unit_arg)]
    black_box(HashMap::<String, String>::from_automerge(&h).unwrap());
}

fn bench_convert_hashmap(c: &mut Criterion) {
    for &limit in &[100, 1000, 10000] {
        c.bench_function(
            &format!("convert hashmap with {} entries to and from", limit),
            |b| {
                b.iter_batched(
                    || {
                        let mut h = HashMap::new();
                        for i in 0..limit {
                            h.insert(i.to_string(), i.to_string());
                        }
                        h
                    },
                    bench_hashmap,
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
}

fn bench_convert_hashmap_to(c: &mut Criterion) {
    for &limit in &[100, 1000, 10000] {
        c.bench_function(&format!("convert hashmap with {} entries to", limit), |b| {
            b.iter_batched(
                || {
                    let mut h = HashMap::new();
                    for i in 0..limit {
                        h.insert(i.to_string(), i.to_string());
                    }
                    h
                },
                bench_hashmap_to,
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn bench_convert_hashmap_from(c: &mut Criterion) {
    for &limit in &[100, 1000, 10000] {
        c.bench_function(
            &format!("convert hashmap with {} entries from", limit),
            |b| {
                b.iter_batched(
                    || {
                        let mut h = HashMap::new();
                        for i in 0..limit {
                            h.insert(i.to_string(), i.to_string());
                        }
                        h.to_automerge()
                    },
                    bench_hashmap_from,
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
}

criterion_group!(
    benches,
    bench_convert_hashmap,
    bench_convert_hashmap_to,
    bench_convert_hashmap_from
);
criterion_main!(benches);
