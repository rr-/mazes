pub(crate) mod aldous_broder;
pub(crate) mod binary_tree;
pub(crate) mod braided;
pub(crate) mod eller;
pub(crate) mod growing_tree;
pub(crate) mod houston;
pub(crate) mod hunt_and_kill;
pub(crate) mod kruskal;
pub(crate) mod origin_shift;
pub(crate) mod prim;
pub(crate) mod recursive_backtracker;
pub(crate) mod recursive_division;
pub(crate) mod sidewinder;
pub(crate) mod spiral_backtracker;
pub(crate) mod wilson;

use crate::rng::time_seed;
use crate::types::{Maze, Vec2i};
use crate::util::normalize_name;

pub(crate) trait BuildStrategy {
    fn name(&self) -> &str;
    fn done(&self) -> bool;
    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i>;
    /// Suggested number of `step()` calls per tick. Used by the runner in
    /// normal (non-paused) mode; pause single-step always calls once.
    fn step_hint(&self) -> usize {
        1
    }
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

fn build_hunt_and_kill(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(hunt_and_kill::HuntAndKill::new(maze))
}

fn build_sidewinder(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(sidewinder::Sidewinder::new(maze))
}

fn build_wilson(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(wilson::WilsonGen::new(maze))
}

fn build_growing_tree(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(growing_tree::GrowingTree::new(maze))
}

fn build_recursive_division(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(recursive_division::RecursiveDivision::new(maze))
}

fn build_eller(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(eller::EllerGen::new(maze))
}

fn build_spiral_backtracker(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(spiral_backtracker::SpiralBacktracker::new(maze))
}

fn build_braided(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(braided::BraidedMaze::new(maze))
}

fn build_houston(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(houston::HoustonGen::new(maze))
}

fn build_origin_shift(maze: &Maze) -> Box<dyn BuildStrategy> {
    Box::new(origin_shift::OriginShift::new(maze))
}

const BUILDER_FACTORIES: [BuilderFactory; 15] = [
    build_kruskal,
    build_prim,
    build_recursive_backtracker,
    build_aldous_broder,
    build_binary_tree,
    build_hunt_and_kill,
    build_sidewinder,
    build_wilson,
    build_growing_tree,
    build_recursive_division,
    build_eller,
    build_spiral_backtracker,
    build_braided,
    build_houston,
    build_origin_shift,
];

fn random_index(len: usize) -> usize {
    (time_seed() as usize) % len
}

pub(crate) fn builder_count() -> usize {
    BUILDER_FACTORIES.len()
}

pub(crate) fn build_builder_at(idx: usize, maze: &Maze) -> Box<dyn BuildStrategy> {
    BUILDER_FACTORIES[idx](maze)
}

pub(crate) fn build_builder(maze: &Maze) -> (usize, Box<dyn BuildStrategy>) {
    let idx = random_index(BUILDER_FACTORIES.len());
    (idx, BUILDER_FACTORIES[idx](maze))
}

pub(crate) fn builder_names() -> Vec<String> {
    let dummy = Maze::new(5, 5);
    BUILDER_FACTORIES
        .iter()
        .map(|f| normalize_name(f(&dummy).name()))
        .collect()
}

pub(crate) fn find_builder_index(name: &str) -> Option<usize> {
    let needle = normalize_name(name);
    builder_names().into_iter().position(|n| n == needle)
}
