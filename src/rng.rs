use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{ALL_DIRS, Dir};

pub(crate) fn xorshift(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}

pub(crate) fn shuffle_dirs(seed: &mut u32) -> [Dir; 4] {
    let mut dirs = ALL_DIRS;
    for i in (1..4).rev() {
        let j = (xorshift(seed) as usize) % (i + 1);
        dirs.swap(i, j);
    }
    dirs
}

pub(crate) fn time_seed() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let n = nanos as u64;
    (n ^ (n >> 32)) as u32
}
