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

use builders::{build_builder, build_builder_at, builder_count};
use config::Config;
use solvers::{SolveStrategy, build_solver};
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
    CycleBuilder,
    CyclePrevBuilder,
    RandomBuilder,
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
                b'r' | b'R' => Input::RandomBuilder,
                b'b' => Input::CycleBuilder,
                b'B' => Input::CyclePrevBuilder,
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
    solver: &mut dyn SolveStrategy,
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

fn footer_left(phase: &str, strat: &str, random: bool, paused: bool) -> String {
    let random_tag = if random { " (random)" } else { "" };
    let pause_tag = if paused { "  [PAUSED]" } else { "" };
    format!("{phase}: {strat}{random_tag}{pause_tag}")
}

fn footer_right(steps: usize, sleep: Duration) -> String {
    let fps = (1.0 / sleep.as_secs_f64()).round() as usize;
    format!("{steps} steps/tick  ·  {fps} fps")
}

#[derive(PartialEq)]
enum Phase {
    Build,
    Solve,
    Complete,
}

fn main() -> io::Result<()> {
    let config = Config::from_args()?;
    let _raw_mode = RawModeGuard::new()?;
    let viewer = build_viewer(config.style);

    let mut paused = false;
    let mut random_mode = true;
    let mut builder_idx: Option<usize> = None;

    // --- helpers that set up a fresh maze ---
    let init_maze = |builder_idx: &mut Option<usize>,
                     _random_mode: bool|
     -> (Maze, MazeOverlay, Box<dyn builders::BuildStrategy>) {
        let (w, h) = maze_size_from_terminal(config.style);
        let maze = Maze::new(w, h);
        let overlay = MazeOverlay::new(w, h);
        let builder = match *builder_idx {
            Some(idx) => build_builder_at(idx, &maze),
            None => {
                let (idx, b) = build_builder(&maze);
                *builder_idx = Some(idx);
                b
            }
        };
        (maze, overlay, builder)
    };

    let (mut maze, mut overlay, mut builder) = init_maze(&mut builder_idx, random_mode);
    let mut solver: Option<Box<dyn SolveStrategy>> = None;
    let mut phase = Phase::Build;

    viewer.clear_screen()?;
    viewer.print(&maze)?;
    viewer.print_footer(
        &maze,
        &footer_left("Building", builder.name(), random_mode, paused),
        &footer_right(config.build_steps, config.build_sleep),
    )?;

    loop {
        // --- Phase transitions ---
        if phase == Phase::Build && builder.done() {
            solver = Some(build_solver(&maze));
            overlay.clear();
            phase = Phase::Solve;
            viewer.print_footer(
                &maze,
                &footer_left("Solving", solver.as_ref().unwrap().name(), false, paused),
                &footer_right(config.solve_steps, config.solve_sleep),
            )?;
        }
        if phase == Phase::Solve && solver.as_ref().map_or(true, |s| s.done()) {
            phase = Phase::Complete;
            viewer.print_footer(
                &maze,
                &format!(
                    "Complete  ·  {} → {}",
                    builder.name(),
                    solver.as_ref().unwrap().name()
                ),
                "",
            )?;
        }

        // --- One tick: step + render, then read input ---
        let input = match phase {
            Phase::Build => {
                if !paused {
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
                }
                sleep_and_read(
                    if paused {
                        Duration::ZERO
                    } else {
                        config.build_sleep
                    },
                    paused,
                )?
            }
            Phase::Solve => {
                let solver = solver.as_mut().unwrap();
                if !paused {
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
                }
                sleep_and_read(
                    if paused {
                        Duration::ZERO
                    } else {
                        config.solve_sleep
                    },
                    paused,
                )?
            }
            Phase::Complete => sleep_and_read(config.wait, paused)?,
        };

        // --- Handle input ---
        let mut start_new_maze = phase == Phase::Complete && input == Input::None;

        match input {
            Input::Quit => break,

            Input::NewMaze => {
                if random_mode {
                    builder_idx = None;
                }
                start_new_maze = true;
            }
            Input::CycleBuilder => {
                builder_idx = Some(match builder_idx {
                    Some(i) => (i + 1) % builder_count(),
                    None => 0,
                });
                random_mode = false;
                start_new_maze = true;
            }
            Input::CyclePrevBuilder => {
                builder_idx = Some(match builder_idx {
                    Some(i) => (i + builder_count() - 1) % builder_count(),
                    None => builder_count() - 1,
                });
                random_mode = false;
                start_new_maze = true;
            }
            Input::RandomBuilder => {
                builder_idx = None;
                random_mode = true;
                start_new_maze = true;
            }

            Input::TogglePause => {
                paused = !paused;
                match phase {
                    Phase::Build => viewer.print_footer(
                        &maze,
                        &footer_left("Building", builder.name(), random_mode, paused),
                        &footer_right(config.build_steps, config.build_sleep),
                    )?,
                    Phase::Solve => viewer.print_footer(
                        &maze,
                        &footer_left("Solving", solver.as_ref().unwrap().name(), false, paused),
                        &footer_right(config.solve_steps, config.solve_sleep),
                    )?,
                    Phase::Complete => {}
                }
            }

            Input::SkipSecond => match phase {
                Phase::Build => {
                    skip_one_second_build(&config, builder.as_mut(), &mut maze, viewer.as_ref())?
                }
                Phase::Solve => skip_one_second_solve(
                    &config,
                    solver.as_mut().unwrap().as_mut(),
                    &maze,
                    &mut overlay,
                    viewer.as_ref(),
                )?,
                Phase::Complete => {}
            },

            Input::Advance if paused => match phase {
                Phase::Build => {
                    let mut updated = Vec::new();
                    while updated.is_empty() && !builder.done() {
                        updated.extend(builder.step(&mut maze));
                    }
                    viewer.update_maze(&maze, updated)?;
                }
                Phase::Solve => {
                    let solver = solver.as_mut().unwrap();
                    let mut updated = Vec::new();
                    while updated.is_empty() && !solver.done() {
                        updated.extend(solver.step(&maze, &mut overlay));
                    }
                    viewer.update_overlay(&maze, &overlay, updated)?;
                }
                Phase::Complete => {}
            },

            _ => {}
        }

        if start_new_maze {
            (maze, overlay, builder) = init_maze(&mut builder_idx, random_mode);
            solver = None;
            phase = Phase::Build;
            viewer.clear_screen()?;
            viewer.print(&maze)?;
            viewer.print_footer(
                &maze,
                &footer_left("Building", builder.name(), random_mode, paused),
                &footer_right(config.build_steps, config.build_sleep),
            )?;
        }
    }

    Ok(())
}
