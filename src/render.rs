use std::fmt;
use std::io::{self, Write};

use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};

use crate::game::*;

pub trait Render<E: fmt::Debug> {
    fn render(&mut self, gen: &Generation) -> Result<(), E>;
}

pub struct TerminalRenderer {
    // TODO: is there a better way to make the ctor private for a zero-field struct?
    _private: (),
}

impl TerminalRenderer {
    pub fn new() -> crossterm::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(TerminalRenderer { _private: () })
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        // FIXME: destructor not called on SIGINT
        // ignore result, don't really care if cleanup fails
        let _ignored = terminal::disable_raw_mode();
    }
}

impl Render<crossterm::ErrorKind> for TerminalRenderer {
    fn render(&mut self, gen: &Generation) -> crossterm::Result<()> {
        let mut out = io::stdout();

        // Clear terminal window
        queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        // Redraw title
        queue!(
            out,
            Print("Generation: "),
            Print(gen.index()),
            MoveToNextLine(2)
        )?;

        // Redraw borders and cells
        for y in 0..gen.height() {
            queue!(out, Print('|'))?;
            for x in 0..gen.width() {
                let position = Position::from((x, y));
                let ch = match gen[position] {
                    Cell::Alive => 'x',
                    Cell::Dead => ' ',
                };
                queue!(out, Print(ch))?;
            }
            queue!(out, Print('|'), MoveToNextLine(1))?;
        }

        out.flush()?;
        Ok(())
    }
}
