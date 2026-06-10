use std::{env, io, time::Duration};

use crate::{builders, solvers, types::RenderStyle};

const DEFAULT_BUILD_SLEEP_SECS: f64 = 1.0 / 15.0;
const DEFAULT_SOLVE_SLEEP_SECS: f64 = 1.0 / 15.0;
const DEFAULT_WAIT_SECS: f64 = 2.0;
const DEFAULT_BUILD_STEPS: usize = 15;
const DEFAULT_SOLVE_STEPS: usize = 15;

pub(crate) struct Config {
    pub(crate) build_sleep: Duration,
    pub(crate) solve_sleep: Duration,
    pub(crate) build_steps: usize,
    pub(crate) solve_steps: usize,
    pub(crate) wait: Duration,
    pub(crate) style: RenderStyle,
    pub(crate) builder: Option<usize>,
    pub(crate) solver: Option<usize>,
    pub(crate) random: bool,
}

impl Config {
    pub(crate) fn from_args() -> io::Result<Self> {
        let mut build_sleep_secs = DEFAULT_BUILD_SLEEP_SECS;
        let mut solve_sleep_secs = DEFAULT_SOLVE_SLEEP_SECS;
        let mut build_steps = DEFAULT_BUILD_STEPS;
        let mut solve_steps = DEFAULT_SOLVE_STEPS;
        let mut wait_secs = DEFAULT_WAIT_SECS;
        let mut style = RenderStyle::HalfBlocks;
        let mut builder: Option<usize> = None;
        let mut solver: Option<usize> = None;
        let mut random: Option<bool> = None;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--build-sleep" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "missing value after --build-sleep",
                        )
                    })?;
                    build_sleep_secs = parse_sleep_secs(&value, "--build-sleep")?;
                }
                "--solve-sleep" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "missing value after --solve-sleep",
                        )
                    })?;
                    solve_sleep_secs = parse_sleep_secs(&value, "--solve-sleep")?;
                }
                "--build-steps" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "missing value after --build-steps",
                        )
                    })?;
                    build_steps = parse_step_count(&value, "--build-steps")?;
                }
                "--solve-steps" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "missing value after --solve-steps",
                        )
                    })?;
                    solve_steps = parse_step_count(&value, "--solve-steps")?;
                }
                "--wait" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing value after --wait")
                    })?;
                    wait_secs = parse_sleep_secs(&value, "--wait")?;
                }
                "--style" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing value after --style")
                    })?;
                    style = parse_style(&value)?;
                }
                "-b" | "--builder" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("missing value after {arg}"),
                        )
                    })?;
                    builder = Some(builders::find_builder_index(&value).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "unknown builder {value:?}: valid builders are: {}",
                                builders::builder_names().join(", ")
                            ),
                        )
                    })?);
                    random = Some(false);
                }
                "-s" | "--solver" => {
                    let value = args.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("missing value after {arg}"),
                        )
                    })?;
                    solver = Some(solvers::find_solver_index(&value).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "unknown solver {value:?}: valid solvers are: {}",
                                solvers::solver_names().join(", ")
                            ),
                        )
                    })?);
                }
                "-r" | "--random" => {
                    builder = None;
                    random = Some(true);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unknown argument: {arg}"),
                    ));
                }
            }
        }

        Ok(Self {
            build_sleep: Duration::from_secs_f64(build_sleep_secs),
            solve_sleep: Duration::from_secs_f64(solve_sleep_secs),
            build_steps,
            solve_steps,
            wait: Duration::from_secs_f64(wait_secs),
            style,
            builder,
            solver,
            random: random.unwrap_or(builder.is_none()),
        })
    }
}

fn parse_sleep_secs(value: &str, flag: &str) -> io::Result<f64> {
    let normalized = value.strip_suffix('s').unwrap_or(value);
    let sleep_secs = normalized.parse::<f64>().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid {flag} value {value:?}: {err}"),
        )
    })?;
    if sleep_secs < 0.0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{flag} must be non-negative"),
        ));
    }
    Ok(sleep_secs)
}

fn parse_step_count(value: &str, flag: &str) -> io::Result<usize> {
    let steps = value.parse::<usize>().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid {flag} value {value:?}: {err}"),
        )
    })?;
    if steps == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{flag} must be at least 1"),
        ));
    }
    Ok(steps)
}

fn print_help() {
    let build_fps = (1.0 / DEFAULT_BUILD_SLEEP_SECS).round() as usize;
    let solve_fps = (1.0 / DEFAULT_SOLVE_SLEEP_SECS).round() as usize;
    let builders = builders::builder_names().join(", ");
    let solvers = solvers::solver_names().join(", ");
    println!(
        "Usage: mazes [OPTIONS]

Options:
  -b, --builder <name>    Pin the maze builder (default: random)
  -r, --random            Use a random builder [default]
  -s, --solver <name>     Pin the maze solver (default: random)
      --build-steps <n>   Steps per tick during build (default: {DEFAULT_BUILD_STEPS})
      --solve-steps <n>   Steps per tick during solve (default: {DEFAULT_SOLVE_STEPS})
      --build-sleep <s>   Seconds between build ticks (default: {DEFAULT_BUILD_SLEEP_SECS:.4}, {build_fps} fps)
      --solve-sleep <s>   Seconds between solve ticks (default: {DEFAULT_SOLVE_SLEEP_SECS:.4}, {solve_fps} fps)
      --wait <s>          Seconds to display completed maze (default: {DEFAULT_WAIT_SECS})
      --style <style>     Render style: lines, blocks, half-blocks (default: half-blocks)
  -h, --help              Show this help

Builders: {builders}
Solvers:  {solvers}"
    );
}

fn parse_style(value: &str) -> io::Result<RenderStyle> {
    match value {
        "lines" => Ok(RenderStyle::Lines),
        "blocks" => Ok(RenderStyle::Blocks),
        "half-blocks" => Ok(RenderStyle::HalfBlocks),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid --style value {value:?}: expected lines, blocks, or half-blocks"),
        )),
    }
}
