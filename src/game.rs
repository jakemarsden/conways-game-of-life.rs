use std::{iter, ops};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position(pub usize, pub usize);

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum State {
    Dead,
    Alive,
}

pub struct Generation {
    width: usize,
    height: usize,
    state: Vec<State>,
}

impl State {
    pub fn is_alive(self) -> bool {
        match self {
            Self::Dead => false,
            Self::Alive => true,
        }
    }
}

impl ops::Not for State {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Dead => Self::Alive,
            Self::Alive => Self::Dead,
        }
    }
}

impl Generation {
    pub fn generation_iter(seed: Generation) -> impl Iterator<Item = Self> {
        iter::successors(Some(seed), |prev_gen| Some(prev_gen.next()))
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            state: vec![State::Dead; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn next(&self) -> Generation {
        let mut next_states = Vec::with_capacity(self.state.len());
        for y in 0..self.height() {
            for x in 0..self.width() {
                let live_neighbour_count = self.live_neighbour_count_of(Position(x, y));
                let next_state = match (live_neighbour_count, self[Position(x, y)]) {
                    (3, _) => State::Alive,
                    (2, State::Alive) => State::Alive,
                    _ => State::Dead,
                };
                next_states.push(next_state);
            }
        }
        Self {
            width: self.width(),
            height: self.height(),
            state: next_states,
        }
    }

    pub fn live_neighbour_count_of(&self, position: Position) -> usize {
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
        let offset_to_state_mapper = |(x_off, y_off): &(isize, isize)| {
            let mut x = position.0 as isize + *x_off;
            let mut y = position.1 as isize + *y_off;
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
        offsets
            .iter()
            .map(offset_to_state_mapper)
            .filter(|state| state.is_alive())
            .count()
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
    type Output = State;

    fn index(&self, index: Position) -> &Self::Output {
        let idx = self.idx(index);
        &self.state[idx]
    }
}

impl ops::IndexMut<Position> for Generation {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        let idx = self.idx(index);
        &mut self.state[idx]
    }
}
