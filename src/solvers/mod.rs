pub(crate) mod bfs;
pub(crate) mod dfs;
pub(crate) mod flood_fill;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Maze, MazeOverlay, Vec2i, normalize_name};

pub(crate) const ALL_DIRS: [crate::types::Dir; 4] = [
    crate::types::Dir::N,
    crate::types::Dir::E,
    crate::types::Dir::S,
    crate::types::Dir::W,
];

pub(crate) trait SolveStrategy {
    fn name(&self) -> &str;
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

fn build_flood_fill_solver(maze: &Maze) -> Box<dyn SolveStrategy> {
    Box::new(flood_fill::FloodFillSolver::new(maze))
}

const SOLVER_FACTORIES: [SolverFactory; 3] =
    [build_dfs_solver, build_bfs_solver, build_flood_fill_solver];

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

pub(crate) fn solver_names() -> Vec<String> {
    let dummy = Maze::new(5, 5);
    SOLVER_FACTORIES
        .iter()
        .map(|f| normalize_name(f(&dummy).name()))
        .collect()
}

pub(crate) fn find_solver_index(name: &str) -> Option<usize> {
    let needle = normalize_name(name);
    solver_names().into_iter().position(|n| n == needle)
}

pub(crate) fn build_solver_at(idx: usize, maze: &Maze) -> Box<dyn SolveStrategy> {
    SOLVER_FACTORIES[idx % SOLVER_FACTORIES.len()](maze)
}
