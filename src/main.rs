use std::time::Duration;
use std::{fmt, thread};

use rand::rngs::SmallRng;
use rand::SeedableRng;
use structopt::StructOpt;

use crate::game::*;

mod game;

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

    #[structopt(
        short = "N",
        long,
        default_value = "1",
        help = "Display only every Nth generation"
    )]
    step: usize,

    #[structopt(short, long, help = "Number of generations to display [default: ∞]")]
    count: Option<usize>,

    #[structopt(
        short,
        long,
        default_value = "20",
        help = "Duration to pause after displaying each generation (in milliseconds)"
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
    let period = Duration::from_millis(cli_opts.period);

    let mut rng = SmallRng::from_entropy();
    let seed_generation = Generation::random(cli_opts.width, cli_opts.height, &mut rng);

    let dump_generation = |(generation_idx, generation): (usize, Generation)| {
        println!("Generation {}\n{}", generation_idx, generation)
    };
    Generation::generation_iter(seed_generation)
        .enumerate()
        .skip(cli_opts.start)
        .step_by(cli_opts.step)
        .take(cli_opts.count.unwrap_or(usize::MAX))
        .inspect(|_| thread::sleep(period))
        .for_each(dump_generation);
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height() {
            write!(f, "|")?;
            for x in 0..self.width() {
                let ch = match self[Position(x, y)] {
                    Cell::Alive => 'x',
                    Cell::Dead => ' ',
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}
