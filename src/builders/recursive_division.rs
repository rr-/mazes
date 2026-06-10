use super::BuildStrategy;
use crate::rng::{time_seed, xorshift};
use crate::types::{Dir, Maze, Vec2i};

struct Chamber {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

/// Wall-adding algorithm. Starts with a fully open grid and recursively
/// subdivides chambers with walls, leaving one passage per wall.
/// Produces mazes with large rectangular rooms and long straight corridors.
pub(crate) struct RecursiveDivision {
    stack: Vec<Chamber>,
    initialized: bool,
    seed: u32,
    w: usize,
    h: usize,
    done: bool,
}

impl RecursiveDivision {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        Self {
            stack: vec![Chamber {
                x: 0,
                y: 0,
                w: maze.w,
                h: maze.h,
            }],
            initialized: false,
            seed,
            w: maze.w,
            h: maze.h,
            done: false,
        }
    }
}

impl BuildStrategy for RecursiveDivision {
    fn name(&self) -> &str {
        "Recursive Division"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        // First step: open all interior passages
        if !self.initialized {
            self.initialized = true;
            let mut updated = Vec::new();
            for y in 0..self.h {
                for x in 0..self.w {
                    let cell = Vec2i {
                        x: x as i32,
                        y: y as i32,
                    };
                    if x + 1 < self.w {
                        maze.carve(cell, Dir::E);
                        updated.push(cell);
                    }
                    if y + 1 < self.h {
                        maze.carve(cell, Dir::S);
                        updated.push(cell);
                    }
                }
            }
            return updated;
        }

        // Process chambers until we produce visible wall changes
        loop {
            let Some(chamber) = self.stack.pop() else {
                self.done = true;
                return Vec::new();
            };

            if chamber.w < 2 && chamber.h < 2 {
                continue;
            }

            // Prefer to divide along the longer axis; flip a coin when square
            let divide_h = if chamber.w < 2 {
                true
            } else if chamber.h < 2 {
                false
            } else {
                chamber.h > chamber.w
                    || (chamber.h == chamber.w && xorshift(&mut self.seed) & 1 == 0)
            };

            if divide_h {
                // Horizontal wall between row (split-1) and row split
                let split = 1 + (xorshift(&mut self.seed) as usize) % (chamber.h - 1);
                let passage = chamber.x + (xorshift(&mut self.seed) as usize) % chamber.w;
                let mut updated = Vec::new();
                for x in chamber.x..chamber.x + chamber.w {
                    if x != passage {
                        let above = Vec2i {
                            x: x as i32,
                            y: (chamber.y + split - 1) as i32,
                        };
                        maze.build(above, Dir::S);
                        updated.push(above);
                        updated.push(Vec2i {
                            x: x as i32,
                            y: (chamber.y + split) as i32,
                        });
                    }
                }
                self.stack.push(Chamber {
                    x: chamber.x,
                    y: chamber.y,
                    w: chamber.w,
                    h: split,
                });
                self.stack.push(Chamber {
                    x: chamber.x,
                    y: chamber.y + split,
                    w: chamber.w,
                    h: chamber.h - split,
                });
                if !updated.is_empty() {
                    return updated;
                }
            } else {
                // Vertical wall between col (split-1) and col split
                let split = 1 + (xorshift(&mut self.seed) as usize) % (chamber.w - 1);
                let passage = chamber.y + (xorshift(&mut self.seed) as usize) % chamber.h;
                let mut updated = Vec::new();
                for y in chamber.y..chamber.y + chamber.h {
                    if y != passage {
                        let left = Vec2i {
                            x: (chamber.x + split - 1) as i32,
                            y: y as i32,
                        };
                        maze.build(left, Dir::E);
                        updated.push(left);
                        updated.push(Vec2i {
                            x: (chamber.x + split) as i32,
                            y: y as i32,
                        });
                    }
                }
                self.stack.push(Chamber {
                    x: chamber.x,
                    y: chamber.y,
                    w: split,
                    h: chamber.h,
                });
                self.stack.push(Chamber {
                    x: chamber.x + split,
                    y: chamber.y,
                    w: chamber.w - split,
                    h: chamber.h,
                });
                if !updated.is_empty() {
                    return updated;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecursiveDivision;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn recursive_division_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = RecursiveDivision::new_with_seed(&maze, 0xC0DE_CAFE);
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
