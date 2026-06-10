pub(crate) mod bfs;
pub(crate) mod dfs;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Maze, MazeOverlay, Vec2i};

pub(crate) const ALL_DIRS: [crate::types::Dir; 4] = [
    crate::types::Dir::N,
    crate::types::Dir::E,
    crate::types::Dir::S,
    crate::types::Dir::W,
];

pub(crate) trait SolveStrategy {
    fn done(&self) -> bool;
    fn step(&mut self, maze: &Maze, overlay: &mut MazeOverlay) -> Vec<Vec2i>;
}

type SolverFactory = fn(&Maze) -> Box<dyn SolveStrategy>;

fn build_dfs_solver(maze: &Maze) -> Box<dyn SolveStrategy> {
    Box::new(dfs::DfsSolver::new(maze))
}

fn build_bfs_solver(maze: &Maze) -> Box<dyn SolveStrategy> {
    Box::new(bfs::BfsSolver::new(maze))
}

const SOLVER_FACTORIES: [SolverFactory; 2] = [build_dfs_solver, build_bfs_solver];

fn random_index(len: usize) -> usize {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (seed as usize) % len
}

pub(crate) fn build_solver(maze: &Maze) -> Box<dyn SolveStrategy> {
    let factory = SOLVER_FACTORIES[random_index(SOLVER_FACTORIES.len())];
    factory(maze)
}
