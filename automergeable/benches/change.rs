use std::collections::HashMap;

use automergeable::Document;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_single(mut document: Document<HashMap<String, String>>, limit: usize) {
    black_box(
        document
            .change::<_, _, std::convert::Infallible>(|doc| {
                for i in 0..limit {
                    doc.insert(i.to_string(), i.to_string());
                }
                Ok(())
            })
            .unwrap(),
    );
}

fn bench_many(mut document: Document<HashMap<String, String>>, limit: usize) {
    #[allow(clippy::unit_arg)]
    black_box(for i in 0..limit {
        document
            .change::<_, _, std::convert::Infallible>(|doc| {
                doc.insert(i.to_string(), i.to_string());

                Ok(())
            })
            .unwrap();
    });
}

fn bench_changes_single(c: &mut Criterion) {
    for &i in &[100, 1000, 10000] {
        c.bench_function(&format!("change with {} entries, single change", i), |b| {
            b.iter_batched(
                Document::<HashMap<String, String>>::new,
                |doc| bench_single(doc, i),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn bench_changes_many(c: &mut Criterion) {
    for &i in &[100, 1000, 10000] {
        c.bench_function(&format!("change with {} entries, many changes", i), |b| {
            b.iter_batched(
                Document::<HashMap<String, String>>::new,
                |doc| bench_many(doc, i),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench_changes_single, bench_changes_many);
criterion_main!(benches);
