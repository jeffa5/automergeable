use std::collections::HashMap;

use automergeable::Document;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut document = Document::<HashMap<String, String>>::new();
    c.bench_function("change", |b| {
        b.iter(|| {
            black_box(document.change::<_, std::convert::Infallible>(|doc| {
                for i in 0..100 {
                    doc.insert(i.to_string(), i.to_string());
                }
                Ok(())
            }))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
