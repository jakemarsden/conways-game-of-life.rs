use std::time::Duration;

use rand::distributions::{Bernoulli, BernoulliError};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use structopt::StructOpt;

use crate::game::Cell;

mod display;
mod game;

const FALLBACK_WIDTH: usize = 40;
const FALLBACK_HEIGHT: usize = 20;

#[derive(StructOpt, Debug)]
#[structopt()]
struct CliOptions {
    #[structopt(
        long,
        help = "Seed for the PRNG which produces the first generation [default: random]"
    )]
    seed: Option<u64>,

    #[structopt(
        long,
        default_value = "0.5",
        help = "Probability that a cell will be alive in the first generation"
    )]
    weight: f32,

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

fn main() -> app::Result<()> {
    let cli_opts: CliOptions = CliOptions::from_args();

    let preferred_size = match (cli_opts.width, cli_opts.height) {
        (Some(w), Some(h)) => Some((w, h)),
        (None, None) => None,
        // TODO: better error handling than for bad user input
        (_, _) => panic!("Bad user args: Must provide exactly 0 or 2 of: [width, height]"),
    };

    // TODO: better error handing for user-supplied weight outside [0.0, 1.0]
    let cell_gen = weighted_cell_generator(cli_opts.weight, cli_opts.seed)
        .expect("Bad user args: Weight must be in the range [0.0, 1.0]");

    let app = app::App::new(
        cli_opts.start,
        cli_opts.step,
        cli_opts.count.unwrap_or(usize::MAX),
        preferred_size,
        Duration::from_millis(cli_opts.period),
        cell_gen,
    )?;

    app.run_to_completion()?;

    Ok(())
}

fn weighted_cell_generator(
    weight: f32,
    seed: Option<u64>,
) -> Result<impl FnMut() -> Cell, BernoulliError> {
    let distr = Bernoulli::new(weight.into())?;
    let rng = seed
        .map(SmallRng::seed_from_u64)
        .unwrap_or_else(SmallRng::from_entropy);
    let mut cell_iter =
        rng.sample_iter::<bool, _>(distr)
            .map(|alive| if alive { Cell::Alive } else { Cell::Dead });
    Ok(move || {
        cell_iter
            .next()
            .expect("Expected Rng::sample_iter(Distribution) to be infinite")
    })
}

mod app {
    use std::time::Duration;
    use std::{fmt, thread};

    use crate::display::*;
    use crate::game::*;

    #[derive(Debug)]
    pub enum Error {
        Display(crossterm::ErrorKind),
    }

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub enum Action {
        Restart,
        Exit,
        Unmapped,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum State {
        Initial,
        Running,
        Finished,
    }

    pub struct App<F>
    where
        F: FnMut() -> Cell,
    {
        start: usize,
        step: usize,
        count: usize,
        curr_count: usize,
        size: (usize, usize),
        period: Duration,
        cell_gen: F,
        state: State,
        generation: Generation,
        display: TerminalDisplay,
    }

    impl<F> App<F>
    where
        F: FnMut() -> Cell,
    {
        pub fn new(
            start: usize,
            step: usize,
            count: usize,
            preferred_size: Option<(usize, usize)>,
            period: Duration,
            mut cell_gen: F,
        ) -> Result<Self> {
            let display = TerminalDisplay::new().map_err(Error::from)?;
            let (width, height) = preferred_size
                .or_else(|| {
                    display
                        .available_cells()
                        .map(|(width, height)| (width as usize, height as usize))
                })
                .unwrap_or_else(|| (super::FALLBACK_WIDTH, super::FALLBACK_HEIGHT));

            let seed_gen = Generation::generate(0, width, height, &mut cell_gen);
            let generation = Generation::nth_after(&seed_gen, start);
            Ok(Self {
                start,
                step,
                count: count - 1,
                curr_count: count - 1,
                size: (width, height),
                period,
                cell_gen,
                state: State::Initial,
                generation,
                display,
            })
        }

        pub fn run_to_completion(mut self) -> Result<()> {
            loop {
                match self.state {
                    State::Initial => {
                        self.render()?;
                        self.state = State::Running;
                    }
                    State::Running => {
                        self.handle_input()?;
                        self.update()?;
                        self.render()?;
                    }
                    State::Finished => {
                        break;
                    }
                }
                if self.state == State::Running {
                    thread::sleep(self.period);
                }
            }
            Ok(())
        }

        fn handle_input(&mut self) -> Result<()> {
            while let Some(ev) = self.display.take_pending_event()? {
                let action = Action::from(ev);
                match action {
                    Action::Restart => {
                        let (width, height) = self.size;
                        let seed_gen = Generation::generate(0, width, height, &mut self.cell_gen);
                        self.generation = Generation::nth_after(&seed_gen, self.start);
                        self.curr_count = self.count;
                        self.state = State::Initial;
                    }
                    Action::Exit => {
                        self.state = State::Finished;
                    }
                    Action::Unmapped => {}
                }
            }
            Ok(())
        }

        fn update(&mut self) -> Result<()> {
            if self.curr_count != 0 {
                self.curr_count -= 1;
                self.generation = Generation::nth_after(&self.generation, self.step);
            } else {
                self.state = State::Finished;
            }
            Ok(())
        }

        fn render(&mut self) -> Result<()> {
            self.display.draw(&self.generation).map_err(Error::from)
        }
    }

    impl From<Event> for Action {
        fn from(ev: Event) -> Self {
            match ev {
                Event::Key(key_ev) => Self::from(key_ev),
                _ => Self::Unmapped,
            }
        }
    }

    impl From<KeyEvent> for Action {
        fn from(key_ev: KeyEvent) -> Self {
            match key_ev.code {
                KeyCode::Char('r') => Self::Restart,
                KeyCode::Char('q') | KeyCode::Esc => Self::Exit,
                KeyCode::Char('c') if key_ev.modifiers.contains(KeyModifiers::CONTROL) => {
                    Self::Exit
                }
                _ => Self::Unmapped,
            }
        }
    }

    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Display(err) => Some(err),
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Display(err) => fmt::Display::fmt(err, f),
            }
        }
    }

    impl From<crossterm::ErrorKind> for Error {
        fn from(source: crossterm::ErrorKind) -> Self {
            Self::Display(source)
        }
    }
}
