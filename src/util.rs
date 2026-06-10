pub(crate) fn normalize_name(name: &str) -> String {
    let name = name
        .strip_suffix("'s")
        .or_else(|| name.strip_suffix('\u{2019}'))
        .unwrap_or(name);
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}
