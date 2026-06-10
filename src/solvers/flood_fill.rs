use std::collections::VecDeque;

use super::{ALL_DIRS, SolveStrategy, advance_reveal};
use crate::types::{CellMark, Maze, MazeOverlay, Vec2i};

pub(crate) struct FloodFillSolver {
    start: Vec2i,
    goal: Vec2i,
    queue: VecDeque<Vec2i>,
    visited: Vec<bool>,
    parent: Vec<Option<Vec2i>>,
    reveal_cursor: Option<Vec2i>,
    finished: bool,
}

impl FloodFillSolver {
    pub(crate) fn new(maze: &Maze) -> Self {
        let start = maze.start();
        let goal = maze.goal();
        let mut visited = vec![false; maze.w * maze.h];
        visited[maze.idx(start)] = true;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        Self {
            start,
            goal,
            queue,
            visited,
            parent: vec![None; maze.w * maze.h],
            reveal_cursor: None,
            finished: false,
        }
    }
}

impl SolveStrategy for FloodFillSolver {
    fn name(&self) -> &str {
        "Flood Fill"
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

        let Some(cell) = self.queue.pop_front() else {
            self.finished = true;
            return Vec::new();
        };

        if cell == self.goal {
            self.reveal_cursor = Some(cell);
            return vec![cell];
        }

        if matches!(overlay.get(maze.w, cell), CellMark::None) {
            overlay.set(maze.w, cell, CellMark::Active);
        }

        let mut updated = vec![cell];
        for dir in ALL_DIRS {
            if maze.has_wall(cell, dir) {
                continue;
            }
            let next = maze.neighbor(cell, dir);
            if !maze.in_bounds(next) {
                continue;
            }
            let next_idx = maze.idx(next);
            if self.visited[next_idx] {
                continue;
            }
            self.visited[next_idx] = true;
            self.parent[next_idx] = Some(cell);
            overlay.set_parent(maze.w, next, cell);
            overlay.set(maze.w, next, CellMark::Active);
            self.queue.push_back(next);
            updated.push(next);
        }

        updated
    }
}
