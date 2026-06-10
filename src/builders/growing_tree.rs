use std::time::{SystemTime, UNIX_EPOCH};

use super::BuildStrategy;
use crate::types::{Dir, Maze, Vec2i};

fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

const ALL_DIRS: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

/// Maintains a growing set of active cells and picks from it each step.
/// Picking newest → DFS (same as Recursive Backtracker).
/// Picking randomly → Prim's-like.
/// This implementation mixes: 75% newest, 25% random, producing an
/// intermediate texture with long corridors and occasional branches.
pub(crate) struct GrowingTree {
    cells: Vec<Vec2i>,
    visited: Vec<bool>,
    seed: u32,
    done: bool,
}

impl GrowingTree {
    pub(crate) fn new(maze: &Maze) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        Self::new_with_seed(maze, seed)
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let start = Vec2i { x: 0, y: 0 };
        let mut visited = vec![false; maze.w * maze.h];
        visited[maze.idx(start)] = true;
        Self {
            cells: vec![start],
            visited,
            seed,
            done: false,
        }
    }
}

impl BuildStrategy for GrowingTree {
    fn name(&self) -> &str {
        "Growing Tree"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        loop {
            if self.cells.is_empty() {
                self.done = true;
                return Vec::new();
            }

            let pick_idx = if xorshift(&mut self.seed) % 4 == 0 {
                (xorshift(&mut self.seed) as usize) % self.cells.len()
            } else {
                self.cells.len() - 1
            };

            let cell = self.cells[pick_idx];

            let mut dirs = ALL_DIRS;
            for i in (1..4).rev() {
                let j = (xorshift(&mut self.seed) as usize) % (i + 1);
                dirs.swap(i, j);
            }

            let carved = dirs.into_iter().find_map(|dir| {
                let neighbor = maze.neighbor(cell, dir);
                if maze.in_bounds(neighbor) && !self.visited[maze.idx(neighbor)] {
                    Some((dir, neighbor))
                } else {
                    None
                }
            });

            if let Some((dir, neighbor)) = carved {
                self.visited[maze.idx(neighbor)] = true;
                maze.carve(cell, dir);
                self.cells.push(neighbor);
                return vec![cell, neighbor];
            } else {
                self.cells.swap_remove(pick_idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GrowingTree;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn growing_tree_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = GrowingTree::new_with_seed(&maze, 0x6785_4321);
        while !builder.done() {
            builder.step(&mut maze);
        }
        let passages: usize = maze
            .grid
            .iter()
            .map(|c| c.wall.iter().filter(|&&w| !w).count())
            .sum::<usize>()
            / 2;
        assert_eq!(passages, 15);
    }
}
