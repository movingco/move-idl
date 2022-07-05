pub fn normalize_doc_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(normalize_indentation(s))
    }
}

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
            .split('\n')
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
