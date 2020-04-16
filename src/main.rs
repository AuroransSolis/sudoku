mod game;

use game::Game;
use std::time::Instant;

fn main() {
    let mut easy = Game::new([
        [2, 0, 5, 0, 0, 9, 0, 0, 4],
        [0, 0, 0, 0, 0, 0, 3, 0, 7],
        [7, 0, 0, 8, 5, 6, 0, 1, 0],
        [4, 5, 0, 7, 0, 0, 0, 0, 0],
        [0, 0, 9, 0, 0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0, 2, 0, 8, 5],
        [0, 2, 0, 4, 1, 8, 0, 0, 6],
        [6, 0, 8, 0, 0, 0, 0, 0, 0],
        [1, 0, 0, 2, 0, 0, 7, 0, 8],
    ]);
    let mut medium = Game::new([
        [0, 0, 6, 0, 9, 0, 2, 0, 0],
        [0, 0, 0, 7, 0, 2, 0, 0, 0],
        [0, 9, 0, 5, 0, 8, 0, 7, 0],
        [9, 0, 0, 0, 3, 0, 0, 0, 6],
        [7, 5, 0, 0, 0, 0, 0, 1, 9],
        [1, 0, 0, 0, 4, 0, 0, 0, 5],
        [0, 1, 0, 3, 0, 9, 0, 8, 0],
        [0, 0, 0, 2, 0, 1, 0, 0, 0],
        [0, 0, 9, 0, 8, 0, 1, 0, 0],
    ]);
    let mut hard = Game::new([
        [0, 0, 0, 8, 0, 0, 0, 0, 0],
        [7, 8, 9, 0, 1, 0, 0, 0, 6],
        [0, 0, 0, 0, 0, 6, 1, 0, 0],
        [0, 0, 7, 0, 0, 0, 0, 5, 0],
        [5, 0, 8, 7, 0, 9, 3, 0, 4],
        [0, 4, 0, 0, 0, 0, 2, 0, 0],
        [0, 0, 3, 2, 0, 0, 0, 0, 0],
        [8, 0, 0, 0, 7, 0, 4, 3, 9],
        [0, 0, 0, 0, 0, 1, 0, 0, 0],
    ]);
    let euss = format!("{}", easy);
    let muss = format!("{}", medium);
    let huss = format!("{}", hard);
    let start = Instant::now();
    easy.solve();
    medium.solve();
    hard.solve();
    let elapsed = start.elapsed();
    let ess = format!("{}", easy);
    let mss = format!("{}", medium);
    let hss = format!("{}", hard);
    for ((euss_line, muss_line), huss_line) in euss.lines().zip(muss.lines()).zip(huss.lines()) {
        println!("{}    {}    {}", euss_line, muss_line, huss_line);
    }
    for ((ess_line, mss_line), hss_line) in ess.lines().zip(mss.lines()).zip(hss.lines()) {
        println!("{}    {}    {}", ess_line, mss_line, hss_line);
    }
    println!("solve time for all: {:?}", elapsed);
}
