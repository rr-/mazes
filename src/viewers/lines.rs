use std::io::{self, Write};

use super::{MazeView, TerminalViewport};
use crate::types::{CellMark, Dir, Maze, MazeOverlay, Vec2i};

fn junction_char(rows: usize, cols: usize, x: usize, y: usize, buf: &[Vec<char>]) -> char {
    let n = y > 0 && buf[y - 1][x] != ' ';
    let s = y + 1 < rows && buf[y + 1][x] != ' ';
    let w = x > 0 && buf[y][x - 1] != ' ';
    let e = x + 1 < cols && buf[y][x + 1] != ' ';

    match (n, e, s, w) {
        (true, true, true, true) => '╋',
        (true, true, true, false) => '┣',
        (true, true, false, true) => '┻',
        (false, true, true, true) => '┳',
        (true, false, true, true) => '┫',
        (true, false, true, false) => '┃',
        (false, true, false, true) => '━',
        (true, true, false, false) => '┗',
        (false, true, true, false) => '┏',
        (false, false, true, true) => '┓',
        (true, false, false, true) => '┛',
        _ => ' ',
    }
}

pub(crate) fn render_lines(maze: &Maze) -> Vec<Vec<char>> {
    let rows = maze.h * 2 + 1;
    let cols = maze.w * 2 + 1;
    let mut buf = vec![vec![' '; cols]; rows];

    for y in 0..maze.h {
        for x in 0..maze.w {
            let c = &maze.grid[y * maze.w + x];
            let gy = y * 2 + 1;
            let gx = x * 2 + 1;

            if c.wall[Dir::N as usize] {
                buf[gy - 1][gx] = '━';
            }
            if c.wall[Dir::S as usize] {
                buf[gy + 1][gx] = '━';
            }
            if c.wall[Dir::W as usize] {
                buf[gy][gx - 1] = '┃';
            }
            if c.wall[Dir::E as usize] {
                buf[gy][gx + 1] = '┃';
            }
        }
    }

    for y in (0..rows).step_by(2) {
        for x in (0..cols).step_by(2) {
            buf[y][x] = junction_char(rows, cols, x, y, &buf);
        }
    }

    buf
}

#[cfg(test)]
pub(crate) fn render_lines_to_string(maze: &Maze) -> String {
    render_lines(maze)
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) struct LineMazeViewer {
    viewport: TerminalViewport,
}

impl LineMazeViewer {
    pub(crate) fn new(row: usize, col: usize) -> Self {
        Self {
            viewport: TerminalViewport::new(row, col),
        }
    }
}

fn overlay_color(mark: CellMark) -> &'static str {
    match mark {
        CellMark::None => "\x1b[0m",
        CellMark::Active => "\x1b[1;93m",
        CellMark::Dead => "\x1b[1;91m",
        CellMark::Solution => "\x1b[1;92m",
    }
}

fn has_overlay_link(maze: &Maze, overlay: &MazeOverlay, cell: Vec2i, dir: Dir) -> bool {
    let next = maze.neighbor(cell, dir);
    if !maze.in_bounds(next) {
        return false;
    }
    if matches!(overlay.get(maze.w, cell), CellMark::None)
        || matches!(overlay.get(maze.w, next), CellMark::None)
    {
        return false;
    }
    overlay.parent(maze.w, cell) == Some(next) || overlay.parent(maze.w, next) == Some(cell)
}

fn overlay_center_char(maze: &Maze, overlay: &MazeOverlay, cell: Vec2i) -> char {
    let n = has_overlay_link(maze, overlay, cell, Dir::N);
    let e = has_overlay_link(maze, overlay, cell, Dir::E);
    let s = has_overlay_link(maze, overlay, cell, Dir::S);
    let w = has_overlay_link(maze, overlay, cell, Dir::W);

    match (n, e, s, w) {
        (true, true, true, true) => '╋',
        (true, true, true, false) => '┣',
        (true, true, false, true) => '┻',
        (false, true, true, true) => '┳',
        (true, false, true, true) => '┫',
        (true, false, true, false) => '┃',
        (false, true, false, true) => '━',
        (true, true, false, false) => '┗',
        (false, true, true, false) => '┏',
        (false, false, true, true) => '┓',
        (true, false, false, true) => '┛',
        (true, false, false, false) => '┃',
        (false, true, false, false) => '━',
        (false, false, true, false) => '┃',
        (false, false, false, true) => '━',
        _ => ' ',
    }
}

fn draw_overlay_cell(
    stdout: &mut impl Write,
    viewport: &TerminalViewport,
    maze: &Maze,
    overlay: &MazeOverlay,
    cell: Vec2i,
) -> io::Result<()> {
    let gy = (cell.y as usize) * 2 + 1;
    let gx = (cell.x as usize) * 2 + 1;
    let mark = overlay.get(maze.w, cell);
    let color = overlay_color(mark);
    let glyph = overlay_center_char(maze, overlay, cell);

    for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
        if !has_overlay_link(maze, overlay, cell, dir) {
            continue;
        }
        let (wall_y, wall_x, wall_glyph) = match dir {
            Dir::N => (gy - 1, gx, '┃'),
            Dir::E => (gy, gx + 1, '━'),
            Dir::S => (gy + 1, gx, '┃'),
            Dir::W => (gy, gx - 1, '━'),
        };
        write!(
            stdout,
            "\x1b[{};{}H{}{}\x1b[0m",
            viewport.row + wall_y,
            viewport.col + wall_x,
            color,
            wall_glyph
        )?;
    }

    write!(
        stdout,
        "\x1b[{};{}H{}{}\x1b[0m",
        viewport.row + gy,
        viewport.col + gx,
        color,
        glyph
    )?;

    Ok(())
}

impl MazeView for LineMazeViewer {
    fn clear_screen(&self) -> io::Result<()> {
        self.viewport.clear_screen()
    }

    fn print(&self, maze: &Maze) -> io::Result<()> {
        let buf = render_lines(maze);
        self.viewport.print_buffer(&buf)
    }

    fn update_maze(&self, maze: &Maze, cells: Vec<Vec2i>) -> io::Result<()> {
        let buf = render_lines(maze);
        self.viewport.update_buffer(&buf, maze, cells)
    }

    fn update_overlay(
        &self,
        maze: &Maze,
        overlay: &MazeOverlay,
        cells: Vec<Vec2i>,
    ) -> io::Result<()> {
        if cells.is_empty() {
            return Ok(());
        }

        let mut stdout = io::stdout().lock();
        for cell in cells {
            let mark = overlay.get(maze.w, cell);
            if !matches!(mark, CellMark::None) {
                draw_overlay_cell(&mut stdout, &self.viewport, maze, overlay, cell)?;
                for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
                    let next = maze.neighbor(cell, dir);
                    if !maze.in_bounds(next) || matches!(overlay.get(maze.w, next), CellMark::None)
                    {
                        continue;
                    }
                    draw_overlay_cell(&mut stdout, &self.viewport, maze, overlay, next)?;
                }
                continue;
            }

            let color = overlay_color(mark);
            let gy = (cell.y as usize) * 2 + 1;
            let gx = (cell.x as usize) * 2 + 1;
            if let Some(parent) = overlay.parent(maze.w, cell) {
                let pgy = (parent.y as usize) * 2 + 1;
                let pgx = (parent.x as usize) * 2 + 1;
                let wall_y = (gy + pgy) / 2;
                let wall_x = (gx + pgx) / 2;
                write!(
                    stdout,
                    "\x1b[{};{}H{}{}\x1b[0m",
                    self.viewport.row + wall_y,
                    self.viewport.col + wall_x,
                    color,
                    ' '
                )?;
            }
            write!(
                stdout,
                "\x1b[{};{}H{}{}\x1b[0m",
                self.viewport.row + gy,
                self.viewport.col + gx,
                color,
                ' '
            )?;
        }

        write!(stdout, "\x1b[{};1H", self.viewport.row + maze.h * 2 + 1)?;
        stdout.flush()
    }

    fn print_footer(&self, maze: &Maze, left: &str, right: &str) -> io::Result<()> {
        self.viewport.print_footer(maze.h * 2 + 1, left, right)
    }
}
