use std::time::{SystemTime, UNIX_EPOCH};

use super::BuildStrategy;
use crate::types::{Dir, Maze, Vec2i};

fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

/// For each cell, randomly carve either North or East (if available).
/// Produces a strong NE-diagonal bias but is extremely fast.
pub(crate) struct BinaryTree {
    x: usize,
    y: usize,
    seed: u32,
    w: usize,
    h: usize,
    done: bool,
}

impl BinaryTree {
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
            seed,
            w: maze.w,
            h: maze.h,
            done: false,
        }
    }
}

impl BuildStrategy for BinaryTree {
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

        let can_north = self.y > 0;
        let can_east = self.x + 1 < self.w;

        let carved = match (can_north, can_east) {
            (true, true) => {
                if xorshift(&mut self.seed) & 1 == 0 {
                    maze.carve(cell, Dir::N);
                    Some(maze.neighbor(cell, Dir::N))
                } else {
                    maze.carve(cell, Dir::E);
                    Some(maze.neighbor(cell, Dir::E))
                }
            }
            (true, false) => {
                maze.carve(cell, Dir::N);
                Some(maze.neighbor(cell, Dir::N))
            }
            (false, true) => {
                maze.carve(cell, Dir::E);
                Some(maze.neighbor(cell, Dir::E))
            }
            (false, false) => None,
        };

        // Advance to next cell in row-major order
        self.x += 1;
        if self.x >= self.w {
            self.x = 0;
            self.y += 1;
            if self.y >= self.h {
                self.done = true;
            }
        }

        if let Some(neighbor) = carved {
            vec![cell, neighbor]
        } else {
            vec![cell]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{builders::BuildStrategy, types::Maze};

    use super::BinaryTree;

    #[test]
    fn binary_tree_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = BinaryTree::new_with_seed(&maze, 0x1111_2222);
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
