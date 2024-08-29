use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use oblivion::models::{client::Client, router::Router, server::Server};
use tokio::runtime::Runtime;

async fn connect() -> Result<()> {
    let client = Client::connect("oblivion://127.0.0.1:7076").await?;
    client.recv().await?;
    client.close().await?;
    Ok(())
}

fn criterion_benchmark_socket(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.spawn(async move {
        Server::new("127.0.0.1", 7076, Router::new()).run().await.unwrap()
    });
    c.bench_function("socket", |b| {
        b.to_async(&rt)
            .iter(|| async { connect().await })
    });
    server.abort();
}

criterion_group!(benches, criterion_benchmark_socket);
criterion_main!(benches);
