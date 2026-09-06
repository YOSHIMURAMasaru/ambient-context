/// The skills describe the tool surface in prose, and prose goes stale the
/// first week nobody checks it. These tests make the suite the checker.
use ambient_context_lib::mcp::tools;
use ambient_context_lib::skills::{body_after_frontmatter, frontmatter_value, BUNDLED};
use regex::Regex;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn every_bundled_skill_lives_in_a_folder_of_its_own_name() {
    for skill in BUNDLED.iter() {
        assert_eq!(
            frontmatter_value(skill.body, "name").as_deref(),
            Some(skill.name),
            "{} has a frontmatter name that differs from its folder",
            skill.name
        );
        assert!(
            repo_root()
                .join("skills")
                .join(skill.name)
                .join("SKILL.md")
                .exists(),
            "skills/{}/SKILL.md is missing",
            skill.name
        );
    }
}

#[test]
fn names_follow_the_agentskills_rule() {
    let rule = Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
    for skill in BUNDLED.iter() {
        assert!(
            rule.is_match(skill.name),
            "{} breaks the name rule",
            skill.name
        );
        assert!(
            skill.name.len() <= 64,
            "{} is longer than 64 characters",
            skill.name
        );
    }
}

#[test]
fn descriptions_are_present_and_bounded() {
    for skill in BUNDLED.iter() {
        let description = frontmatter_value(skill.body, "description")
            .unwrap_or_else(|| panic!("{} has no description", skill.name));
        assert!(
            (1..=1024).contains(&description.chars().count()),
            "{} has a description outside 1 to 1024 characters",
            skill.name
        );
    }
}

#[test]
fn every_declared_tool_exists() {
    for skill in BUNDLED.iter() {
        let declared = frontmatter_value(skill.body, "tools")
            .unwrap_or_else(|| panic!("{} declares no metadata.tools", skill.name));
        for tool in declared.split_whitespace() {
            assert!(
                tools::exists(tool),
                "{} declares {tool}, which is not a tool",
                skill.name
            );
        }
    }
}

#[test]
fn every_tool_named_in_the_body_is_declared() {
    let ticked = Regex::new("`([a-z_]+)`").unwrap();
    for skill in BUNDLED.iter() {
        let declared: Vec<String> = frontmatter_value(skill.body, "tools")
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let body = body_after_frontmatter(skill.body);
        for capture in ticked.captures_iter(&body) {
            let name = &capture[1];
            if tools::exists(name) {
                assert!(
                    declared.iter().any(|d| d == name),
                    "{} names `{name}` in its body but not in metadata.tools",
                    skill.name
                );
            }
        }
    }
}

#[test]
fn docs_skills_md_matches_the_bundle() {
    let doc = std::fs::read_to_string(repo_root().join("docs/skills.md"))
        .expect("docs/skills.md should exist");
    for skill in BUNDLED.iter() {
        assert!(
            doc.contains(&format!("### `{}`", skill.name)),
            "docs/skills.md has no section for {}",
            skill.name
        );
    }
    for line in doc.lines().filter(|line| line.starts_with("### `")) {
        let name = line.trim_start_matches("### `").trim_end_matches('`');
        assert!(
            BUNDLED.iter().any(|skill| skill.name == name),
            "docs/skills.md documents {name}, which is not bundled"
        );
    }
}
