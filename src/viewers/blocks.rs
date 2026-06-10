use std::io::{self, Write};

use super::{MazeView, TerminalViewport, maze_update_points, overlay_update_points};
use crate::types::{CellMark, Dir, Maze, MazeOverlay, Vec2i};

#[derive(Copy, Clone)]
enum BlockPixel {
    Empty,
    Wall,
    Overlay(CellMark),
}

fn block_pixel_color(pixel: BlockPixel) -> u8 {
    match pixel {
        BlockPixel::Empty => 16,
        BlockPixel::Wall => 255,
        BlockPixel::Overlay(CellMark::None) => 16,
        BlockPixel::Overlay(CellMark::Active) => 142,
        BlockPixel::Overlay(CellMark::Dead) => 235,
        BlockPixel::Overlay(CellMark::Solution) => 154,
    }
}

fn render_blocks(maze: &Maze, overlay: Option<&MazeOverlay>) -> Vec<Vec<u8>> {
    let rows = maze.h * 2 + 1;
    let cols = maze.w * 2 + 1;
    let mut buf = vec![vec![block_pixel_color(BlockPixel::Wall); cols]; rows];

    for y in 0..maze.h {
        for x in 0..maze.w {
            let c = &maze.grid[y * maze.w + x];
            let gy = y * 2 + 1;
            let gx = x * 2 + 1;

            buf[gy][gx] = block_pixel_color(BlockPixel::Empty);
            if !c.wall[Dir::N as usize] {
                buf[gy - 1][gx] = block_pixel_color(BlockPixel::Empty);
            }
            if !c.wall[Dir::S as usize] {
                buf[gy + 1][gx] = block_pixel_color(BlockPixel::Empty);
            }
            if !c.wall[Dir::W as usize] {
                buf[gy][gx - 1] = block_pixel_color(BlockPixel::Empty);
            }
            if !c.wall[Dir::E as usize] {
                buf[gy][gx + 1] = block_pixel_color(BlockPixel::Empty);
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
                let color = block_pixel_color(BlockPixel::Overlay(mark));
                buf[gy][gx] = color;

                if let Some(parent) = overlay.parent(maze.w, cell) {
                    let pgy = (parent.y as usize) * 2 + 1;
                    let pgx = (parent.x as usize) * 2 + 1;
                    let wall_y = (gy + pgy) / 2;
                    let wall_x = (gx + pgx) / 2;
                    buf[wall_y][wall_x] = color;
                }
            }
        }
    }

    buf
}

fn print_color_buffer(viewport: &TerminalViewport, buf: &[Vec<u8>]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    for (y, row) in buf.iter().enumerate() {
        write!(stdout, "\x1b[{};{}H", viewport.row + y, viewport.col)?;
        for color in row {
            write!(stdout, "\x1b[38;5;{}m█", color)?;
        }
        write!(stdout, "\x1b[0m")?;
    }
    write!(stdout, "\x1b[{};1H", viewport.row + buf.len())?;
    stdout.flush()
}

fn update_color_buffer(
    viewport: &TerminalViewport,
    buf: &[Vec<u8>],
    points: impl IntoIterator<Item = (usize, usize)>,
) -> io::Result<()> {
    let rows = buf.len();
    let cols = buf.first().map_or(0, Vec::len);
    let mut stdout = io::stdout().lock();

    for (y, x) in points {
        if y >= rows || x >= cols {
            continue;
        }
        write!(
            stdout,
            "\x1b[{};{}H\x1b[38;5;{}m█\x1b[0m",
            viewport.row + y,
            viewport.col + x,
            buf[y][x]
        )?;
    }

    write!(stdout, "\x1b[{};1H", viewport.row + rows)?;
    stdout.flush()
}

pub(crate) struct BlockMazeViewer {
    viewport: TerminalViewport,
}

impl BlockMazeViewer {
    pub(crate) fn new(row: usize, col: usize) -> Self {
        Self {
            viewport: TerminalViewport::new(row, col),
        }
    }
}

impl MazeView for BlockMazeViewer {
    fn clear_screen(&self) -> io::Result<()> {
        self.viewport.clear_screen()
    }

    fn print(&self, maze: &Maze) -> io::Result<()> {
        let buf = render_blocks(maze, None);
        print_color_buffer(&self.viewport, &buf)
    }

    fn update_maze(&self, maze: &Maze, cells: Vec<Vec2i>) -> io::Result<()> {
        let buf = render_blocks(maze, None);
        update_color_buffer(&self.viewport, &buf, maze_update_points(cells))
    }

    fn update_overlay(
        &self,
        maze: &Maze,
        overlay: &MazeOverlay,
        cells: Vec<Vec2i>,
    ) -> io::Result<()> {
        let buf = render_blocks(maze, Some(overlay));
        update_color_buffer(
            &self.viewport,
            &buf,
            overlay_update_points(maze, overlay, cells),
        )
    }
}
