use std::time::{SystemTime, UNIX_EPOCH};

use super::BuildStrategy;
use crate::types::{Dir, Maze, Vec2i};

fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

pub(crate) struct PrimGen {
    in_maze: Vec<bool>,
    frontier: Vec<(Vec2i, Dir)>,
    seed: u32,
    done: bool,
}

impl PrimGen {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, Self::time_seed())
    }

    fn time_seed() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let mut s = Self {
            in_maze: vec![false; maze.w * maze.h],
            frontier: Vec::new(),
            seed,
            done: false,
        };
        let start = Vec2i { x: 0, y: 0 };
        s.add_to_maze(maze, start);
        s
    }

    fn add_to_maze(&mut self, maze: &Maze, cell: Vec2i) {
        self.in_maze[maze.idx(cell)] = true;
        for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let neighbor = maze.neighbor(cell, dir);
            if maze.in_bounds(neighbor) && !self.in_maze[maze.idx(neighbor)] {
                self.frontier.push((cell, dir));
            }
        }
    }

    fn rand_frontier_idx(&mut self) -> usize {
        (xorshift(&mut self.seed) as usize) % self.frontier.len()
    }
}

impl BuildStrategy for PrimGen {
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        loop {
            if self.frontier.is_empty() {
                self.done = true;
                return Vec::new();
            }

            let idx = self.rand_frontier_idx();
            let (from, dir) = self.frontier.swap_remove(idx);
            let to = maze.neighbor(from, dir);

            if !maze.in_bounds(to) || self.in_maze[maze.idx(to)] {
                continue;
            }

            maze.carve(from, dir);
            self.add_to_maze(maze, to);
            return vec![from, to];
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{builders::BuildStrategy, types::Maze, viewers::lines::render_lines_to_string};

    use super::PrimGen;

    const TEST_SEED: u32 = 0xDEAD_BEEF;

    #[test]
    fn prim_4x4_seeded_produces_valid_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = PrimGen::new_with_seed(&maze, TEST_SEED);
        while !builder.done() {
            builder.step(&mut maze);
        }
        let rendered = render_lines_to_string(&maze);
        // Verify it's a connected maze (all cells reachable) by checking no isolated cells.
        // A perfect maze on 4x4 has exactly 15 carved passages.
        let passages: usize = maze
            .grid
            .iter()
            .map(|c| c.wall.iter().filter(|&&w| !w).count())
            .sum::<usize>()
            / 2;
        assert_eq!(
            passages, 15,
            "4x4 perfect maze must have exactly 15 passages\n{rendered}"
        );
    }
}
