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

pub(crate) struct RecursiveBacktracker {
    stack: Vec<Vec2i>,
    visited: Vec<bool>,
    seed: u32,
    done: bool,
}

impl RecursiveBacktracker {
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
            stack: vec![start],
            visited,
            seed,
            done: false,
        }
    }
}

impl BuildStrategy for RecursiveBacktracker {
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        loop {
            let Some(&cell) = self.stack.last() else {
                self.done = true;
                return Vec::new();
            };

            let mut dirs = ALL_DIRS;
            // Fisher-Yates shuffle the 4 directions
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
                self.stack.push(neighbor);
                return vec![cell, neighbor];
            } else {
                self.stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{builders::BuildStrategy, types::Maze};

    use super::RecursiveBacktracker;

    #[test]
    fn recursive_backtracker_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = RecursiveBacktracker::new_with_seed(&maze, 0xCAFE_BABE);
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
