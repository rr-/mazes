mod builders;
mod config;
mod solvers;
mod types;
mod viewers;

use std::{
    cell::RefCell,
    io::{self, Read, Write},
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

        let mut stdout = io::stdout().lock();
        write!(stdout, "\x1b[?25l")?;
        stdout.flush()?;
        drop(stdout);

        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = write!(io::stdout(), "\x1b[?25h");
        STTY_STATE.with(|state| {
            if let Some(saved) = state.borrow_mut().take() {
                let _ = Command::new("stty").arg(saved).status();
            }
        });
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Input {
    None,
    SkipSecond,
    Advance,
    TogglePause,
    NewMaze,
    Quit,
}

fn read_input() -> io::Result<Input> {
    let mut stdin = io::stdin();
    let mut buf = [0_u8; 64];
    let n = match stdin.read(&mut buf) {
        Ok(0) | Err(_) => return Ok(Input::None),
        Ok(n) => n,
    };
    let bytes = &buf[..n];

    let mut result = Input::None;
    let mut i = 0;
    while i < bytes.len() {
        let input = if bytes[i] == 0x1b && i + 2 < bytes.len() && bytes[i + 1] == b'[' {
            let seq = match bytes[i + 2] {
                b'C' => Input::SkipSecond, // →
                _ => Input::None,
            };
            i += 3;
            seq
        } else {
            let key = match bytes[i] {
                4 | 27 | b'q' | b'Q' => Input::Quit,
                b'n' | b'N' => Input::NewMaze,
                b'p' | b'P' => Input::TogglePause,
                b'.' => Input::Advance,
                _ => Input::None,
            };
            i += 1;
            key
        };
        if input > result {
            result = input;
        }
    }
    Ok(result)
}

/// Sleeps for `duration` in 10ms ticks, returning the highest-priority input seen.
/// If `paused`, blocks indefinitely until any input arrives.
fn sleep_and_read(duration: Duration, paused: bool) -> io::Result<Input> {
    let tick = Duration::from_millis(10);
    let mut remaining = duration;
    loop {
        let input = read_input()?;
        if input != Input::None {
            return Ok(input);
        }
        if !paused && remaining == Duration::ZERO {
            return Ok(Input::None);
        }
        thread::sleep(tick);
        if !paused {
            remaining = remaining.saturating_sub(tick);
        }
    }
}

/// Runs enough steps to fill roughly one second of animation, then renders.
fn skip_one_second_build(
    config: &Config,
    builder: &mut dyn builders::BuildStrategy,
    maze: &mut Maze,
    viewer: &dyn viewers::MazeView,
) -> io::Result<()> {
    let n = (1.0 / config.build_sleep.as_secs_f64()).ceil() as usize * config.build_steps;
    let mut updated = Vec::new();
    for _ in 0..n {
        if builder.done() {
            break;
        }
        updated.extend(builder.step(maze));
    }
    if !updated.is_empty() {
        viewer.update_maze(maze, updated)?;
    }
    Ok(())
}

fn skip_one_second_solve(
    config: &Config,
    solver: &mut dyn solvers::SolveStrategy,
    maze: &Maze,
    overlay: &mut MazeOverlay,
    viewer: &dyn viewers::MazeView,
) -> io::Result<()> {
    let n = (1.0 / config.solve_sleep.as_secs_f64()).ceil() as usize * config.solve_steps;
    let mut updated = Vec::new();
    for _ in 0..n {
        if solver.done() {
            break;
        }
        updated.extend(solver.step(maze, overlay));
    }
    if !updated.is_empty() {
        viewer.update_overlay(maze, overlay, updated)?;
    }
    Ok(())
}

fn footer_left(phase: &str, strat: &str, paused: bool) -> String {
    let pause_tag = if paused { "  [PAUSED]" } else { "" };
    format!("{phase}: {strat}{pause_tag}")
}

fn footer_right(steps: usize, sleep: Duration) -> String {
    let fps = (1.0 / sleep.as_secs_f64()).round() as usize;
    format!("{steps} steps/tick  ·  {fps} fps")
}

fn main() -> io::Result<()> {
    let config = Config::from_args()?;
    let _raw_mode = RawModeGuard::new()?;
    let viewer = build_viewer(config.style);

    let mut paused = false;

    'outer: loop {
        let (maze_w, maze_h) = maze_size_from_terminal(config.style);
        let mut maze = Maze::new(maze_w, maze_h);
        let mut overlay = MazeOverlay::new(maze_w, maze_h);
        let mut builder = build_builder(&maze);

        viewer.clear_screen()?;
        viewer.print(&maze)?;
        viewer.print_footer(
            &maze,
            &footer_left("Building", builder.name(), paused),
            &footer_right(config.build_steps, config.build_sleep),
        )?;

        // Build phase
        while !builder.done() {
            if paused {
                match sleep_and_read(Duration::ZERO, true)? {
                    Input::Quit => break 'outer,
                    Input::NewMaze => continue 'outer,
                    Input::TogglePause => {
                        paused = false;
                        viewer.print_footer(
                            &maze,
                            &footer_left("Building", builder.name(), paused),
                            &footer_right(config.build_steps, config.build_sleep),
                        )?;
                    }
                    Input::SkipSecond => {
                        skip_one_second_build(
                            &config,
                            builder.as_mut(),
                            &mut maze,
                            viewer.as_ref(),
                        )?;
                        continue;
                    }
                    Input::Advance => {
                        let mut updated = Vec::new();
                        while updated.is_empty() && !builder.done() {
                            updated.extend(builder.step(&mut maze));
                        }
                        viewer.update_maze(&maze, updated)?;
                        continue;
                    }
                    Input::None => continue,
                }
            }

            let mut updated = Vec::new();
            while updated.is_empty() {
                for _ in 0..config.build_steps {
                    if builder.done() {
                        break;
                    }
                    updated.extend(builder.step(&mut maze));
                }
                if builder.done() {
                    break;
                }
            }
            viewer.update_maze(&maze, updated)?;

            if paused {
                continue;
            }

            match sleep_and_read(config.build_sleep, false)? {
                Input::Quit => break 'outer,
                Input::NewMaze => continue 'outer,
                Input::TogglePause => {
                    paused = true;
                    viewer.print_footer(
                        &maze,
                        &footer_left("Building", builder.name(), paused),
                        &footer_right(config.build_steps, config.build_sleep),
                    )?;
                }
                Input::SkipSecond => {
                    skip_one_second_build(&config, builder.as_mut(), &mut maze, viewer.as_ref())?
                }
                Input::Advance | Input::None => {}
            }
        }

        let mut solver = build_solver(&maze);
        overlay.clear();
        viewer.print_footer(
            &maze,
            &footer_left("Solving", solver.name(), paused),
            &footer_right(config.solve_steps, config.solve_sleep),
        )?;

        // Solve phase
        while !solver.done() {
            if paused {
                match sleep_and_read(Duration::ZERO, true)? {
                    Input::Quit => break 'outer,
                    Input::NewMaze => continue 'outer,
                    Input::TogglePause => {
                        paused = false;
                        viewer.print_footer(
                            &maze,
                            &footer_left("Solving", solver.name(), paused),
                            &footer_right(config.solve_steps, config.solve_sleep),
                        )?;
                    }
                    Input::SkipSecond => {
                        skip_one_second_solve(
                            &config,
                            solver.as_mut(),
                            &maze,
                            &mut overlay,
                            viewer.as_ref(),
                        )?;
                        continue;
                    }
                    Input::Advance => {
                        let mut updated = Vec::new();
                        while updated.is_empty() && !solver.done() {
                            updated.extend(solver.step(&maze, &mut overlay));
                        }
                        viewer.update_overlay(&maze, &overlay, updated)?;
                        continue;
                    }
                    Input::None => continue,
                }
            }

            let mut updated = Vec::new();
            while updated.is_empty() {
                for _ in 0..config.solve_steps {
                    if solver.done() {
                        break;
                    }
                    updated.extend(solver.step(&maze, &mut overlay));
                }
                if solver.done() {
                    break;
                }
            }
            viewer.update_overlay(&maze, &overlay, updated)?;

            if paused {
                continue;
            }

            match sleep_and_read(config.solve_sleep, false)? {
                Input::Quit => break 'outer,
                Input::NewMaze => continue 'outer,
                Input::TogglePause => {
                    paused = true;
                    viewer.print_footer(
                        &maze,
                        &footer_left("Solving", solver.name(), paused),
                        &footer_right(config.solve_steps, config.solve_sleep),
                    )?;
                }
                Input::SkipSecond => skip_one_second_solve(
                    &config,
                    solver.as_mut(),
                    &maze,
                    &mut overlay,
                    viewer.as_ref(),
                )?,
                Input::Advance | Input::None => {}
            }
        }

        viewer.print_footer(
            &maze,
            &format!("Complete  ·  {} → {}", builder.name(), solver.name()),
            "",
        )?;

        match sleep_and_read(config.wait, paused)? {
            Input::Quit => break 'outer,
            Input::NewMaze => continue 'outer,
            Input::TogglePause => paused = !paused,
            Input::SkipSecond | Input::Advance | Input::None => {}
        }
    }

    Ok(())
}
