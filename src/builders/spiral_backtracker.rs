use std::time::{SystemTime, UNIX_EPOCH};

use super::BuildStrategy;
use crate::types::{Dir, Maze, Vec2i};

fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

fn turn_left(d: Dir) -> Dir {
    match d {
        Dir::N => Dir::W,
        Dir::W => Dir::S,
        Dir::S => Dir::E,
        Dir::E => Dir::N,
    }
}

fn turn_right(d: Dir) -> Dir {
    match d {
        Dir::N => Dir::E,
        Dir::E => Dir::S,
        Dir::S => Dir::W,
        Dir::W => Dir::N,
    }
}

fn reverse(d: Dir) -> Dir {
    match d {
        Dir::N => Dir::S,
        Dir::S => Dir::N,
        Dir::E => Dir::W,
        Dir::W => Dir::E,
    }
}

/// DFS that strongly prefers to continue in the same direction, then turn,
/// producing long curved corridors that wrap around each other in spiral patterns.
pub(crate) struct SpiralBacktracker {
    stack: Vec<(Vec2i, Dir)>,
    visited: Vec<bool>,
    seed: u32,
    done: bool,
}

impl SpiralBacktracker {
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
        // Pick a random initial direction
        let initial_dirs = [Dir::N, Dir::E, Dir::S, Dir::W];
        let mut s = seed;
        xorshift(&mut s);
        let initial_dir = initial_dirs[(s as usize) % 4];
        Self {
            stack: vec![(start, initial_dir)],
            visited,
            seed: s,
            done: false,
        }
    }
}

impl BuildStrategy for SpiralBacktracker {
    fn name(&self) -> &str {
        "Spiral Backtracker"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        loop {
            let Some(&(cell, facing)) = self.stack.last() else {
                self.done = true;
                return Vec::new();
            };

            // 50% straight, 25% prefer-left-turn, 25% prefer-right-turn.
            let r = xorshift(&mut self.seed) % 4;
            let dirs = match r {
                0 | 1 => [
                    facing,
                    turn_left(facing),
                    turn_right(facing),
                    reverse(facing),
                ],
                2 => [
                    turn_left(facing),
                    facing,
                    turn_right(facing),
                    reverse(facing),
                ],
                _ => [
                    turn_right(facing),
                    facing,
                    turn_left(facing),
                    reverse(facing),
                ],
            };

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
                self.stack.push((neighbor, dir));
                return vec![cell, neighbor];
            } else {
                self.stack.pop();
                // backtrack emits no visual change — keep looping
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SpiralBacktracker;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn spiral_backtracker_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = SpiralBacktracker::new_with_seed(&maze, 0x5721_5721);
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
