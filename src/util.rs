#[cfg(test)]
mod tests {
    use super::normalize_name;

    #[test]
    fn lowercase_and_hyphenate() {
        assert_eq!(
            normalize_name("Recursive Backtracker"),
            "recursive-backtracker"
        );
    }

    #[test]
    fn strips_ascii_apostrophe_s() {
        assert_eq!(normalize_name("Prim's"), "prim");
    }

    #[test]
    fn strips_curly_apostrophe_s() {
        assert_eq!(normalize_name("Prim\u{2019}s"), "prim");
    }

    #[test]
    fn preserves_existing_hyphens() {
        assert_eq!(normalize_name("Flood-Fill"), "flood-fill");
    }

    #[test]
    fn strips_other_punctuation() {
        assert_eq!(normalize_name("A* Search!"), "a-search");
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_name(""), "");
    }
}

pub(crate) fn normalize_name(name: &str) -> String {
    let name = name
        .strip_suffix("'s")
        .or_else(|| name.strip_suffix("\u{2019}s"))
        .unwrap_or(name);
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}
