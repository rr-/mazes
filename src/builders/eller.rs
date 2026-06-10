use std::collections::HashSet;

use super::BuildStrategy;
use crate::rng::{time_seed, xorshift};
use crate::types::{Dir, Maze, Vec2i};

#[derive(PartialEq)]
enum Phase {
    Horizontal,
    Vertical,
}

/// Row-by-row algorithm that never backtracks and uses O(width) memory.
/// Each row: randomly merge adjacent cells from different sets (carving East),
/// then carve at least one South passage per set before moving on.
/// Produces mazes with a distinctive horizontal banding texture.
pub(crate) struct EllerGen {
    row_sets: Vec<usize>,
    next_row: Vec<usize>,
    sets_with_south: HashSet<usize>,
    next_set: usize,
    y: usize,
    x: usize,
    phase: Phase,
    seed: u32,
    w: usize,
    h: usize,
    done: bool,
}

impl EllerGen {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let w = maze.w;
        Self {
            row_sets: (0..w).collect(),
            next_row: vec![0; w],
            sets_with_south: HashSet::new(),
            next_set: w,
            y: 0,
            x: 0,
            phase: Phase::Horizontal,
            seed,
            w,
            h: maze.h,
            done: maze.h == 0,
        }
    }

    fn merge_sets(&mut self, from: usize, into: usize) {
        for s in &mut self.row_sets {
            if *s == from {
                *s = into;
            }
        }
    }

    fn last_of_set_in_row(&self, x: usize) -> bool {
        let set = self.row_sets[x];
        !self.row_sets[x + 1..].iter().any(|&s| s == set)
    }
}

impl BuildStrategy for EllerGen {
    fn name(&self) -> &str {
        "Eller's"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        let last_row = self.y + 1 == self.h;

        if self.phase == Phase::Horizontal {
            // Process one horizontal merge decision at position x, x+1
            while self.x + 1 < self.w {
                let x = self.x;
                self.x += 1;
                let a = self.row_sets[x];
                let b = self.row_sets[x + 1];
                if a != b && (last_row || xorshift(&mut self.seed) & 1 == 0) {
                    self.merge_sets(b, a);
                    let cell = Vec2i {
                        x: x as i32,
                        y: self.y as i32,
                    };
                    maze.carve(cell, Dir::E);
                    return vec![cell, maze.neighbor(cell, Dir::E)];
                }
            }
            // Done with horizontal — start vertical
            self.x = 0;
            self.sets_with_south.clear();
            self.phase = Phase::Vertical;
        }

        if self.phase == Phase::Vertical && !last_row {
            while self.x < self.w {
                let x = self.x;
                self.x += 1;
                let set = self.row_sets[x];
                let is_last = self.last_of_set_in_row(x);
                let must_carve = is_last && !self.sets_with_south.contains(&set);
                if must_carve || xorshift(&mut self.seed) & 1 == 0 {
                    self.sets_with_south.insert(set);
                    self.next_row[x] = set;
                    let cell = Vec2i {
                        x: x as i32,
                        y: self.y as i32,
                    };
                    maze.carve(cell, Dir::S);
                    return vec![cell, maze.neighbor(cell, Dir::S)];
                } else {
                    self.next_row[x] = self.next_set;
                    self.next_set += 1;
                }
            }
            // Advance to next row
            std::mem::swap(&mut self.row_sets, &mut self.next_row);
            self.y += 1;
            self.x = 0;
            self.phase = Phase::Horizontal;
        }

        if last_row {
            self.done = true;
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::EllerGen;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn eller_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = EllerGen::new_with_seed(&maze, 0xE77E_E77E);
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
