use crate::game::*;

pub trait Render {
    fn render(&mut self, gen: &Generation);
}

pub struct PrintlnRenderer {}

impl Render for PrintlnRenderer {
    fn render(&mut self, gen: &Generation) {
        println!("Generation: {}", gen.index());
        println!();
        for y in 0..gen.height() {
            print!("|");
            for x in 0..gen.width() {
                let position = Position::from((x, y));
                let ch = match gen[position] {
                    Cell::Alive => 'x',
                    Cell::Dead => ' ',
                };
                print!("{}", ch);
            }
            println!("|");
        }
    }
}
