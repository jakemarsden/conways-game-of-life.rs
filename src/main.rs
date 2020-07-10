use std::time::Duration;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use structopt::StructOpt;

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

fn main() -> app::Result<()> {
    let cli_opts: CliOptions = CliOptions::from_args();

    let preferred_size = match (cli_opts.width, cli_opts.height) {
        (Some(w), Some(h)) => Some((w, h)),
        (None, None) => None,
        // TODO: better error handling than just `panic!()`ing !!
        (_, _) => panic!("Must provide exactly 0 or 2 of: [width, height]"),
    };

    let mut rng = SmallRng::from_entropy();

    let app = app::App::new(
        cli_opts.start,
        cli_opts.step,
        cli_opts.count.unwrap_or(usize::MAX),
        preferred_size,
        Duration::from_millis(cli_opts.period),
        &mut rng,
    )?;

    app.run_to_completion()?;

    Ok(())
}

mod app {
    use std::time::Duration;
    use std::{fmt, thread};

    use rand::Rng;

    use crate::display::*;
    use crate::game::*;

    #[derive(Debug)]
    pub enum Error {
        Display(crossterm::ErrorKind),
    }

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub enum Action {
        Exit,
        Unmapped,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum State {
        Initial,
        Running,
        Finished,
    }

    pub struct App {
        step: usize,
        count: usize,
        period: Duration,
        state: State,
        generation: Generation,
        display: TerminalDisplay,
    }

    impl App {
        pub fn new<R>(
            start: usize,
            step: usize,
            count: usize,
            preferred_size: Option<(usize, usize)>,
            period: Duration,
            rng: &mut R,
        ) -> Result<Self>
        where
            R: Rng + ?Sized,
        {
            let display = TerminalDisplay::new().map_err(Error::from)?;
            let (width, height) = preferred_size
                .or_else(|| {
                    display
                        .available_cells()
                        .map(|(width, height)| (width as usize, height as usize))
                })
                .unwrap_or_else(|| (super::FALLBACK_WIDTH, super::FALLBACK_HEIGHT));

            let seed_gen = Generation::random(0, width, height, rng);
            let generation = Generation::nth_after(&seed_gen, start);
            Ok(Self {
                step,
                count: count - 1,
                period,
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
                    Action::Exit => self.state = State::Finished,
                    Action::Unmapped => {}
                }
            }
            Ok(())
        }

        fn update(&mut self) -> Result<()> {
            if self.count != 0 {
                self.count -= 1;
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
