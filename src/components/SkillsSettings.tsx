import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { canRemove, primaryLabel, type SkillsStatus } from "../lib/skills";

export const SKILLS_SETTINGS_ID = "skills-settings";

/// One word beside a skill when its file is not what the app wrote.
function skillNote(skill: SkillsStatus["skills"][number]): string | null {
  if (skill.edited_in.length > 0) return "edited";
  if (skill.missing_in.length > 0) return "missing";
  return null;
}

/// The fieldset renders its legend before the first status arrives so the
/// Overview's nudge has something to scroll to on the same frame.
export function SkillsSettings() {
  const [status, setStatus] = useState<SkillsStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    void invoke<SkillsStatus>("skills_status").then(setStatus);
  }, []);

  const run = async (command: "install_skills" | "remove_skills", args?: { force: boolean }) => {
    setBusy(true);
    try {
      setStatus(await invoke<SkillsStatus>(command, args));
    } finally {
      setBusy(false);
    }
  };

  const copy = async () => {
    if (!status) return;
    await writeText(status.npx_command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const editedNames = (status?.skills ?? [])
    .filter((skill) => skill.edited_in.length > 0)
    .map((skill) => skill.name);

  return (
    <fieldset id={SKILLS_SETTINGS_ID}>
      <legend>Agent skills</legend>
      <p>
        The MCP server gives an agent the tools. These skills tell it when to
        use them, how much of the record to read, and what to leave alone.
        Install writes five <code>SKILL.md</code> files where agents look.
      </p>

      {status ? (
        <>
          <ul className="skills-list">
            {status.skills.map((skill) => {
              const note = skillNote(skill);
              return (
                <li key={skill.name}>
                  <code>{skill.name}</code>
                  {note ? <span className="settings-note"> {note}</span> : null}
                  <div className="settings-note">{skill.description}</div>
                </li>
              );
            })}
          </ul>

          <div className="button-row">
            <button
              type="button"
              disabled={busy || status.state === "installed"}
              onClick={() => void run("install_skills", { force: false })}
            >
              {primaryLabel(status.state)}
            </button>
            {canRemove(status.state) ? (
              <button type="button" disabled={busy} onClick={() => void run("remove_skills")}>
                Remove
              </button>
            ) : null}
          </div>

          {(status.skipped.length > 0 || status.skills.some((skill) => skill.edited_in.length > 0)) ? (
            <p className="settings-note">
              {status.skipped.length > 0
                ? `Left alone, because you edited ${status.skipped.length === 1 ? "it" : "them"} or another tool put ${status.skipped.length === 1 ? "it" : "them"} there: ${status.skipped.join(", ")}.`
                : `Edited by you: ${editedNames.join(", ")}. Install and Update leave edited files alone.`}{" "}
              <button
                type="button"
                disabled={busy}
                onClick={() => void run("install_skills", { force: true })}
              >
                Replace my edits
              </button>
            </p>
          ) : null}

          {status.errors.map((error) => (
            <p key={error} className="settings-note">
              {error}
            </p>
          ))}

          <p className="settings-note">
            {status.targets.map((target) => (
              <span key={target.path}>
                {target.path} ({target.read_by})
                <br />
              </span>
            ))}
          </p>

          <p className="settings-note">Prefer your own tooling?</p>
          <pre className="mcp-block">{status.npx_command}</pre>
          <button type="button" onClick={() => void copy()}>
            {copied ? "Copied" : "Copy"}
          </button>
        </>
      ) : null}
    </fieldset>
  );
}
