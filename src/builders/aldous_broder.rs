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

pub(crate) struct AldousBroder {
    current: Vec2i,
    visited: Vec<bool>,
    remaining: usize,
    seed: u32,
    done: bool,
}

impl AldousBroder {
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
            current: start,
            visited,
            remaining: maze.w * maze.h - 1,
            seed,
            done: false,
        }
    }
}

impl BuildStrategy for AldousBroder {
    fn name(&self) -> &str {
        "Aldous-Broder"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        // Pick a random valid neighbor
        let dir = loop {
            let dir = ALL_DIRS[(xorshift(&mut self.seed) as usize) % 4];
            let neighbor = maze.neighbor(self.current, dir);
            if maze.in_bounds(neighbor) {
                break (dir, neighbor);
            }
        };
        let (dir, neighbor) = dir;

        if !self.visited[maze.idx(neighbor)] {
            self.visited[maze.idx(neighbor)] = true;
            maze.carve(self.current, dir);
            self.remaining -= 1;
            if self.remaining == 0 {
                self.done = true;
            }
            let prev = self.current;
            self.current = neighbor;
            return vec![prev, neighbor];
        }

        self.current = neighbor;
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{builders::BuildStrategy, types::Maze};

    use super::AldousBroder;

    #[test]
    fn aldous_broder_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = AldousBroder::new_with_seed(&maze, 0xABCD_1234);
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
