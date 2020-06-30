use std::time::Duration;
use std::{fmt, thread};

use crate::game::*;

mod game;

fn main() {
    let mut seed_gen = Generation::new(10, 10);
    // Start with a glider near the centre
    seed_gen[Position(5, 6)] = State::Alive;
    seed_gen[Position(6, 7)] = State::Alive;
    seed_gen[Position(4, 8)] = State::Alive;
    seed_gen[Position(5, 8)] = State::Alive;
    seed_gen[Position(6, 8)] = State::Alive;

    let gens = Generation::generation_iter(seed_gen);
    for (gen_idx, gen) in gens.enumerate() {
        println!("Generation {}", gen_idx);
        println!("{}", gen);
        thread::sleep(Duration::from_millis(500));
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height() {
            write!(f, "|")?;
            for x in 0..self.width() {
                let ch = match self[Position(x, y)] {
                    State::Dead => ' ',
                    State::Alive => 'x',
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}
