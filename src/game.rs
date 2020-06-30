use std::{iter, ops};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position(pub usize, pub usize);

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Cell {
    Alive,
    Dead,
}

pub struct Generation {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Cell {
    pub fn is_alive(self) -> bool {
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

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Dead; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn next(&self) -> Generation {
        let mut next_cells = Vec::with_capacity(self.cells.len());
        for y in 0..self.height() {
            for x in 0..self.width() {
                let live_neighbour_count = self
                    .neighbouring_cells(Position(x, y))
                    .iter()
                    .filter(|cell| cell.is_alive())
                    .count();
                let next_cell = match (live_neighbour_count, self[Position(x, y)]) {
                    (3, _) => Cell::Alive,
                    (2, Cell::Alive) => Cell::Alive,
                    _ => Cell::Dead,
                };
                next_cells.push(next_cell);
            }
        }
        Self {
            width: self.width(),
            height: self.height(),
            cells: next_cells,
        }
    }

    pub fn neighbouring_cells(&self, relative_to: Position) -> Vec<Cell> {
        let offsets = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        let offset_to_cell_mapper = |(x_off, y_off): &(isize, isize)| {
            let mut x = relative_to.0 as isize + *x_off;
            let mut y = relative_to.1 as isize + *y_off;
            if x < 0 {
                x += self.width() as isize;
            } else {
                x %= self.width() as isize;
            }
            if y < 0 {
                y += self.height() as isize;
            } else {
                y %= self.height() as isize;
            }
            self[Position(x as usize, y as usize)]
        };
        offsets.iter().map(offset_to_cell_mapper).collect()
    }

    fn idx(&self, position: Position) -> usize {
        // TODO: why can't I destructure here or in the args list?
        let (x, y) = (position.0, position.1);
        debug_assert!(x < self.width());
        debug_assert!(y < self.height());
        x + y * self.width()
    }
}

impl ops::Index<Position> for Generation {
    type Output = Cell;

    fn index(&self, index: Position) -> &Self::Output {
        let idx = self.idx(index);
        &self.cells[idx]
    }
}

impl ops::IndexMut<Position> for Generation {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        let idx = self.idx(index);
        &mut self.cells[idx]
    }
}
