use super::BuildStrategy;
use crate::rng::{time_seed, xorshift};
use crate::types::{ALL_DIRS, Dir, Maze, Vec2i};

fn dir_between(from: Vec2i, to: Vec2i) -> Dir {
    match (to.x - from.x, to.y - from.y) {
        (1, 0) => Dir::E,
        (-1, 0) => Dir::W,
        (0, 1) => Dir::S,
        _ => Dir::N,
    }
}

/// Loop-erased random walk. Produces a uniform spanning tree (same distribution as Aldous-Broder)
/// but is much faster on average: starts slow, accelerates as the maze fills.
pub(crate) struct WilsonGen {
    in_maze: Vec<bool>,
    walk: Vec<Vec2i>,
    walk_idx: Vec<Option<usize>>,
    unvisited: usize,
    seed: u32,
    w: usize,
    h: usize,
    done: bool,
}

impl WilsonGen {
    pub(crate) fn new(maze: &Maze) -> Self {
        Self::new_with_seed(maze, time_seed())
    }

    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let n = maze.w * maze.h;
        let mut in_maze = vec![false; n];
        in_maze[0] = true;
        Self {
            in_maze,
            walk: Vec::new(),
            walk_idx: vec![None; n],
            unvisited: n - 1,
            seed,
            w: maze.w,
            h: maze.h,
            done: n == 1,
        }
    }

    fn cidx(&self, p: Vec2i) -> usize {
        (p.y as usize) * self.w + (p.x as usize)
    }

    fn random_unvisited(&mut self) -> Vec2i {
        let target = (xorshift(&mut self.seed) as usize) % self.unvisited;
        (0..self.h)
            .flat_map(|y| (0..self.w).map(move |x| (x, y)))
            .filter(|&(x, y)| !self.in_maze[y * self.w + x])
            .nth(target)
            .map(|(x, y)| Vec2i {
                x: x as i32,
                y: y as i32,
            })
            .expect("unvisited counter desync")
    }
}

impl BuildStrategy for WilsonGen {
    fn name(&self) -> &str {
        "Wilson's"
    }
    fn done(&self) -> bool {
        self.done
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done {
            return Vec::new();
        }

        if self.walk.is_empty() {
            let start = self.random_unvisited();
            let start_idx = self.cidx(start);
            self.walk_idx[start_idx] = Some(0);
            self.walk.push(start);
            return Vec::new();
        }

        let current = *self.walk.last().unwrap();
        let next = loop {
            let dir = ALL_DIRS[(xorshift(&mut self.seed) as usize) % 4];
            let n = maze.neighbor(current, dir);
            if maze.in_bounds(n) {
                break n;
            }
        };
        let next_idx = self.cidx(next);

        if self.in_maze[next_idx] {
            // Commit the walk to the maze
            let walk = std::mem::take(&mut self.walk);
            let mut updated = Vec::new();
            for i in 0..walk.len() {
                let from = walk[i];
                let to = walk.get(i + 1).copied().unwrap_or(next);
                let from_idx = self.cidx(from);
                maze.carve(from, dir_between(from, to));
                self.in_maze[from_idx] = true;
                self.walk_idx[from_idx] = None;
                self.unvisited -= 1;
                updated.push(from);
                updated.push(to);
            }
            if self.unvisited == 0 {
                self.done = true;
            }
            return updated;
        }

        if let Some(loop_idx) = self.walk_idx[next_idx] {
            // Erase the loop back to where we re-entered the walk
            let tail: Vec<Vec2i> = self.walk.drain(loop_idx + 1..).collect();
            let w = self.w;
            for cell in tail {
                self.walk_idx[(cell.y as usize) * w + (cell.x as usize)] = None;
            }
            return Vec::new();
        }

        // Extend the walk
        self.walk_idx[next_idx] = Some(self.walk.len());
        self.walk.push(next);
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::WilsonGen;
    use crate::{builders::BuildStrategy, types::Maze};

    #[test]
    fn wilson_4x4_produces_perfect_maze() {
        let mut maze = Maze::new(4, 4);
        let mut builder = WilsonGen::new_with_seed(&maze, 0xFEED_FACE);
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
