use std::{env, fs};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use game_server::GameReplay;

fn benchmark_games(c: &mut Criterion) {
    let dataset_id =
        env::var("GAME_DATASET_ID").expect("GAME_DATASET_ID must be set by the benchmark runner");
    let dataset_path = env::var("GAME_DATASET_PATH")
        .expect("GAME_DATASET_PATH must be set by the benchmark runner");
    let dataset = fs::read_to_string(dataset_path).expect("game dataset must be readable");
    let replay = GameReplay::new(&dataset).expect("game dataset must be valid");
    let moves = replay.run();
    let mut group = c.benchmark_group("games");
    group.throughput(Throughput::Elements(moves as u64));

    group.bench_function(dataset_id, |b| {
        b.iter(|| replay.run());
    });
    group.finish();
}

criterion_group!(benches, benchmark_games);
criterion_main!(benches);
