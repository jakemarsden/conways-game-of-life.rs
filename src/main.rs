use std::thread;
use std::time::Duration;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use structopt::StructOpt;

use crate::display::*;
use crate::game::*;

mod display;
mod game;

const FALLBACK_WIDTH: usize = 40;
const FALLBACK_HEIGHT: usize = 20;

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
        help = "Number of horizontal cells to simulate [default: terminal-width]"
    )]
    width: Option<usize>,

    #[structopt(
        short,
        long,
        help = "Number of vertical cells to simulate [default: terminal-height]"
    )]
    height: Option<usize>,
}

fn main() -> crossterm::Result<()> {
    let cli_opts: CliOptions = CliOptions::from_args();
    let period = Duration::from_millis(cli_opts.period);

    let mut display = TerminalDisplay::new()?;
    let available_space = display.available_cells();
    let width = cli_opts
        .width
        .or_else(|| available_space.map(|(x, _)| x))
        .unwrap_or(FALLBACK_WIDTH);
    let height = cli_opts
        .height
        .or_else(|| available_space.map(|(_, y)| y))
        .unwrap_or(FALLBACK_HEIGHT);

    let mut rng = SmallRng::from_entropy();
    let seed_generation = Generation::random(0, width, height, &mut rng);

    for gen in Generation::generation_iter(seed_generation)
        .skip(cli_opts.start)
        .step_by(cli_opts.step)
        .take(cli_opts.count.unwrap_or(usize::MAX))
    {
        thread::sleep(period);
        display.draw(&gen)?;
    }
    Ok(())
}
