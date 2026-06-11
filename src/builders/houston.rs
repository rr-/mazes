use super::BuildStrategy;
use crate::rng::{shuffle_dirs, time_seed, xorshift};
use crate::types::{ALL_DIRS, Maze, Vec2i};

/// Hybrid: Aldous-Broder until ~50% of cells are visited, then Hunt-and-Kill.
/// AB gives an unbiased start; H&K closes the remainder quickly.
/// Result: faster convergence than pure AB with less directional bias than pure H&K.
pub(crate) struct HoustonGen {
    visited: Vec<bool>,
    remaining: usize,
    threshold: usize,
    ab_current: Vec2i,
    hk_current: Option<Vec2i>,
    hunt_idx: usize,
    seed: u32,
    phase_two: bool,
    done: bool,
}

impl HoustonGen {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let start = Vec2i { x: 0, y: 0 };
        let mut visited = vec![false; maze.w * maze.h];
        visited[maze.idx(start)] = true;
        let total = maze.w * maze.h;
        Self {
            visited,
            remaining: total - 1,
            threshold: total / 2,
            ab_current: start,
            hk_current: None,
            hunt_idx: 0,
            seed,
            phase_two: false,
            done: false,
        }
    }
}

impl BuildStrategy for HoustonGen {
    fn name(&self) -> &str {
        "Houston's"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        if !self.phase_two {
            let (dir, neighbor) = loop {
                let d = ALL_DIRS[(xorshift(&mut self.seed) as usize) % 4];
                let n = maze.neighbor(self.ab_current, d);
                if maze.in_bounds(n) {
                    break (d, n);
                }
            };

            let prev = self.ab_current;
            self.ab_current = neighbor;

            if !self.visited[maze.idx(neighbor)] {
                self.visited[maze.idx(neighbor)] = true;
                maze.carve(prev, dir);
                self.remaining -= 1;
                if self.remaining <= self.threshold {
                    self.phase_two = true;
                    self.hk_current = Some(neighbor);
                    self.hunt_idx = 0;
                }
                return vec![prev, neighbor];
            }
            return Vec::new();
        }

        if let Some(current) = self.hk_current {
            for dir in shuffle_dirs(&mut self.seed) {
                let next = maze.neighbor(current, dir);
                if maze.in_bounds(next) && !self.visited[maze.idx(next)] {
                    self.visited[maze.idx(next)] = true;
                    maze.carve(current, dir);
                    self.remaining -= 1;
                    self.hk_current = Some(next);
                    self.hunt_idx = 0;
                    if self.remaining == 0 {
                        self.done = true;
                    }
                    return vec![current, next];
                }
            }
            self.hk_current = None;
            return Vec::new();
        }

        while self.hunt_idx < self.visited.len() {
            let idx = self.hunt_idx;
            self.hunt_idx += 1;
            if self.visited[idx] {
                continue;
            }
            let cell = Vec2i {
                x: (idx % maze.w) as i32,
                y: (idx / maze.w) as i32,
            };
            for dir in ALL_DIRS {
                let neighbor = maze.neighbor(cell, dir);
                if maze.in_bounds(neighbor) && self.visited[maze.idx(neighbor)] {
                    self.visited[idx] = true;
                    maze.carve(cell, dir);
                    self.remaining -= 1;
                    self.hk_current = Some(cell);
                    self.hunt_idx = 0;
                    if self.remaining == 0 {
                        self.done = true;
                    }
                    return vec![cell, neighbor];
                }
            }
        }

        self.done = true;
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::HoustonGen;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn houston_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = HoustonGen::new_with_seed(&maze, 0xC0DE_CAFE);
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
