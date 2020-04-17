mod game;
mod gameboards;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use game::Game;
use gameboards::*;

fn bench_easy(c: &mut Criterion) {
    c.bench_function("easy", move |b| {
        b.iter_with_setup(|| Game::new(EASY), |mut game| black_box(game.solve()));
    });
}

fn bench_medium(c: &mut Criterion) {
    c.bench_function("medium", move |b| {
        b.iter_with_setup(|| Game::new(MEDIUM), |mut game| black_box(game.solve()));
    });
}

fn bench_hard(c: &mut Criterion) {
    c.bench_function("hard", move |b| {
        b.iter_with_setup(|| Game::new(HARD), |mut game| black_box(game.solve()));
    });
}

fn bench_seventeen(c: &mut Criterion) {
    c.bench_function("seventeen", move |b| {
        b.iter_with_setup(|| Game::new(SEVENTEEN), |mut game| black_box(game.solve()));
    });
}

fn bench_zeros(c: &mut Criterion) {
    c.bench_function("zeros", move |b| {
        b.iter_with_setup(|| Game::new(ZEROS), |mut game| black_box(game.solve()));
    });
}

criterion_group! {
    name = bench;
    config = Criterion::default();
    targets = bench_easy, bench_medium, bench_hard, bench_seventeen, bench_zeros
}

criterion_main!(bench);
