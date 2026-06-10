mod builders;
mod config;
mod solvers;
mod types;
mod viewers;

use std::{
    cell::RefCell,
    io::{self, Read},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use builders::build_builder;
use config::Config;
use solvers::build_solver;
use types::{Maze, MazeOverlay};
use viewers::{build_viewer, maze_size_from_terminal};

struct RawModeGuard;

thread_local! {
    static STTY_STATE: RefCell<Option<String>> = const { RefCell::new(None) };
}

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        let saved = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .output()?;
        if !saved.status.success() {
            return Err(io::Error::other("failed to capture terminal state"));
        }

        let saved = String::from_utf8(saved.stdout)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let saved = saved.trim().to_owned();

        let status = Command::new("stty")
            .args(["raw", "-echo", "min", "0", "time", "0"])
            .status()?;
        if !status.success() {
            return Err(io::Error::other("failed to enable raw terminal mode"));
        }

        STTY_STATE.with(|state| {
            *state.borrow_mut() = Some(saved);
        });
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        STTY_STATE.with(|state| {
            if let Some(saved) = state.borrow_mut().take() {
                let _ = Command::new("stty").arg(saved).status();
            }
        });
    }
}

fn should_quit() -> io::Result<bool> {
    let mut stdin = io::stdin();
    let mut buf = [0_u8; 16];
    loop {
        match stdin.read(&mut buf) {
            Ok(0) => return Ok(false),
            Ok(n) => {
                if buf[..n]
                    .iter()
                    .any(|byte| matches!(byte, 4 | 27 | b'q' | b'Q'))
                {
                    return Ok(true);
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(err) => return Err(err),
        }
    }
}

fn pause_with_quit(duration: Duration) -> io::Result<bool> {
    let tick = Duration::from_millis(10);
    let mut remaining = duration;
    while remaining > Duration::ZERO {
        if should_quit()? {
            return Ok(true);
        }
        let step = remaining.min(tick);
        thread::sleep(step);
        remaining = remaining.saturating_sub(step);
    }
    should_quit()
}

fn main() -> io::Result<()> {
    let config = Config::from_args()?;
    let _raw_mode = RawModeGuard::new()?;
    let viewer = build_viewer(config.style);

    loop {
        if should_quit()? {
            break;
        }

        let (maze_w, maze_h) = maze_size_from_terminal(config.style);
        let mut maze = Maze::new(maze_w, maze_h);
        let mut overlay = MazeOverlay::new(maze_w, maze_h);
        let mut builder = build_builder(&maze);

        viewer.clear_screen()?;
        viewer.print(&maze)?;
        while !builder.done() {
            if should_quit()? {
                return Ok(());
            }

            let mut updated = Vec::new();
            for _ in 0..config.build_steps {
                if builder.done() {
                    break;
                }
                updated.extend(builder.step(&mut maze));
            }
            viewer.update_maze(&maze, updated)?;
            if pause_with_quit(config.build_sleep)? {
                return Ok(());
            }
        }

        let mut solver = build_solver(&maze);
        overlay.clear();
        while !solver.done() {
            if should_quit()? {
                return Ok(());
            }

            let mut updated = Vec::new();
            for _ in 0..config.solve_steps {
                if solver.done() {
                    break;
                }
                updated.extend(solver.step(&maze, &mut overlay));
            }
            viewer.update_overlay(&maze, &overlay, updated)?;
            if pause_with_quit(config.solve_sleep)? {
                return Ok(());
            }
        }

        if pause_with_quit(config.wait)? {
            break;
        }
    }

    Ok(())
}
