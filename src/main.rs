use std::thread;
use std::time::Duration;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use structopt::StructOpt;

use crate::game::*;
use crate::render::*;

mod game;
mod render;

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

fn main() -> crossterm::Result<()> {
    let cli_opts: CliOptions = CliOptions::from_args();
    let period = Duration::from_millis(cli_opts.period);

    let mut rng = SmallRng::from_entropy();
    let seed_generation = Generation::random(0, cli_opts.width, cli_opts.height, &mut rng);

    let mut renderer = TerminalRenderer::new()?;
    for gen in Generation::generation_iter(seed_generation)
        .skip(cli_opts.start)
        .step_by(cli_opts.step)
        .take(cli_opts.count.unwrap_or(usize::MAX))
    {
        thread::sleep(period);
        renderer.render(&gen)?;
    }
    Ok(())
}
