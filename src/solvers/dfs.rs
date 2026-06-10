use super::{ALL_DIRS, SolveStrategy, advance_reveal};
use crate::types::{CellMark, Maze, MazeOverlay, Vec2i};

struct DfsFrame {
    cell: Vec2i,
    next_dir_idx: usize,
}

pub(crate) struct DfsSolver {
    start: Vec2i,
    goal: Vec2i,
    stack: Vec<DfsFrame>,
    discovered: Vec<bool>,
    parent: Vec<Option<Vec2i>>,
    reveal_cursor: Option<Vec2i>,
    finished: bool,
}

impl DfsSolver {
    pub(crate) fn new(maze: &Maze) -> Self {
        let start = maze.start();
        let goal = maze.goal();
        let mut discovered = vec![false; maze.w * maze.h];
        discovered[maze.idx(start)] = true;
        Self {
            start,
            goal,
            stack: vec![DfsFrame {
                cell: start,
                next_dir_idx: 0,
            }],
            discovered,
            parent: vec![None; maze.w * maze.h],
            reveal_cursor: None,
            finished: false,
        }
    }
}

impl SolveStrategy for DfsSolver {
    fn name(&self) -> &str {
        "DFS"
    }
    fn done(&self) -> bool {
        self.finished
    }

    fn step(&mut self, maze: &Maze, overlay: &mut MazeOverlay) -> Vec<Vec2i> {
        if self.finished {
            return Vec::new();
        }

        if let Some(cursor) = self.reveal_cursor {
            let (next, done) = advance_reveal(cursor, self.start, &self.parent, maze, overlay);
            self.reveal_cursor = next;
            self.finished = done;
            return vec![cursor];
        }

        if self.stack.is_empty() {
            self.finished = true;
            return Vec::new();
        }

        let cell = self.stack.last().unwrap().cell;
        if matches!(overlay.get(maze.w, cell), CellMark::None) {
            overlay.set(maze.w, cell, CellMark::Active);
            return vec![cell];
        }

        if cell == self.goal {
            self.reveal_cursor = Some(cell);
            return vec![cell];
        }

        while let Some(frame) = self.stack.last_mut() {
            if frame.next_dir_idx >= ALL_DIRS.len() {
                let cell = frame.cell;
                self.stack.pop();
                if cell == self.start {
                    self.finished = true;
                } else {
                    overlay.set(maze.w, cell, CellMark::Dead);
                }
                return vec![cell];
            }

            let dir = ALL_DIRS[frame.next_dir_idx];
            frame.next_dir_idx += 1;
            if maze.has_wall(cell, dir) {
                continue;
            }

            let next = maze.neighbor(cell, dir);
            if !maze.in_bounds(next) {
                continue;
            }

            let next_idx = maze.idx(next);
            if self.discovered[next_idx] {
                continue;
            }

            self.discovered[next_idx] = true;
            self.parent[next_idx] = Some(cell);
            overlay.set_parent(maze.w, next, cell);
            self.stack.push(DfsFrame {
                cell: next,
                next_dir_idx: 0,
            });
            overlay.set(maze.w, next, CellMark::Active);
            return vec![next];
        }

        Vec::new()
    }
}
