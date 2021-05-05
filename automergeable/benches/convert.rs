use std::collections::HashMap;

use automergeable::{FromAutomerge, ToAutomerge};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hashmap(h: HashMap<String, String>) {
    #[allow(clippy::unit_arg)]
    black_box({
        let ha = h.to_automerge();
        HashMap::<String, String>::from_automerge(&ha).unwrap();
    });
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

criterion_group!(benches, bench_convert_hashmap);
criterion_main!(benches);
