use std::{
    collections::BTreeSet,
    io::{self, Write},
};

use super::{MazeView, TerminalViewport, maze_update_points, overlay_update_points};
use crate::types::{CellMark, Dir, Maze, MazeOverlay, Vec2i};

#[derive(Copy, Clone)]
enum HalfBlockPixel {
    Empty,
    Wall,
    Overlay(CellMark),
}

#[derive(Copy, Clone)]
struct HalfBlockCell {
    fg: u8,
    bg: u8,
}

fn half_block_pixel_color(pixel: HalfBlockPixel) -> u8 {
    match pixel {
        HalfBlockPixel::Empty => 16,
        HalfBlockPixel::Wall => 255,
        HalfBlockPixel::Overlay(CellMark::None) => 16,
        HalfBlockPixel::Overlay(CellMark::Active) => 142,
        HalfBlockPixel::Overlay(CellMark::Dead) => 235,
        HalfBlockPixel::Overlay(CellMark::Solution) => 154,
    }
}

fn render_half_blocks(maze: &Maze, overlay: Option<&MazeOverlay>) -> Vec<Vec<HalfBlockCell>> {
    let virtual_rows = maze.h * 2 + 1;
    let cols = maze.w * 2 + 1;
    let mut pixels = vec![vec![HalfBlockPixel::Wall; cols]; virtual_rows];

    for y in 0..maze.h {
        for x in 0..maze.w {
            let c = &maze.grid[y * maze.w + x];
            let gy = y * 2 + 1;
            let gx = x * 2 + 1;

            pixels[gy][gx] = HalfBlockPixel::Empty;
            if !c.wall[Dir::N as usize] {
                pixels[gy - 1][gx] = HalfBlockPixel::Empty;
            }
            if !c.wall[Dir::S as usize] {
                pixels[gy + 1][gx] = HalfBlockPixel::Empty;
            }
            if !c.wall[Dir::W as usize] {
                pixels[gy][gx - 1] = HalfBlockPixel::Empty;
            }
            if !c.wall[Dir::E as usize] {
                pixels[gy][gx + 1] = HalfBlockPixel::Empty;
            }
        }
    }

    if let Some(overlay) = overlay {
        for y in 0..maze.h {
            for x in 0..maze.w {
                let cell = Vec2i {
                    x: x as i32,
                    y: y as i32,
                };
                let mark = overlay.get(maze.w, cell);
                if matches!(mark, CellMark::None) {
                    continue;
                }

                let gy = y * 2 + 1;
                let gx = x * 2 + 1;
                pixels[gy][gx] = HalfBlockPixel::Overlay(mark);

                if let Some(parent) = overlay.parent(maze.w, cell) {
                    let pgy = (parent.y as usize) * 2 + 1;
                    let pgx = (parent.x as usize) * 2 + 1;
                    let wall_y = (gy + pgy) / 2;
                    let wall_x = (gx + pgx) / 2;
                    pixels[wall_y][wall_x] = HalfBlockPixel::Overlay(mark);
                }
            }
        }
    }

    let rows = maze.h + 1;
    let mut buf = vec![vec![HalfBlockCell { fg: 16, bg: 16 }; cols]; rows];
    for y in 0..rows {
        let top_y = y * 2;
        let bottom_y = top_y + 1;
        for x in 0..cols {
            let top = pixels[top_y][x];
            let bottom = if bottom_y < virtual_rows {
                pixels[bottom_y][x]
            } else {
                HalfBlockPixel::Empty
            };
            buf[y][x] = HalfBlockCell {
                fg: half_block_pixel_color(bottom),
                bg: half_block_pixel_color(top),
            };
        }
    }

    buf
}

fn compress_half_block_points(points: BTreeSet<(usize, usize)>) -> BTreeSet<(usize, usize)> {
    points.into_iter().map(|(y, x)| (y / 2, x)).collect()
}

fn print_half_block_buffer(
    viewport: &TerminalViewport,
    buf: &[Vec<HalfBlockCell>],
) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    for (y, row) in buf.iter().enumerate() {
        write!(stdout, "\x1b[{};{}H", viewport.row + y, viewport.col)?;
        for cell in row {
            write!(stdout, "\x1b[38;5;{};48;5;{}m▄", cell.fg, cell.bg)?;
        }
        write!(stdout, "\x1b[0m")?;
    }
    write!(stdout, "\x1b[{};1H", viewport.row + buf.len())?;
    stdout.flush()
}

fn update_half_block_buffer(
    viewport: &TerminalViewport,
    buf: &[Vec<HalfBlockCell>],
    points: BTreeSet<(usize, usize)>,
) -> io::Result<()> {
    if points.is_empty() {
        return Ok(());
    }

    let rows = buf.len();
    let cols = buf.first().map_or(0, Vec::len);
    let mut stdout = io::stdout().lock();

    for (y, x) in points {
        if y >= rows || x >= cols {
            continue;
        }
        let cell = buf[y][x];
        write!(
            stdout,
            "\x1b[{};{}H\x1b[38;5;{};48;5;{}m▄\x1b[0m",
            viewport.row + y,
            viewport.col + x,
            cell.fg,
            cell.bg
        )?;
    }

    write!(stdout, "\x1b[{};1H", viewport.row + rows)?;
    stdout.flush()
}

pub(crate) struct HalfBlockMazeViewer {
    viewport: TerminalViewport,
}

impl HalfBlockMazeViewer {
    pub(crate) fn new(row: usize, col: usize) -> Self {
        Self {
            viewport: TerminalViewport::new(row, col),
        }
    }
}

impl MazeView for HalfBlockMazeViewer {
    fn clear_screen(&self) -> io::Result<()> {
        self.viewport.clear_screen()
    }

    fn print(&self, maze: &Maze) -> io::Result<()> {
        let buf = render_half_blocks(maze, None);
        print_half_block_buffer(&self.viewport, &buf)
    }

    fn update_maze(&self, maze: &Maze, cells: Vec<Vec2i>) -> io::Result<()> {
        let buf = render_half_blocks(maze, None);
        let points = compress_half_block_points(maze_update_points(cells));
        update_half_block_buffer(&self.viewport, &buf, points)
    }

    fn update_overlay(
        &self,
        maze: &Maze,
        overlay: &MazeOverlay,
        cells: Vec<Vec2i>,
    ) -> io::Result<()> {
        let buf = render_half_blocks(maze, Some(overlay));
        let points = compress_half_block_points(overlay_update_points(maze, overlay, cells));
        update_half_block_buffer(&self.viewport, &buf, points)
    }
}
