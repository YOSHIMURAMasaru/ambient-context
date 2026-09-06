//! The agent skills the app ships and the install that puts them where
//! agents look. The binary is the source: each SKILL.md is compiled in the
//! way the prompts are, and install copies them out with a manifest of what
//! was written, so a later run can tell an app-written file from one the
//! user has edited.

pub struct Skill {
    pub name: &'static str,
    pub body: &'static str,
}

pub const BUNDLED: [Skill; 5] = [
    Skill {
        name: "ambient-context",
        body: include_str!("../../skills/ambient-context/SKILL.md"),
    },
    Skill {
        name: "ambient-context-catch-me-up",
        body: include_str!("../../skills/ambient-context-catch-me-up/SKILL.md"),
    },
    Skill {
        name: "ambient-context-standup",
        body: include_str!("../../skills/ambient-context-standup/SKILL.md"),
    },
    Skill {
        name: "ambient-context-weekly-review",
        body: include_str!("../../skills/ambient-context-weekly-review/SKILL.md"),
    },
    Skill {
        name: "ambient-context-tune-rules",
        body: include_str!("../../skills/ambient-context-tune-rules/SKILL.md"),
    },
];

/// The value of a `key:` line in the frontmatter, quotes stripped. Nested
/// keys (`tools:` under `metadata:`) are found by the same scan, which is
/// enough for the five files this crate owns; there is no YAML parser in
/// the dependency tree and these files do not justify one.
pub fn frontmatter_value(body: &str, key: &str) -> Option<String> {
    let mut lines = body.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        let Some(rest) = line.trim_start().strip_prefix(key) else {
            continue;
        };
        let Some(value) = rest.strip_prefix(':') else {
            continue;
        };
        return Some(value.trim().trim_matches('"').to_string());
    }
    None
}

/// Everything after the closing `---` of the frontmatter.
pub fn body_after_frontmatter(body: &str) -> String {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        return body.to_string();
    }
    let rest: Vec<&str> = lines
        .skip_while(|line| line.trim() != "---")
        .skip(1)
        .collect();
    rest.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_value_reads_top_level_and_nested_keys() {
        let body = "---\nname: a-skill\ndescription: \"Quoted.\"\nmetadata:\n  tools: \"read_day list_days\"\n---\n# Body\n`name:` is not frontmatter\n";
        assert_eq!(frontmatter_value(body, "name").as_deref(), Some("a-skill"));
        assert_eq!(
            frontmatter_value(body, "description").as_deref(),
            Some("Quoted.")
        );
        assert_eq!(
            frontmatter_value(body, "tools").as_deref(),
            Some("read_day list_days")
        );
        assert_eq!(frontmatter_value(body, "absent"), None);
        assert_eq!(frontmatter_value("no frontmatter", "name"), None);
    }

    #[test]
    fn body_after_frontmatter_drops_the_block() {
        let body = "---\nname: a\n---\n# Body\nline\n";
        assert_eq!(body_after_frontmatter(body), "# Body\nline");
        assert_eq!(body_after_frontmatter("plain"), "plain");
    }
}
