pub(crate) mod kruskal;

use crate::types::{Maze, Vec2i};

pub(crate) trait BuildStrategy {
    fn done(&self) -> bool;
    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i>;
}

pub(crate) fn build_builder(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(kruskal::KruskalGen::new(maze))
}
