use std::collections::VecDeque;

use super::{ALL_DIRS, SolveStrategy, advance_reveal};
use crate::types::{CellMark, Maze, MazeOverlay, Vec2i};

pub(crate) struct BfsSolver {
    start: Vec2i,
    goal: Vec2i,
    queue: VecDeque<Vec2i>,
    discovered: Vec<bool>,
    parent: Vec<Option<Vec2i>>,
    reveal_cursor: Option<Vec2i>,
    expanded: Vec<bool>,
    live_children: Vec<usize>,
    finished: bool,
}

impl BfsSolver {
    pub(crate) fn new(maze: &Maze) -> Self {
        let start = maze.start();
        let goal = maze.goal();
        let mut discovered = vec![false; maze.w * maze.h];
        discovered[maze.idx(start)] = true;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        Self {
            start,
            goal,
            queue,
            discovered,
            parent: vec![None; maze.w * maze.h],
            reveal_cursor: None,
            expanded: vec![false; maze.w * maze.h],
            live_children: vec![0; maze.w * maze.h],
            finished: false,
        }
    }

    fn collapse_dead_branch(
        &mut self,
        maze: &Maze,
        overlay: &mut MazeOverlay,
        start: Vec2i,
    ) -> Vec<Vec2i> {
        let mut updated = Vec::new();
        let mut cursor = Some(start);

        while let Some(cell) = cursor {
            if matches!(
                overlay.get(maze.w, cell),
                CellMark::Dead | CellMark::Solution
            ) {
                break;
            }

            overlay.set(maze.w, cell, CellMark::Dead);
            updated.push(cell);

            let Some(parent) = self.parent[maze.idx(cell)] else {
                break;
            };

            let parent_idx = maze.idx(parent);
            self.live_children[parent_idx] = self.live_children[parent_idx].saturating_sub(1);

            if parent == self.start {
                break;
            }
            if !self.expanded[parent_idx] || self.live_children[parent_idx] > 0 {
                break;
            }

            cursor = Some(parent);
        }

        updated
    }
}

impl SolveStrategy for BfsSolver {
    fn name(&self) -> &str {
        "BFS"
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

        if matches!(overlay.get(maze.w, cell), CellMark::None) {
            overlay.set(maze.w, cell, CellMark::Active);
        }

        if cell == self.goal {
            self.reveal_cursor = Some(cell);
            return vec![cell];
        }

        self.expanded[maze.idx(cell)] = true;
        let mut spawned_children = 0usize;
        for dir in ALL_DIRS {
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
            self.queue.push_back(next);
            spawned_children += 1;
        }

        self.live_children[maze.idx(cell)] = spawned_children;
        if spawned_children == 0 {
            return self.collapse_dead_branch(maze, overlay, cell);
        }

        vec![cell]
    }
}
