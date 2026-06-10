use std::time::{SystemTime, UNIX_EPOCH};

use super::BuildStrategy;
use crate::types::{Dir, Maze, Vec2i};

fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

/// Row-by-row algorithm. For each cell, either carve East (extending the current run)
/// or close the run by carving North from a random cell within it.
/// Produces a guaranteed open top row and a strong horizontal bias.
pub(crate) struct Sidewinder {
    x: usize,
    y: usize,
    run_start: usize,
    seed: u32,
    w: usize,
    h: usize,
    done: bool,
}

impl Sidewinder {
    pub(crate) fn new(maze: &Maze) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        Self::new_with_seed(maze, seed)
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            run_start: 0,
            seed,
            w: maze.w,
            h: maze.h,
            done: false,
        }
    }
}

impl BuildStrategy for Sidewinder {
    fn name(&self) -> &str {
        "Sidewinder"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        let cell = Vec2i {
            x: self.x as i32,
            y: self.y as i32,
        };
        let can_east = self.x + 1 < self.w;
        let can_north = self.y > 0;

        // Close the run if we must (east boundary) or randomly choose to (when north is possible)
        let close_run = !can_east || (can_north && xorshift(&mut self.seed) & 1 == 0);

        let mut updated = vec![cell];

        if close_run && can_north {
            let run_len = self.x - self.run_start + 1;
            let pick_x = self.run_start + (xorshift(&mut self.seed) as usize) % run_len;
            let pick = Vec2i {
                x: pick_x as i32,
                y: self.y as i32,
            };
            maze.carve(pick, Dir::N);
            updated.push(pick);
            updated.push(maze.neighbor(pick, Dir::N));
            self.run_start = self.x + 1;
        } else if can_east {
            maze.carve(cell, Dir::E);
            updated.push(maze.neighbor(cell, Dir::E));
        }

        self.x += 1;
        if self.x >= self.w {
            self.x = 0;
            self.run_start = 0;
            self.y += 1;
            if self.y >= self.h {
                self.done = true;
            }
        }

        updated
    }
}

#[cfg(test)]
mod tests {
    use super::Sidewinder;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn sidewinder_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = Sidewinder::new_with_seed(&maze, 0x5EED_5EED);
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
