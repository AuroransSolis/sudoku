mod game;
#[allow(dead_code)]
mod gameboards;

use game::Game;
use gameboards::*;
use std::time::Instant;

fn main() {
    let mut game = Game::new(SEVENTEEN);
    println!("{}", game);
    let start = Instant::now();
    game.solve();
    let elapsed = start.elapsed();
    println!("{}", game);
    println!("Time taken: {:?}", elapsed);
}
