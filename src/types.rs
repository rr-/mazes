#[derive(Copy, Clone)]
pub(crate) enum RenderStyle {
    Lines,
    Blocks,
    HalfBlocks,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Dir {
    N,
    E,
    S,
    W,
}

pub(crate) const ALL_DIRS: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Vec2i {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) pos: Vec2i,
    pub(crate) dir: Dir,
}

#[derive(Clone)]
pub(crate) struct Cell {
    pub(crate) wall: [bool; 4],
}

pub(crate) struct Maze {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) grid: Vec<Cell>,
}

#[derive(Copy, Clone)]
pub(crate) enum CellMark {
    None,
    Active,
    Dead,
    Solution,
}

pub(crate) struct MazeOverlay {
    pub(crate) marks: Vec<CellMark>,
    parents: Vec<Option<Vec2i>>,
}

impl MazeOverlay {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Self {
            marks: vec![CellMark::None; w * h],
            parents: vec![None; w * h],
        }
    }

    fn idx(&self, w: usize, p: Vec2i) -> usize {
        (p.y as usize) * w + (p.x as usize)
    }

    pub(crate) fn get(&self, w: usize, p: Vec2i) -> CellMark {
        self.marks[self.idx(w, p)]
    }

    pub(crate) fn set(&mut self, w: usize, p: Vec2i, mark: CellMark) {
        let idx = self.idx(w, p);
        self.marks[idx] = mark;
    }

    pub(crate) fn parent(&self, w: usize, p: Vec2i) -> Option<Vec2i> {
        self.parents[self.idx(w, p)]
    }

    pub(crate) fn set_parent(&mut self, w: usize, p: Vec2i, parent: Vec2i) {
        let idx = self.idx(w, p);
        self.parents[idx] = Some(parent);
    }

    pub(crate) fn clear(&mut self) {
        for mark in &mut self.marks {
            *mark = CellMark::None;
        }
        for parent in &mut self.parents {
            *parent = None;
        }
    }
}

impl Maze {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            grid: vec![Cell { wall: [true; 4] }; w * h],
        }
    }

    pub(crate) fn idx(&self, p: Vec2i) -> usize {
        (p.y as usize) * self.w + (p.x as usize)
    }

    pub(crate) fn in_bounds(&self, p: Vec2i) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.w && (p.y as usize) < self.h
    }

    pub(crate) fn opposite(d: Dir) -> Dir {
        match d {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::W => Dir::E,
            Dir::E => Dir::W,
        }
    }

    pub(crate) fn neighbor(&self, p: Vec2i, d: Dir) -> Vec2i {
        match d {
            Dir::N => Vec2i { x: p.x, y: p.y - 1 },
            Dir::S => Vec2i { x: p.x, y: p.y + 1 },
            Dir::W => Vec2i { x: p.x - 1, y: p.y },
            Dir::E => Vec2i { x: p.x + 1, y: p.y },
        }
    }

    fn set_wall(&mut self, a: Vec2i, d: Dir, wall: bool) {
        let i = self.idx(a);
        self.grid[i].wall[d as usize] = wall;
        let b = self.neighbor(a, d);
        if self.in_bounds(b) {
            let j = self.idx(b);
            self.grid[j].wall[Maze::opposite(d) as usize] = wall;
        }
    }

    pub(crate) fn carve(&mut self, a: Vec2i, d: Dir) {
        self.set_wall(a, d, false);
    }

    pub(crate) fn build(&mut self, a: Vec2i, d: Dir) {
        self.set_wall(a, d, true);
    }

    pub(crate) fn has_wall(&self, p: Vec2i, d: Dir) -> bool {
        self.grid[self.idx(p)].wall[d as usize]
    }

    pub(crate) fn start(&self) -> Vec2i {
        Vec2i { x: 0, y: 0 }
    }

    pub(crate) fn goal(&self) -> Vec2i {
        Vec2i {
            x: self.w as i32 - 1,
            y: self.h as i32 - 1,
        }
    }
}
