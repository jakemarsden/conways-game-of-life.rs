use std::{iter, ops};

use rand::Rng;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position(pub isize, pub isize);

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Cell {
    Alive,
    Dead,
}

pub struct Generation {
    width: usize,
    height: usize,
    index: usize,
    cells: Vec<Cell>,
}

impl Position {
    pub fn x(&self) -> isize {
        self.0
    }

    pub fn y(&self) -> isize {
        self.1
    }
}

impl ops::Add<Self> for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl ops::AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.x();
        self.1 += rhs.y();
    }
}

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Self(x as isize, y as isize)
    }
}

impl Cell {
    pub fn is_alive(&self) -> bool {
        match self {
            Self::Alive => true,
            Self::Dead => false,
        }
    }
}

impl ops::Not for Cell {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Alive => Self::Dead,
            Self::Dead => Self::Alive,
        }
    }
}

impl Generation {
    pub fn generation_iter(seed: Generation) -> impl Iterator<Item = Self> {
        iter::successors(Some(seed), |prev| Some(prev.next()))
    }

    pub fn filled(index: usize, width: usize, height: usize, filler: Cell) -> Self {
        let cells = vec![filler; width * height];
        Self {
            width,
            height,
            index,
            cells,
        }
    }

    pub fn random<R>(index: usize, width: usize, height: usize, rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let mut cells = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            let cell_state = if rng.gen() { Cell::Alive } else { Cell::Dead };
            cells.push(cell_state);
        }
        Self {
            width,
            height,
            index,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn next(&self) -> Self {
        let mut next = Self::filled(self.index() + 1, self.width(), self.height(), Cell::Dead);
        for y in 0..self.height() {
            for x in 0..self.width() {
                let position = Position::from((x, y));
                let live_neighbour_count = self
                    .neighbouring_cells(position)
                    .iter()
                    .filter(|cell| cell.is_alive())
                    .count();
                next[position] = match (live_neighbour_count, self[position]) {
                    (3, _) => Cell::Alive,
                    (2, Cell::Alive) => Cell::Alive,
                    _ => Cell::Dead,
                };
            }
        }
        next
    }

    pub fn neighbouring_cells(&self, relative_to: Position) -> Vec<Cell> {
        static OFFSETS: [Position; 8] = [
            Position(-1, -1),
            Position(0, -1),
            Position(1, -1),
            Position(-1, 0),
            Position(1, 0),
            Position(-1, 1),
            Position(0, 1),
            Position(1, 1),
        ];
        OFFSETS
            .iter()
            .map(|offset| self[relative_to + *offset])
            .collect()
    }

    fn cell_idx(&self, position: Position) -> usize {
        let x = if position.x() < 0 {
            (position.x() + self.width() as isize) as usize
        } else {
            position.x() as usize % self.width()
        };
        let y = if position.y() < 0 {
            (position.y() + self.height() as isize) as usize
        } else {
            position.y() as usize % self.height()
        };
        x + y * self.width()
    }
}

impl ops::Index<Position> for Generation {
    type Output = Cell;

    /// Index will wrap around if outside of `[0, self.width)`, `[0, self.height)`
    fn index(&self, index: Position) -> &Self::Output {
        let idx = self.cell_idx(index);
        &self.cells[idx]
    }
}

impl ops::IndexMut<Position> for Generation {
    /// Index will wrap around if outside of `[0, self.width)`, `[0, self.height)`
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        let idx = self.cell_idx(index);
        &mut self.cells[idx]
    }
}
