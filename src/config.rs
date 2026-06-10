use std::{env, io, time::Duration};

use crate::types::RenderStyle;

const DEFAULT_BUILD_SLEEP_SECS: f64 = 0.005;
const DEFAULT_SOLVE_SLEEP_SECS: f64 = 0.005;
const DEFAULT_WAIT_SECS: f64 = 2.0;
const DEFAULT_BUILD_STEPS: usize = 5;
const DEFAULT_SOLVE_STEPS: usize = 1;

pub(crate) struct Config {
    pub(crate) build_sleep: Duration,
    pub(crate) solve_sleep: Duration,
    pub(crate) build_steps: usize,
    pub(crate) solve_steps: usize,
    pub(crate) wait: Duration,
    pub(crate) style: RenderStyle,
}

impl Config {
    pub(crate) fn from_args() -> io::Result<Self> {
        let mut build_sleep_secs = DEFAULT_BUILD_SLEEP_SECS;
        let mut solve_sleep_secs = DEFAULT_SOLVE_SLEEP_SECS;
        let mut build_steps = DEFAULT_BUILD_STEPS;
        let mut solve_steps = DEFAULT_SOLVE_STEPS;
        let mut wait_secs = DEFAULT_WAIT_SECS;
        let mut style = RenderStyle::HalfBlocks;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
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
