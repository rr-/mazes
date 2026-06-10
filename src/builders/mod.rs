pub(crate) mod aldous_broder;
pub(crate) mod binary_tree;
pub(crate) mod kruskal;
pub(crate) mod prim;
pub(crate) mod recursive_backtracker;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Maze, Vec2i};

pub(crate) trait BuildStrategy {
    fn done(&self) -> bool;
    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i>;
}

type BuilderFactory = fn(&Maze) -> Box<dyn BuildStrategy>;

fn build_kruskal(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(kruskal::KruskalGen::new(maze))
}

fn build_prim(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(prim::PrimGen::new(maze))
}

fn build_recursive_backtracker(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(recursive_backtracker::RecursiveBacktracker::new(maze))
}

fn build_aldous_broder(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(aldous_broder::AldousBroder::new(maze))
}

fn build_binary_tree(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(binary_tree::BinaryTree::new(maze))
}

const BUILDER_FACTORIES: [BuilderFactory; 5] = [
    build_kruskal,
    build_prim,
    build_recursive_backtracker,
    build_aldous_broder,
    build_binary_tree,
];

fn random_index(len: usize) -> usize {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (seed as usize) % len
}

pub(crate) fn build_builder(maze: &Maze) -> Box<dyn BuildStrategy> {
    let factory = BUILDER_FACTORIES[random_index(BUILDER_FACTORIES.len())];
    factory(maze)
}
