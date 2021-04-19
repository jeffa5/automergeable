use std::collections::HashMap;

use automerge::{Primitive, Value};
use automergeable::ToAutomerge;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut m = HashMap::new();
    for i in 0..100 {
        m.insert(i.to_string(), i.to_string());
    }
    let value = m.to_automerge();
    c.bench_function("diff", |b| {
        b.iter(|| {
            black_box(automergeable::diff_values(
                &value,
                &Value::Primitive(Primitive::Null),
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
