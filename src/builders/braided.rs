use super::{BuildStrategy, recursive_backtracker::RecursiveBacktracker};
use crate::rng::{time_seed, xorshift};
use crate::types::{ALL_DIRS, Dir, Maze, Vec2i};

fn open_count(maze: &Maze, cell: Vec2i) -> usize {
    ALL_DIRS
        .iter()
        .filter(|&&d| !maze.has_wall(cell, d))
        .count()
}

fn shuffle<T>(arr: &mut [T], seed: u32) {
    let mut s = seed;
    for i in (1..arr.len()).rev() {
        let j = (xorshift(&mut s) as usize) % (i + 1);
        arr.swap(i, j);
    }
}

/// Generates a perfect maze (via Recursive Backtracker), then eliminates dead ends
/// by carving extra passages at each one, creating a maze with no dead ends and many loops.
/// Prefer connecting dead ends to other dead ends for a more uniform braid.
pub(crate) struct BraidedMaze {
    inner: RecursiveBacktracker,
    dead_ends: Vec<Vec2i>,
    braiding: bool,
    seed: u32,
    done: bool,
}

impl BraidedMaze {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        Self {
            inner: RecursiveBacktracker::new_with_seed(maze, seed),
            dead_ends: Vec::new(),
            braiding: false,
            seed,
            done: false,
        }
    }
}

impl BuildStrategy for BraidedMaze {
    fn name(&self) -> &str {
        "Braided"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        if !self.braiding {
            let updated = self.inner.step(maze);
            if self.inner.done() {
                // Collect all dead ends (cells with exactly 1 open passage)
                self.dead_ends = (0..maze.h)
                    .flat_map(|y| {
                        (0..maze.w).map(move |x| Vec2i {
                            x: x as i32,
                            y: y as i32,
                        })
                    })
                    .filter(|&cell| open_count(maze, cell) == 1)
                    .collect();
                shuffle(&mut self.dead_ends, self.seed);
                self.braiding = true;
            }
            return updated;
        }

        // Braiding phase: process one dead end per step
        while let Some(cell) = self.dead_ends.pop() {
            if open_count(maze, cell) != 1 {
                continue; // already connected by a previous braid step
            }

            // Prefer carving to another dead end; fall back to any walled in-bounds neighbor
            let walled_neighbors: Vec<Dir> = ALL_DIRS
                .iter()
                .filter(|&&d| {
                    let n = maze.neighbor(cell, d);
                    maze.in_bounds(n) && maze.has_wall(cell, d)
                })
                .copied()
                .collect();

            if walled_neighbors.is_empty() {
                continue;
            }

            let dir = walled_neighbors
                .iter()
                .find(|&&d| open_count(maze, maze.neighbor(cell, d)) == 1)
                .copied()
                .unwrap_or(
                    walled_neighbors[(xorshift(&mut self.seed) as usize) % walled_neighbors.len()],
                );

            let neighbor = maze.neighbor(cell, dir);
            maze.carve(cell, dir);
            return vec![cell, neighbor];
        }

        self.done = true;
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::BraidedMaze;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn braided_has_no_dead_ends() {
        let mut maze = Maze::new(6, 6);
        let mut builder = BraidedMaze::new_with_seed(&maze, 0xB8A1_DEF0);
        while !builder.done() {
            builder.step(&mut maze);
        }
        let dead_ends = (0..maze.h)
            .flat_map(|y| (0..maze.w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let cell = crate::types::Vec2i {
                    x: x as i32,
                    y: y as i32,
                };
                [
                    crate::types::Dir::N,
                    crate::types::Dir::E,
                    crate::types::Dir::S,
                    crate::types::Dir::W,
                ]
                .iter()
                .filter(|&&d| !maze.has_wall(cell, d))
                .count()
                    == 1
            })
            .count();
        assert_eq!(dead_ends, 0, "braided maze should have no dead ends");
    }
}
