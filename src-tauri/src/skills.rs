//! The agent skills the app ships and the install that puts them where
//! agents look. The binary is the source: each SKILL.md is compiled in the
//! way the prompts are, and install copies them out with a manifest of what
//! was written, so a later run can tell an app-written file from one the
//! user has edited.

use crate::ledger::sha256_of;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

pub const NPX_COMMAND: &str = "npx skills add dragthelake/ambient-context";

/// The user's home. `AMBIENT_CONTEXT_SKILLS_HOME` redirects it for a manual
/// test against a scratch directory; the unit tests pass a path instead.
pub fn home() -> PathBuf {
    std::env::var("AMBIENT_CONTEXT_SKILLS_HOME")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Target {
    pub path: String,
    pub read_by: String,
}

/// The two directories that between them reach every agent that reads the
/// SKILL.md format. Claude Code has its own; the rest converged on
/// `.agents`.
pub fn targets(home: &Path) -> Vec<Target> {
    vec![
        Target {
            path: home.join(".claude/skills").to_string_lossy().into_owned(),
            read_by: "Claude Code".to_string(),
        },
        Target {
            path: home.join(".agents/skills").to_string_lossy().into_owned(),
            read_by: "Cursor, Codex CLI, Zed, GitHub Copilot, Gemini CLI, Goose and OpenCode"
                .to_string(),
        },
    ]
}

fn skill_path(target: &str, name: &str) -> PathBuf {
    Path::new(target).join(name).join("SKILL.md")
}

/// What the app wrote, so a later run can tell its own file from an edit.
/// Never the sole source of status: the directories are read every time.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub app_version: String,
    pub installed_at: String,
    pub removed_at: Option<String>,
    /// Target directory to skill name to the sha256 the app wrote there.
    pub targets: BTreeMap<String, BTreeMap<String, String>>,
}

pub fn manifest_path(data_dir: &Path) -> PathBuf {
    data_dir.join("skills-install.json")
}

fn read_manifest(data_dir: &Path) -> Option<Manifest> {
    fs::read_to_string(manifest_path(data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn write_manifest(data_dir: &Path, manifest: &Manifest) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let json = serde_json::to_string_pretty(manifest).map_err(std::io::Error::other)?;
    fs::write(manifest_path(data_dir), json)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    NeverInstalled,
    Installed,
    UpdateAvailable,
    Partial,
    Removed,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    /// Targets where the file matches neither the bundle nor the manifest.
    pub edited_in: Vec<String>,
    /// Targets where the file is absent.
    pub missing_in: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Status {
    pub state: State,
    pub skills: Vec<SkillInfo>,
    pub targets: Vec<Target>,
    pub npx_command: String,
    /// Paths an install left alone because the user had edited them.
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

fn recorded_hash<'a>(
    manifest: Option<&'a Manifest>,
    target: &str,
    name: &str,
) -> Option<&'a String> {
    manifest?.targets.get(target)?.get(name)
}

pub fn status(home: &Path, data_dir: &Path) -> Status {
    let targets = targets(home);
    let manifest = read_manifest(data_dir);
    let inspect_disk = manifest.as_ref().is_some_and(|m| m.removed_at.is_none());
    let mut any_missing = false;
    let mut any_update = false;
    let mut skills = Vec::with_capacity(BUNDLED.len());
    for skill in BUNDLED.iter() {
        let mut info = SkillInfo {
            name: skill.name.to_string(),
            description: frontmatter_value(skill.body, "description").unwrap_or_default(),
            edited_in: Vec::new(),
            missing_in: Vec::new(),
        };
        if inspect_disk {
            let bundled = sha256_of(skill.body.as_bytes());
            for target in &targets {
                match fs::read(skill_path(&target.path, skill.name)) {
                    Err(_) => {
                        any_missing = true;
                        info.missing_in.push(target.path.clone());
                    }
                    Ok(bytes) => {
                        let on_disk = sha256_of(&bytes);
                        if on_disk == bundled {
                            continue;
                        }
                        if recorded_hash(manifest.as_ref(), &target.path, skill.name)
                            == Some(&on_disk)
                        {
                            any_update = true;
                        } else {
                            info.edited_in.push(target.path.clone());
                        }
                    }
                }
            }
        }
        skills.push(info);
    }
    let state = match &manifest {
        None => State::NeverInstalled,
        Some(m) if m.removed_at.is_some() => State::Removed,
        _ if any_missing => State::Partial,
        _ if any_update => State::UpdateAvailable,
        _ => State::Installed,
    };
    Status {
        state,
        skills,
        targets,
        npx_command: NPX_COMMAND.to_string(),
        skipped: Vec::new(),
        errors: Vec::new(),
    }
}

pub struct Outcome {
    pub status: Status,
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// True when the app can claim the file: it matches the current bundle or
/// the hash the manifest recorded. Anything else is the user's.
fn app_wrote(on_disk: &str, bundled: &str, recorded: Option<&String>) -> bool {
    on_disk == bundled || recorded.map(String::as_str) == Some(on_disk)
}

/// Writes every skill to every target. Serves Install, Update and the
/// repair of a partial install alike; the result says which files moved.
pub fn install(home: &Path, data_dir: &Path, force: bool) -> Outcome {
    let previous = read_manifest(data_dir).unwrap_or_default();
    let mut manifest = Manifest {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        installed_at: chrono::Local::now().to_rfc3339(),
        removed_at: None,
        targets: BTreeMap::new(),
    };
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();
    for target in targets(home) {
        let mut entry = BTreeMap::new();
        for skill in BUNDLED.iter() {
            let path = skill_path(&target.path, skill.name);
            let bundled = sha256_of(skill.body.as_bytes());
            let on_disk = fs::read(&path).ok().map(|bytes| sha256_of(&bytes));
            let recorded = previous
                .targets
                .get(&target.path)
                .and_then(|t| t.get(skill.name));
            if let Some(on_disk) = &on_disk {
                if !force && !app_wrote(on_disk, &bundled, recorded) {
                    skipped.push(path);
                    continue;
                }
            }
            if on_disk.as_deref() != Some(bundled.as_str()) {
                let result = match path.parent() {
                    Some(dir) => fs::create_dir_all(dir).and_then(|_| fs::write(&path, skill.body)),
                    None => Err(std::io::Error::other("no parent directory")),
                };
                if let Err(error) = result {
                    errors.push(format!("{}: {error}", display(&path)));
                    continue;
                }
                written.push(path);
            }
            entry.insert(skill.name.to_string(), bundled);
        }
        manifest.targets.insert(target.path.clone(), entry);
    }
    if let Err(error) = write_manifest(data_dir, &manifest) {
        errors.push(format!("{}: {error}", display(&manifest_path(data_dir))));
    }
    let mut status = status(home, data_dir);
    status.skipped = skipped.iter().map(|p| display(p)).collect();
    status.errors = errors;
    Outcome {
        status,
        written,
        skipped,
        deleted: Vec::new(),
    }
}

/// Deletes the files the app wrote and nothing else. The manifest stays,
/// marked removed, so the Overview does not ask again.
pub fn remove(home: &Path, data_dir: &Path) -> Outcome {
    let mut manifest = read_manifest(data_dir).unwrap_or_default();
    let mut deleted = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();
    for target in targets(home) {
        let recorded = manifest
            .targets
            .get(&target.path)
            .cloned()
            .unwrap_or_default();
        let mut kept = BTreeMap::new();
        for skill in BUNDLED.iter() {
            let path = skill_path(&target.path, skill.name);
            let Ok(bytes) = fs::read(&path) else { continue };
            let on_disk = sha256_of(&bytes);
            if !app_wrote(
                &on_disk,
                &sha256_of(skill.body.as_bytes()),
                recorded.get(skill.name),
            ) {
                if let Some(hash) = recorded.get(skill.name) {
                    kept.insert(skill.name.to_string(), hash.clone());
                }
                skipped.push(path);
                continue;
            }
            match fs::remove_file(&path) {
                Ok(()) => {
                    if let Some(dir) = path.parent() {
                        let _ = fs::remove_dir(dir);
                    }
                    deleted.push(path);
                }
                Err(error) => errors.push(format!("{}: {error}", display(&path))),
            }
        }
        manifest.targets.insert(target.path.clone(), kept);
    }
    manifest.removed_at = Some(chrono::Local::now().to_rfc3339());
    if let Err(error) = write_manifest(data_dir, &manifest) {
        errors.push(format!("{}: {error}", display(&manifest_path(data_dir))));
    }
    let mut status = status(home, data_dir);
    status.skipped = skipped.iter().map(|p| display(p)).collect();
    status.errors = errors;
    Outcome {
        status,
        written: Vec::new(),
        skipped,
        deleted,
    }
}

/// One ledger line per install, update or remove, listing every path that
/// moved. The ledger is how the user later answers "when did this agent
/// start behaving differently".
pub fn record(folder: &Path, action: &str, outcome: &Outcome) -> std::io::Result<PathBuf> {
    let mut lines = Vec::new();
    lines.extend(
        outcome
            .written
            .iter()
            .map(|p| format!("written: {}", p.display())),
    );
    lines.extend(
        outcome
            .deleted
            .iter()
            .map(|p| format!("deleted: {}", p.display())),
    );
    lines.extend(
        outcome
            .skipped
            .iter()
            .map(|p| format!("skipped: {}", p.display())),
    );
    lines.extend(outcome.status.errors.iter().map(|e| format!("error: {e}")));
    crate::ledger::append(
        folder,
        &crate::ledger::Entry {
            at: chrono::Local::now(),
            trigger: crate::ledger::Trigger::Settings,
            action: action.to_string(),
            prompt_id: None,
            prompt_sha256: None,
            engine: None,
            inputs: Vec::new(),
            output: Some(lines.join("\n")),
            reasoning: None,
            took_ms: None,
            disposition: crate::ledger::Disposition::Applied,
        },
    )
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

    fn fresh() -> (tempfile::TempDir, tempfile::TempDir) {
        (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap())
    }

    fn write_manifest_for_test(data_dir: &std::path::Path, manifest: &Manifest) {
        std::fs::write(
            manifest_path(data_dir),
            serde_json::to_string_pretty(manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn targets_are_claude_then_agents_under_the_given_home() {
        let home = std::path::Path::new("/Users/someone");
        let targets = targets(home);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].path, "/Users/someone/.claude/skills");
        assert_eq!(targets[0].read_by, "Claude Code");
        assert_eq!(targets[1].path, "/Users/someone/.agents/skills");
        assert!(targets[1].read_by.contains("Cursor"));
    }

    #[test]
    fn never_installed_before_the_first_install() {
        let (home, data) = fresh();
        let status = status(home.path(), data.path());
        assert_eq!(status.state, State::NeverInstalled);
        assert_eq!(status.skills.len(), 5);
        assert_eq!(status.skills[0].name, "ambient-context");
        assert!(status.skills.iter().all(|s| !s.description.is_empty()));
        assert_eq!(
            status.npx_command,
            "npx skills add dragthelake/ambient-context"
        );
        assert!(status.skipped.is_empty());
        assert!(status.errors.is_empty());
    }

    #[test]
    fn a_manifest_with_removed_at_is_removed() {
        let (home, data) = fresh();
        let manifest = Manifest {
            removed_at: Some("2026-09-06T10:00:00+10:00".into()),
            ..Manifest::default()
        };
        write_manifest_for_test(data.path(), &manifest);
        assert_eq!(status(home.path(), data.path()).state, State::Removed);
    }

    #[test]
    fn a_manifest_with_nothing_on_disk_is_partial() {
        let (home, data) = fresh();
        write_manifest_for_test(data.path(), &Manifest::default());
        let status = status(home.path(), data.path());
        assert_eq!(status.state, State::Partial);
        assert_eq!(status.skills[0].missing_in.len(), 2);
    }

    #[test]
    fn files_matching_the_bundle_are_installed_and_an_older_app_copy_is_an_update() {
        let (home, data) = fresh();
        let mut manifest = Manifest::default();
        for target in targets(home.path()) {
            let mut entry = std::collections::BTreeMap::new();
            for skill in BUNDLED.iter() {
                let path = skill_path(&target.path, skill.name);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, skill.body).unwrap();
                entry.insert(
                    skill.name.to_string(),
                    crate::ledger::sha256_of(skill.body.as_bytes()),
                );
            }
            manifest.targets.insert(target.path.clone(), entry);
        }
        write_manifest_for_test(data.path(), &manifest);
        let result = status(home.path(), data.path());
        assert_eq!(result.state, State::Installed);

        // An older release wrote a different body; the manifest knows that
        // body, so this is the app's own file and an update is due.
        let older = "older body";
        let claude = &targets(home.path())[0].path;
        std::fs::write(skill_path(claude, "ambient-context-standup"), older).unwrap();
        manifest.targets.get_mut(claude).unwrap().insert(
            "ambient-context-standup".into(),
            crate::ledger::sha256_of(older.as_bytes()),
        );
        write_manifest_for_test(data.path(), &manifest);
        let result = status(home.path(), data.path());
        assert_eq!(result.state, State::UpdateAvailable);
        assert!(result.skills[2].edited_in.is_empty());

        // A body the manifest never saw is the user's edit.
        std::fs::write(skill_path(claude, "ambient-context-standup"), "my edit").unwrap();
        let result = status(home.path(), data.path());
        assert_eq!(result.state, State::Installed);
        assert_eq!(result.skills[2].edited_in, vec![claude.clone()]);
    }

    #[test]
    fn fresh_install_writes_every_skill_to_both_targets() {
        let (home, data) = fresh();
        let out = install(home.path(), data.path(), false);
        assert_eq!(out.written.len(), 10);
        assert!(out.skipped.is_empty());
        assert!(home
            .path()
            .join(".claude/skills/ambient-context/SKILL.md")
            .exists());
        assert!(home
            .path()
            .join(".agents/skills/ambient-context-tune-rules/SKILL.md")
            .exists());
        assert_eq!(out.status.state, State::Installed);
        assert!(out.status.errors.is_empty());
        let manifest: Manifest =
            serde_json::from_str(&std::fs::read_to_string(manifest_path(data.path())).unwrap())
                .unwrap();
        assert_eq!(manifest.app_version, env!("CARGO_PKG_VERSION"));
        assert!(manifest.removed_at.is_none());
        assert_eq!(manifest.targets.len(), 2);
        assert_eq!(manifest.targets.values().next().unwrap().len(), 5);
    }

    #[test]
    fn installing_again_writes_nothing() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        let again = install(home.path(), data.path(), false);
        assert!(again.written.is_empty());
        assert_eq!(again.status.state, State::Installed);
    }

    #[test]
    fn a_missing_target_is_partial_and_install_repairs_it() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        std::fs::remove_dir_all(home.path().join(".agents")).unwrap();
        assert_eq!(status(home.path(), data.path()).state, State::Partial);
        let repaired = install(home.path(), data.path(), false);
        assert_eq!(repaired.written.len(), 5);
        assert_eq!(repaired.status.state, State::Installed);
    }

    #[test]
    fn an_edited_file_is_kept_unless_forced() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        let edited = home.path().join(".claude/skills/ambient-context/SKILL.md");
        std::fs::write(&edited, "my edit").unwrap();

        let out = install(home.path(), data.path(), false);
        assert_eq!(out.skipped, vec![edited.clone()]);
        assert!(out.written.is_empty());
        assert_eq!(std::fs::read_to_string(&edited).unwrap(), "my edit");
        assert_eq!(
            out.status.skipped,
            vec![edited.to_string_lossy().into_owned()]
        );
        assert_eq!(out.status.skills[0].edited_in.len(), 1);

        let forced = install(home.path(), data.path(), true);
        assert_eq!(forced.written, vec![edited.clone()]);
        assert_eq!(std::fs::read_to_string(&edited).unwrap(), BUNDLED[0].body);
        assert!(forced.status.skills[0].edited_in.is_empty());
    }

    #[test]
    fn a_file_the_app_wrote_under_an_older_bundle_is_overwritten() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        let older = "older body";
        let path = home
            .path()
            .join(".agents/skills/ambient-context-standup/SKILL.md");
        std::fs::write(&path, older).unwrap();
        let mut manifest: Manifest =
            serde_json::from_str(&std::fs::read_to_string(manifest_path(data.path())).unwrap())
                .unwrap();
        let agents = targets(home.path())[1].path.clone();
        manifest.targets.get_mut(&agents).unwrap().insert(
            "ambient-context-standup".into(),
            crate::ledger::sha256_of(older.as_bytes()),
        );
        write_manifest_for_test(data.path(), &manifest);
        assert_eq!(
            status(home.path(), data.path()).state,
            State::UpdateAvailable
        );

        let out = install(home.path(), data.path(), false);
        assert_eq!(out.written, vec![path.clone()]);
        assert_eq!(out.status.state, State::Installed);
    }

    #[test]
    fn a_file_the_app_never_wrote_is_treated_as_the_users() {
        let (home, data) = fresh();
        let theirs = home.path().join(".agents/skills/ambient-context/SKILL.md");
        std::fs::create_dir_all(theirs.parent().unwrap()).unwrap();
        std::fs::write(&theirs, "installed by npx, then edited").unwrap();
        let out = install(home.path(), data.path(), false);
        assert_eq!(out.written.len(), 9);
        assert_eq!(out.skipped, vec![theirs.clone()]);
        assert_eq!(
            std::fs::read_to_string(&theirs).unwrap(),
            "installed by npx, then edited"
        );
    }

    #[test]
    fn remove_deletes_app_written_files_and_keeps_edits() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        let edited = home
            .path()
            .join(".claude/skills/ambient-context-standup/SKILL.md");
        std::fs::write(&edited, "my edit").unwrap();

        let out = remove(home.path(), data.path());
        assert_eq!(out.deleted.len(), 9);
        assert_eq!(out.skipped, vec![edited.clone()]);
        assert!(edited.exists());
        assert!(!home.path().join(".claude/skills/ambient-context").exists());
        assert!(!home
            .path()
            .join(".agents/skills/ambient-context-standup/SKILL.md")
            .exists());
        assert_eq!(out.status.state, State::Removed);
        let manifest: Manifest =
            serde_json::from_str(&std::fs::read_to_string(manifest_path(data.path())).unwrap())
                .unwrap();
        assert!(manifest.removed_at.is_some());
    }

    #[test]
    fn install_after_remove_is_installed_again() {
        let (home, data) = fresh();
        install(home.path(), data.path(), false);
        remove(home.path(), data.path());
        let out = install(home.path(), data.path(), false);
        assert_eq!(out.written.len(), 10);
        assert_eq!(out.status.state, State::Installed);
    }

    #[test]
    fn an_unwritable_target_is_reported_not_fatal() {
        let (home, data) = fresh();
        // A file where the directory should be: create_dir_all fails.
        std::fs::create_dir_all(home.path().join(".agents")).unwrap();
        std::fs::write(home.path().join(".agents/skills"), "not a directory").unwrap();
        let out = install(home.path(), data.path(), false);
        assert_eq!(out.written.len(), 5);
        assert_eq!(out.status.errors.len(), 5);
        assert_eq!(out.status.state, State::Partial);
    }

    #[test]
    fn install_update_and_remove_each_leave_a_ledger_entry() {
        let (home, data) = fresh();
        let folder = tempfile::tempdir().unwrap();

        let installed = install(home.path(), data.path(), false);
        let path = record(folder.path(), "install_skills", &installed).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("install_skills"));
        assert!(text.contains("written: "));
        assert!(text.contains("ambient-context-standup/SKILL.md"));

        let edited = home.path().join(".claude/skills/ambient-context/SKILL.md");
        std::fs::write(&edited, "my edit").unwrap();
        let updated = install(home.path(), data.path(), false);
        record(folder.path(), "update_skills", &updated).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("update_skills"));
        assert!(text.contains(&format!("skipped: {}", edited.display())));

        let removed = remove(home.path(), data.path());
        record(folder.path(), "remove_skills", &removed).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("remove_skills"));
        assert!(text.contains("deleted: "));
    }
}
