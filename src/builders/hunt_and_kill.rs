use super::BuildStrategy;
use crate::rng::{shuffle_dirs, time_seed};
use crate::types::{ALL_DIRS, Maze, Vec2i};

/// Random walk that "hunts" for a new entry point when stuck.
/// Produces long winding corridors like Recursive Backtracker but with a
/// different texture — the hunt scan introduces a subtle top-left bias.
pub(crate) struct HuntAndKill {
    current: Option<Vec2i>,
    visited: Vec<bool>,
    hunt_idx: usize,
    seed: u32,
    w: usize,
    done: bool,
}

impl HuntAndKill {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let start = Vec2i { x: 0, y: 0 };
        let mut visited = vec![false; maze.w * maze.h];
        visited[maze.idx(start)] = true;
        Self {
            current: Some(start),
            visited,
            hunt_idx: 0,
            seed,
            w: maze.w,
            done: false,
        }
    }
}

impl BuildStrategy for HuntAndKill {
    fn name(&self) -> &str {
        "Hunt and Kill"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        if let Some(current) = self.current {
            // Kill phase: random-walk to an unvisited neighbor
            for dir in shuffle_dirs(&mut self.seed) {
                let next = maze.neighbor(current, dir);
                if maze.in_bounds(next) && !self.visited[maze.idx(next)] {
                    self.visited[maze.idx(next)] = true;
                    maze.carve(current, dir);
                    self.current = Some(next);
                    self.hunt_idx = 0;
                    return vec![current, next];
                }
            }
            // Stuck — enter hunt mode
            self.current = None;
            return Vec::new();
        }

        // Hunt phase: scan row-by-row for an unvisited cell next to a visited one
        while self.hunt_idx < self.visited.len() {
            let idx = self.hunt_idx;
            self.hunt_idx += 1;
            if self.visited[idx] {
                continue;
            }
            let cell = Vec2i {
                x: (idx % self.w) as i32,
                y: (idx / self.w) as i32,
            };
            for dir in ALL_DIRS {
                let neighbor = maze.neighbor(cell, dir);
                if maze.in_bounds(neighbor) && self.visited[maze.idx(neighbor)] {
                    self.visited[idx] = true;
                    maze.carve(cell, dir);
                    self.current = Some(cell);
                    self.hunt_idx = 0;
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
    use super::HuntAndKill;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn hunt_and_kill_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = HuntAndKill::new_with_seed(&maze, 0xDEAD_BEEF);
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
