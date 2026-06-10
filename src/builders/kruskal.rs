use super::BuildStrategy;
use crate::rng::{time_seed, xorshift};
use crate::types::{Dir, Edge, Maze, Vec2i};

fn shuffle_with_seed<T>(arr: &mut [T], mut seed: u32) {
    for i in (1..arr.len()).rev() {
        let j = (xorshift(&mut seed) as usize) % (i + 1);
        arr.swap(i, j);
    }
}

fn shuffle<T>(arr: &mut [T]) {
    shuffle_with_seed(arr, time_seed());
}

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[rb] = ra;
        }
    }
}

pub(crate) struct KruskalGen {
    w: usize,
    edges: Vec<Edge>,
    dsu: Dsu,
    idx: usize,
}

impl KruskalGen {
    fn build_edges(w: usize, h: usize) -> Vec<Edge> {
        let mut edges = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let pos = Vec2i {
                    x: x as i32,
                    y: y as i32,
                };
                if x + 1 < w {
                    edges.push(Edge { pos, dir: Dir::E });
                }
                if y + 1 < h {
                    edges.push(Edge { pos, dir: Dir::S });
                }
            }
        }
        edges
    }

    pub(crate) fn new(maze: &Maze) -> Self {
        let (w, h) = (maze.w, maze.h);
        let mut edges = Self::build_edges(w, h);
        shuffle(&mut edges);
        Self {
            w,
            dsu: Dsu::new(w * h),
            edges,
            idx: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_seed(maze: &Maze, seed: u32) -> Self {
        let (w, h) = (maze.w, maze.h);
        let mut edges = Self::build_edges(w, h);
        shuffle_with_seed(&mut edges, seed);
        Self {
            w,
            dsu: Dsu::new(w * h),
            edges,
            idx: 0,
        }
    }
}

impl BuildStrategy for KruskalGen {
    fn name(&self) -> &str {
        "Kruskal's"
    }
    fn done(&self) -> bool {
        self.idx >= self.edges.len()
    }

    fn step(&mut self, maze: &mut Maze) -> Vec<Vec2i> {
        if self.done() {
            return Vec::new();
        }
        let e = self.edges[self.idx];
        self.idx += 1;

        let cell = |x: i32, y: i32| (y as usize) * self.w + (x as usize);
        let a = e.pos;
        let b = maze.neighbor(a, e.dir);

        let a_idx = cell(a.x, a.y);
        let b_idx = cell(b.x, b.y);
        if self.dsu.find(a_idx) != self.dsu.find(b_idx) {
            self.dsu.union(a_idx, b_idx);
            maze.carve(a, e.dir);
            return vec![a, b];
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{builders::BuildStrategy, types::Maze, viewers::lines::render_lines_to_string};

    use super::KruskalGen;

    const TEST_SEED: u32 = 0x1234_5678;

    #[test]
    fn kruskal_4x4_seeded_first_carves_match_snapshots() {
        let mut maze = Maze::new(4, 4);
        let mut generator = KruskalGen::new_with_seed(&maze, TEST_SEED);
        let expected = [
            (
                vec![(1, 1), (1, 2)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┣━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━╋━┫\n┃ ┃ ┃ ┃ ┃\n┗━┻━┻━┻━┛",
            ),
            (
                vec![(3, 2), (3, 3)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┣━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━┫ ┃\n┃ ┃ ┃ ┃ ┃\n┗━┻━┻━┻━┛",
            ),
            (
                vec![(1, 2), (2, 2)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┗━╋━┫\n┃ ┃   ┃ ┃\n┣━╋━┳━┫ ┃\n┃ ┃ ┃ ┃ ┃\n┗━┻━┻━┻━┛",
            ),
            (
                vec![(3, 1), (3, 2)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━╋━╋━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┗━┫ ┃\n┃ ┃   ┃ ┃\n┣━╋━┳━┫ ┃\n┃ ┃ ┃ ┃ ┃\n┗━┻━┻━┻━┛",
            ),
            (
                vec![(1, 0), (1, 1)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┣━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┗━┫ ┃\n┃ ┃   ┃ ┃\n┣━╋━┳━┫ ┃\n┃ ┃ ┃ ┃ ┃\n┗━┻━┻━┻━┛",
            ),
            (
                vec![(1, 3), (2, 3)],
                "┏━┳━┳━┳━┓\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┣━╋━┫\n┃ ┃ ┃ ┃ ┃\n┣━┫ ┗━┫ ┃\n┃ ┃   ┃ ┃\n┣━╋━━━┫ ┃\n┃ ┃   ┃ ┃\n┗━┻━━━┻━┛",
            ),
        ];

        for (expected_cells, expected_render) in expected {
            let updated = loop {
                let updated = generator.step(&mut maze);
                if !updated.is_empty() {
                    break updated;
                }
            };

            let updated_cells = updated.into_iter().map(|p| (p.x, p.y)).collect::<Vec<_>>();
            assert_eq!(updated_cells, expected_cells);
            assert_eq!(render_lines_to_string(&maze), expected_render);
        }
    }
}
