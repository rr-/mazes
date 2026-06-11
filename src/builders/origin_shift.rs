use super::BuildStrategy;
use crate::rng::{time_seed, xorshift};
use crate::types::{ALL_DIRS, Dir, Maze, Vec2i};

/// Starts from a snake-path spanning tree and repeatedly walks the origin,
/// redirecting the neighbor's edge to point at the origin each step.
/// Each step produces a valid perfect maze.
/// Stops once every unvisited cell is fully surrounded by visited cells —
/// isolated unvisited cells are acceptable, clusters are not.
pub(crate) struct OriginShift {
    origin: Vec2i,
    /// Direction from each cell toward its parent (None = origin/root).
    parent: Vec<Option<Dir>>,
    init_edges: Vec<(Vec2i, Dir)>,
    init_idx: usize,
    origin_visited: Vec<bool>,
    /// How many of each cell's in-bounds neighbors are still unvisited.
    unvisited_neighbor_count: Vec<usize>,
    /// Count of unvisited cells that still have ≥1 unvisited neighbor.
    unvisited_non_isolated: usize,
    batch_size: usize,
    seed: u32,
    done: bool,
}

impl OriginShift {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let mut parent = vec![None; maze.w * maze.h];
        let mut init_edges = Vec::with_capacity(maze.w * maze.h - 1);

        // Snake-path spanning tree.
        // Even rows (y%2==0) go left→right; odd rows go right→left.
        for y in 0..maze.h {
            for x in 0..maze.w {
                if x == 0 && y == 0 {
                    continue;
                }
                let p = Vec2i {
                    x: x as i32,
                    y: y as i32,
                };
                let dir = if y % 2 == 0 && x == 0 {
                    Dir::N
                } else if y % 2 == 1 && x == maze.w - 1 {
                    Dir::N
                } else if y % 2 == 0 {
                    Dir::W
                } else {
                    Dir::E
                };
                parent[maze.idx(p)] = Some(dir);
                init_edges.push((p, dir));
            }
        }

        let total_cells = maze.w * maze.h;
        let start = Vec2i { x: 0, y: 0 };

        // For each cell, count how many in-bounds neighbors are unvisited.
        // Initially all cells are unvisited, so count = in-bounds degree.
        let mut unvisited_neighbor_count: Vec<usize> = (0..total_cells)
            .map(|i| {
                let p = Vec2i {
                    x: (i % maze.w) as i32,
                    y: (i / maze.w) as i32,
                };
                ALL_DIRS
                    .iter()
                    .filter(|&&d| maze.in_bounds(maze.neighbor(p, d)))
                    .count()
            })
            .collect();

        // Mark start as visited: decrement its neighbors' counts.
        let mut origin_visited = vec![false; total_cells];
        origin_visited[maze.idx(start)] = true;
        for d in ALL_DIRS {
            let n = maze.neighbor(start, d);
            if maze.in_bounds(n) {
                unvisited_neighbor_count[maze.idx(n)] -= 1;
            }
        }

        // Unvisited cells that still have ≥1 unvisited neighbor (non-isolated).
        let unvisited_non_isolated = (0..total_cells)
            .filter(|&i| !origin_visited[i] && unvisited_neighbor_count[i] > 0)
            .count();

        Self {
            origin: start,
            parent,
            init_edges,
            init_idx: 0,
            origin_visited,
            unvisited_neighbor_count,
            unvisited_non_isolated,
            batch_size: (((maze.w * maze.h) as f64).sqrt() * 0.25) as usize,
            seed,
            done: false,
        }
    }
}

impl BuildStrategy for OriginShift {
    fn name(&self) -> &str {
        "Origin Shift"
    }
    fn done(&self) -> bool {
        self.done
    }
    fn step_hint(&self) -> usize {
        self.batch_size
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        // Phase 1: carve the initial snake-path spanning tree one edge at a time.
        if self.init_idx < self.init_edges.len() {
            let (cell, dir) = self.init_edges[self.init_idx];
            self.init_idx += 1;
            maze.carve(cell, dir);
            return vec![cell, maze.neighbor(cell, dir)];
        }

        // Phase 2: one origin shift.
        let (dir_to_neighbor, neighbor) = loop {
            let d = ALL_DIRS[(xorshift(&mut self.seed) as usize) % 4];
            let n = maze.neighbor(self.origin, d);
            if maze.in_bounds(n) {
                break (d, n);
            }
        };
        let dir_to_origin = Maze::opposite(dir_to_neighbor);

        let mut updated = Vec::new();
        if self.parent[maze.idx(neighbor)] != Some(dir_to_origin) {
            if let Some(old_dir) = self.parent[maze.idx(neighbor)] {
                let old_parent = maze.neighbor(neighbor, old_dir);
                maze.build(neighbor, old_dir);
                updated.push(old_parent);
            }
            maze.carve(neighbor, dir_to_origin);
            updated.push(self.origin);
            updated.push(neighbor);
        }

        self.parent[maze.idx(self.origin)] = Some(dir_to_neighbor);
        self.parent[maze.idx(neighbor)] = None;
        self.origin = neighbor;

        let n_idx = maze.idx(neighbor);
        if !self.origin_visited[n_idx] {
            // Remove neighbor from non-isolated count if it was contributing.
            if self.unvisited_neighbor_count[n_idx] > 0 {
                self.unvisited_non_isolated -= 1;
            }
            self.origin_visited[n_idx] = true;

            // Decrement unvisited_neighbor_count for each of neighbor's neighbors.
            for d in ALL_DIRS {
                let nn = maze.neighbor(neighbor, d);
                if !maze.in_bounds(nn) {
                    continue;
                }
                let nn_idx = maze.idx(nn);
                if !self.origin_visited[nn_idx] {
                    self.unvisited_neighbor_count[nn_idx] -= 1;
                    if self.unvisited_neighbor_count[nn_idx] == 0 {
                        self.unvisited_non_isolated -= 1;
                    }
                }
            }

            if self.unvisited_non_isolated == 0 {
                self.done = true;
            }
        }

        updated
    }
}

#[cfg(test)]
mod tests {
    use super::OriginShift;
    use crate::{builders::BuildStrategy, types::Maze};

    fn count_passages(maze: &Maze) -> usize {
        maze.grid
            .iter()
            .map(|c| c.wall.iter().filter(|&&w| !w).count())
            .sum::<usize>()
            / 2
    }

    fn run_to_done(maze: &mut Maze, seed: u32) {
        let mut builder = OriginShift::new_with_seed(maze, seed);
        while !builder.done() {
            builder.step(maze);
        }
    }

    #[test]
    fn origin_shift_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        run_to_done(&mut maze, 0xFEED_FACE);
        assert_eq!(count_passages(&maze), 15);
    }

    #[test]
    fn origin_shift_1x8_produces_perfect_maze() {
        let mut maze = Maze::new(1, 8);
        run_to_done(&mut maze, 0x1234_ABCD);
        assert_eq!(count_passages(&maze), 7);
    }
}
