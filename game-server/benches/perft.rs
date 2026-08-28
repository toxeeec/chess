use std::env;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use game_server::Perft;
use serde::Deserialize;

#[derive(Deserialize)]
struct PerftCase {
    depth: u32,
    fen: String,
}

fn benchmark_perft(c: &mut Criterion) {
    let cases = env::var("PERFT_CASES").expect("PERFT_CASES must be set by the benchmark runner");
    let label = env::var("PERFT_LABEL").expect("PERFT_LABEL must be set by the benchmark runner");
    let cases: Vec<PerftCase> =
        serde_json::from_str(&cases).expect("PERFT_CASES must be valid JSON");
    assert!(!cases.is_empty(), "PERFT_CASES must not be empty");
    let perfts: Vec<_> = cases
        .into_iter()
        .map(|case| {
            let perft = Perft::new(&case.fen).expect("PERFT_CASES must contain valid FENs");
            (perft, case.depth)
        })
        .collect();
    let nodes = perfts.iter().map(|(perft, depth)| perft.run(*depth)).sum();
    let mut group = c.benchmark_group("perft");
    group.throughput(Throughput::Elements(nodes));

    group.bench_function(label, |b| {
        b.iter(|| {
            perfts
                .iter()
                .map(|(perft, depth)| perft.run(*depth))
                .sum::<u64>()
        });
    });
    group.finish();
}

criterion_group!(benches, benchmark_perft);
criterion_main!(benches);
