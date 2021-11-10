use criterion::{black_box, Criterion, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_http_date", |b| {
        b.iter(|| {
            let d = black_box("Wed, 21 Oct 2015 07:28:00 GMT");
            black_box(httpdate::parse_http_date(d)).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
