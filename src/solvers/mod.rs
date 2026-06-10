pub(crate) mod bfs;
pub(crate) mod dfs;
pub(crate) mod flood_fill;

use crate::rng::time_seed;
use crate::types::{CellMark, Maze, MazeOverlay, Vec2i};
use crate::util::normalize_name;

pub(crate) use crate::types::ALL_DIRS;

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
    (time_seed() as usize) % len
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
    SOLVER_FACTORIES[idx](maze)
}

pub(crate) fn advance_reveal(
    cursor: Vec2i,
    start: Vec2i,
    parent: &[Option<Vec2i>],
    maze: &Maze,
    overlay: &mut MazeOverlay,
) -> (Option<Vec2i>, bool) {
    overlay.set(maze.w, cursor, CellMark::Solution);
    if cursor == start {
        (None, true)
    } else {
        (parent[maze.idx(cursor)], false)
    }
}
