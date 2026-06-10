pub(crate) mod blocks;
pub(crate) mod half_blocks;
pub(crate) mod lines;

use std::{
    collections::BTreeSet,
    io::{self, Write},
};

use terminal_size::{Height, Width, terminal_size};

use crate::types::{Maze, MazeOverlay, RenderStyle, Vec2i};

pub(crate) trait MazeView {
    fn clear_screen(&self) -> io::Result<()>;
    fn print(&self, maze: &Maze) -> io::Result<()>;
    fn update_maze(&self, maze: &Maze, cells: Vec<Vec2i>) -> io::Result<()>;
    fn update_overlay(
        &self,
        maze: &Maze,
        overlay: &MazeOverlay,
        cells: Vec<Vec2i>,
    ) -> io::Result<()>;
}

pub(crate) fn build_viewer(style: RenderStyle) -> Box<dyn MazeView> {
    match style {
        RenderStyle::Lines => Box::new(lines::LineMazeViewer::new(1, 1)),
        RenderStyle::Blocks => Box::new(blocks::BlockMazeViewer::new(1, 1)),
        RenderStyle::HalfBlocks => Box::new(half_blocks::HalfBlockMazeViewer::new(1, 1)),
    }
}

pub(crate) fn maze_size_from_terminal(style: RenderStyle) -> (usize, usize) {
    if let Some((Width(cols), Height(rows))) = terminal_size() {
        let maze_w = usize::from(cols.saturating_sub(1)).max(3) / 2;
        let maze_h = match style {
            RenderStyle::Lines | RenderStyle::Blocks => {
                usize::from(rows.saturating_sub(2)).max(3) / 2
            }
            RenderStyle::HalfBlocks => usize::from(rows.saturating_sub(2)),
        };
        return (maze_w.max(1), maze_h.max(1));
    }

    (10, 6)
}

pub(crate) fn maze_update_points(cells: Vec<Vec2i>) -> BTreeSet<(usize, usize)> {
    let mut points = BTreeSet::new();
    for cell in cells {
        let gy = (cell.y as usize) * 2 + 1;
        let gx = (cell.x as usize) * 2 + 1;

        points.insert((gy, gx));
        points.insert((gy - 1, gx));
        points.insert((gy + 1, gx));
        points.insert((gy, gx - 1));
        points.insert((gy, gx + 1));
        points.insert((gy - 1, gx - 1));
        points.insert((gy - 1, gx + 1));
        points.insert((gy + 1, gx - 1));
        points.insert((gy + 1, gx + 1));
    }
    points
}

pub(crate) fn overlay_update_points(
    maze: &Maze,
    overlay: &MazeOverlay,
    cells: Vec<Vec2i>,
) -> BTreeSet<(usize, usize)> {
    let mut points = BTreeSet::new();
    for cell in cells {
        let gy = (cell.y as usize) * 2 + 1;
        let gx = (cell.x as usize) * 2 + 1;
        points.insert((gy, gx));
        if let Some(parent) = overlay.parent(maze.w, cell) {
            let pgy = (parent.y as usize) * 2 + 1;
            let pgx = (parent.x as usize) * 2 + 1;
            points.insert(((gy + pgy) / 2, (gx + pgx) / 2));
        }
    }
    points
}

pub(crate) struct TerminalViewport {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

impl TerminalViewport {
    pub(crate) fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub(crate) fn clear_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout().lock();
        write!(stdout, "\x1b[2J\x1b[H")?;
        stdout.flush()
    }

    pub(crate) fn print_buffer(&self, buf: &[Vec<char>]) -> io::Result<()> {
        let mut stdout = io::stdout().lock();
        for (y, row) in buf.iter().enumerate() {
            write!(stdout, "\x1b[{};{}H", self.row + y, self.col)?;
            for ch in row {
                write!(stdout, "{ch}")?;
            }
        }
        write!(stdout, "\x1b[{};1H", self.row + buf.len())?;
        stdout.flush()
    }

    pub(crate) fn update_buffer(
        &self,
        buf: &[Vec<char>],
        maze: &Maze,
        cells: Vec<Vec2i>,
    ) -> io::Result<()> {
        if cells.is_empty() {
            return Ok(());
        }

        let points = maze_update_points(cells);
        let rows = maze.h * 2 + 1;
        let cols = maze.w * 2 + 1;
        let mut stdout = io::stdout().lock();

        for (y, x) in points {
            if y >= rows || x >= cols {
                continue;
            }
            write!(
                stdout,
                "\x1b[{};{}H{}",
                self.row + y,
                self.col + x,
                buf[y][x]
            )?;
        }

        write!(stdout, "\x1b[{};1H", self.row + rows)?;
        stdout.flush()
    }
}
