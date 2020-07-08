use std::io::{self, Write};
use std::{fmt, mem};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Colorize, PrintStyledContent, Styler};
use crossterm::terminal::{self, Clear, ClearType};

use crate::game::*;

pub trait Render<E: fmt::Debug> {
    fn available_space(&self) -> Option<(usize, usize)>;
    fn render(&mut self, gen: &Generation) -> Result<(), E>;
}

pub struct TerminalRenderer {
    prev_gen: Option<Generation>,
}

impl TerminalRenderer {
    pub fn new() -> crossterm::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { prev_gen: None })
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        // FIXME: destructor not called on SIGINT
        // ignore result, don't really care if cleanup fails
        let _ignored = terminal::disable_raw_mode();
    }
}

impl TerminalRenderer {
    const CELL_OFFSET_X: u16 = Self::BORDER_THICKNESS;
    const CELL_OFFSET_Y: u16 = Self::BORDER_THICKNESS + Self::TITLE_POSITION_Y + 1;
    const BORDER_THICKNESS: u16 = 1;

    const TITLE_POSITION_X: u16 = Self::BORDER_THICKNESS;
    const TITLE_POSITION_Y: u16 = Self::BORDER_THICKNESS;
    const TITLE_TEXT_PREFIX: &'static str = "Generation: ";

    /// - if `curr_gen` is `Some` => redraw the title for `next_gen` only if it differs from `curr_gen`
    /// - if `curr_gen` is `None` => unconditionally redraw the title for `next_gen`
    fn redraw_title_if_needed(
        &mut self,
        next_gen: &Generation,
        curr_gen: Option<&Generation>,
    ) -> crossterm::Result<()> {
        enum RedrawStrategy {
            /// Redraw full title (inc. prefix)
            Full,
            /// Redraw partial title (exc. prefix)
            Partial,
            /// Redraw nothing
            Nop,
        }

        let next_index = next_gen.index();
        let strategy = match curr_gen {
            Some(curr_gen) if next_index == curr_gen.index() => RedrawStrategy::Nop,
            Some(_) => RedrawStrategy::Partial,
            None => RedrawStrategy::Full,
        };

        match strategy {
            RedrawStrategy::Full => {
                let mut out = io::stdout();
                queue!(
                    out,
                    MoveTo(Self::TITLE_POSITION_X, Self::TITLE_POSITION_Y),
                    Clear(ClearType::UntilNewLine),
                    PrintStyledContent(
                        format!("{}{}", Self::TITLE_TEXT_PREFIX, next_index).underlined()
                    )
                )?;
            }
            RedrawStrategy::Partial => {
                let mut out = io::stdout();
                queue!(
                    out,
                    MoveTo(
                        Self::TITLE_POSITION_X + Self::TITLE_TEXT_PREFIX.len() as u16,
                        Self::TITLE_POSITION_Y,
                    ),
                    Clear(ClearType::UntilNewLine),
                    PrintStyledContent(next_index.to_string().underlined())
                )?;
            }
            RedrawStrategy::Nop => {}
        }
        Ok(())
    }

    /// - if `curr_gen` is `Some`, redraw those cells of `next_gen` which differ from those of `curr_gen`
    /// - if `curr_gen` is `None`, unconditionally redraw all the cells of `next_gen`
    fn redraw_changed_cells(
        &mut self,
        next_gen: &Generation,
        curr_gen: Option<&Generation>,
    ) -> crossterm::Result<()> {
        for y in 0..next_gen.height() as u16 {
            for x in 0..next_gen.width() as u16 {
                let position = Position::from((x, y));
                let cell_redraw_needed = match curr_gen {
                    Some(curr_gen) => next_gen[position] != curr_gen[position],
                    None => true,
                };
                if cell_redraw_needed {
                    self.redraw_cell((x, y), &next_gen[position])?;
                }
            }
        }
        Ok(())
    }

    fn redraw_cell(&mut self, (x, y): (u16, u16), cell: &Cell) -> crossterm::Result<()> {
        let cell_display = match cell {
            Cell::Alive => '•'.bold().dark_green().on_black(),
            Cell::Dead => ' '.on_black(),
        };
        let mut out = io::stdout();
        queue!(
            out,
            MoveTo(x + Self::CELL_OFFSET_X, y + Self::CELL_OFFSET_Y),
            PrintStyledContent(cell_display),
        )?;
        Ok(())
    }
}

impl Render<crossterm::ErrorKind> for TerminalRenderer {
    fn available_space(&self) -> Option<(usize, usize)> {
        let (term_width, term_height) = terminal::size().ok()?;
        let (avail_width, avail_height) = (
            term_width - Self::CELL_OFFSET_X - Self::BORDER_THICKNESS,
            term_height - Self::CELL_OFFSET_Y - Self::BORDER_THICKNESS,
        );
        Some((avail_width as usize, avail_height as usize))
    }

    fn render(&mut self, next_gen: &Generation) -> crossterm::Result<()> {
        let mut curr_gen = Some(next_gen.clone());
        mem::swap(&mut curr_gen, &mut self.prev_gen);

        let curr_gen = curr_gen.as_ref();
        let (width, height) = (next_gen.width() as u16, next_gen.height() as u16);

        // we can get away with a partial redraw if
        //     1. not specifically asked to redraw everything from scratch (e.g. on the first draw)
        //     2. the next_gen is the same size as the curr_gen (and we actually have a curr_gen)
        let full_redraw_needed = match curr_gen {
            Some(curr_gen) => {
                width as usize != curr_gen.width() || height as usize != curr_gen.height()
            }
            None => true,
        };

        let mut out = io::stdout();
        if full_redraw_needed {
            queue!(out, Clear(ClearType::All))?;
            self.redraw_title_if_needed(next_gen, None)?;
            self.redraw_changed_cells(next_gen, None)?;
        } else {
            self.redraw_title_if_needed(next_gen, curr_gen)?;
            self.redraw_changed_cells(next_gen, curr_gen)?;
        }

        out.flush()?;
        Ok(())
    }
}
