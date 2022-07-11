//! Utilities for manipulating and parsing documentation strings.

/// Removes leading indentation and trailing whitespace from a string, returning `None` if the string is empty.
pub fn normalize_doc_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(remove_asterisks_if_asterisk_docs(&normalize_indentation(s)))
    }
}

/// Removes leading indentation and trailing whitespace from a string.
pub fn normalize_indentation(string: &str) -> String {
    let lines = string.lines();
    if lines.count() == 1 {
        string.trim().to_string()
    } else {
        let num_leading = string
            .lines()
            .filter_map(|l| l.chars().position(|c| !c.is_whitespace()))
            .min()
            .unwrap_or(0);
        string
            .lines()
            .map(|line| {
                if line.len() > num_leading {
                    &line[num_leading..]
                } else {
                    line
                }
            })
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Remove asterisks if the doc string is formatted with asterisks.
pub fn remove_asterisks_if_asterisk_docs(s: &str) -> String {
    let should_remove_asterisks = s.lines().all(|l| l.is_empty() || l.starts_with("* "));
    if should_remove_asterisks {
        s.lines()
            .map(|line| {
                if let Some(fmt) = line.strip_prefix("* ") {
                    fmt
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        s.to_string()
    }
}
