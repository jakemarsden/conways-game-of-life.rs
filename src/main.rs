use std::time::Duration;
use std::{fmt, thread};

use structopt::StructOpt;

use crate::game::*;

mod game;

const GLIDER_CELLS: [Position; 5] = [
    Position(1, 0),
    Position(2, 1),
    Position(0, 2),
    Position(1, 2),
    Position(2, 2),
];

#[derive(StructOpt, Debug)]
#[structopt()]
struct CliOptions {
    #[structopt(
        short,
        long,
        default_value = "0",
        help = "Index of the first generation to display (zero-based)"
    )]
    start: usize,
    #[structopt(short, long, help = "Number of generations to display [default: ∞]")]
    count: Option<usize>,
    #[structopt(
        short,
        long,
        default_value = "20",
        help = "Duration to pause between generations (in milliseconds)"
    )]
    period: u64,

    #[structopt(
        short,
        long,
        default_value = "40",
        help = "Number of horizontal cells to simulate"
    )]
    width: usize,
    #[structopt(
        short,
        long,
        default_value = "20",
        help = "Number of vertical cells to simulate"
    )]
    height: usize,
}

fn main() {
    let cli_opts: CliOptions = CliOptions::from_args();

    let start_idx = cli_opts.start;
    let count = cli_opts.count.unwrap_or(usize::MAX);
    let period = Duration::from_millis(cli_opts.period);

    let mut seed_gen = Generation::new(cli_opts.width, cli_opts.height);
    // Start with a glider in the top-left
    for cell_pos in GLIDER_CELLS.iter() {
        seed_gen[*cell_pos] = State::Alive;
    }

    let dump_generation = |(gen_idx, gen): (usize, Generation)| {
        println!("Generation {}", gen_idx);
        println!("{}", gen);
    };
    Generation::generation_iter(seed_gen)
        .enumerate()
        .skip(start_idx)
        .take(count)
        .inspect(|_| thread::sleep(period))
        .for_each(dump_generation);
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
