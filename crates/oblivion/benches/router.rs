use criterion::{criterion_group, criterion_main, Criterion};
use oblivion::{
    models::{router::Router, session::Session},
    path_route,
    prelude::ServerResponse,
};
use oblivion_codegen::async_route;

#[async_route]
async fn handler(_: Session) -> ServerResponse {
    todo!()
}

fn criterion_benchmark_router(c: &mut Criterion) {
    let mut router = Router::new();
    for i in 0..1000 {
        path_route!(router, &format!("/{}", i) => handler);
    }
    c.bench_function("router", |b| {
        b.iter(|| {
            router.get_handler("/500").unwrap();
        })
    });
}

fn criterion_benchmark_router_less(c: &mut Criterion) {
    let mut router = Router::new();
    for i in 0..10 {
        path_route!(router, &format!("/{}", i) => handler);
    }
    c.bench_function("router_less", |b| {
        b.iter(|| {
            router.get_handler("/5").unwrap();
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark_router,
    criterion_benchmark_router_less
);
criterion_main!(benches);
