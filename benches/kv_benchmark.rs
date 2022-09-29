use std::io::Cursor;

use anode_kv::codec::decode;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

fn kv_benchmark(c: &mut Criterion) {
    c.bench_function("parse_string", |b| {
        b.iter_batched_ref(
            || {
                let line = b"+Hello\r\n";
                Cursor::new(&line[..])
            },
            |mut cursor| decode(&mut cursor),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, kv_benchmark);
criterion_main!(benches);
