import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { callsOf, mockInvoke } from "./tauri-mock";
import { SkillsSettings } from "../components/SkillsSettings";
import type { SkillsStatus } from "../lib/skills";

vi.mock("@tauri-apps/api/core", async () => {
  const mock = await import("./tauri-mock");
  return { invoke: mock.invoke };
});

const writeText = vi.fn();
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (text: string) => writeText(text),
}));

function status(overrides: Partial<SkillsStatus> = {}): SkillsStatus {
  return {
    state: "never_installed",
    skills: [
      { name: "ambient-context", description: "The core loop.", edited_in: [], missing_in: [] },
      { name: "ambient-context-standup", description: "A standup.", edited_in: [], missing_in: [] },
    ],
    targets: [
      { path: "/Users/someone/.claude/skills", read_by: "Claude Code" },
      { path: "/Users/someone/.agents/skills", read_by: "Cursor and others" },
    ],
    npx_command: "npx skills add dragthelake/ambient-context",
    skipped: [],
    errors: [],
    ...overrides,
  };
}

function handler(current: SkillsStatus, afterInstall: SkillsStatus = status({ state: "installed" })) {
  return (command: string) => {
    switch (command) {
      case "skills_status":
        return current;
      case "install_skills":
        return afterInstall;
      case "remove_skills":
        return status({ state: "removed" });
      default:
        throw new Error(`unexpected command ${command}`);
    }
  };
}

describe("SkillsSettings", () => {
  afterEach(cleanup);

  it("lists the skills and offers Install when none are installed", async () => {
    mockInvoke(handler(status()));
    render(<SkillsSettings />);
    expect(await screen.findByText("ambient-context-standup")).toBeTruthy();
    expect(screen.getByText("A standup.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Install" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Remove" })).toBeNull();
    expect(screen.getByText(/\/Users\/someone\/\.claude\/skills/)).toBeTruthy();
  });

  it("offers Update when the bundle is newer", async () => {
    mockInvoke(handler(status({ state: "update_available" })));
    render(<SkillsSettings />);
    expect(await screen.findByRole("button", { name: "Update" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Remove" })).toBeTruthy();
  });

  it("shows Installed, disabled, when everything is current", async () => {
    mockInvoke(handler(status({ state: "installed" })));
    render(<SkillsSettings />);
    const button = await screen.findByRole("button", { name: "Installed" });
    expect(button.hasAttribute("disabled")).toBe(true);
  });

  it("installs on click and shows the new state", async () => {
    mockInvoke(handler(status()));
    render(<SkillsSettings />);
    fireEvent.click(await screen.findByRole("button", { name: "Install" }));
    expect(await screen.findByRole("button", { name: "Installed" })).toBeTruthy();
    expect(callsOf("install_skills")[0].args).toEqual({ force: false });
  });

  it("names skipped edits and offers to replace them", async () => {
    const skipped = status({
      state: "installed",
      skipped: ["/Users/someone/.claude/skills/ambient-context/SKILL.md"],
      skills: [
        {
          name: "ambient-context",
          description: "The core loop.",
          edited_in: ["/Users/someone/.claude/skills"],
          missing_in: [],
        },
      ],
    });
    mockInvoke(handler(status(), skipped));
    render(<SkillsSettings />);
    fireEvent.click(await screen.findByRole("button", { name: "Install" }));
    expect(await screen.findByText(/Left alone/)).toBeTruthy();
    expect(screen.getByText("edited")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Replace my edits" }));
    await waitFor(() => expect(callsOf("install_skills").length).toBe(2));
    expect(callsOf("install_skills")[1].args).toEqual({ force: true });
  });

  it("offers Replace my edits while installed with an edited skill", async () => {
    mockInvoke(
      handler(
        status({
          state: "installed",
          skills: [
            {
              name: "ambient-context",
              description: "The core loop.",
              edited_in: ["/Users/someone/.claude/skills"],
              missing_in: [],
            },
          ],
        }),
      ),
    );
    render(<SkillsSettings />);
    expect(await screen.findByText(/Edited by you: ambient-context/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Replace my edits" }));
    await waitFor(() => expect(callsOf("install_skills").length).toBe(1));
    expect(callsOf("install_skills")[0].args).toEqual({ force: true });
  });

  it("shows errors from the backend", async () => {
    mockInvoke(handler(status({ state: "partial", errors: ["/x/SKILL.md: Permission denied"] })));
    render(<SkillsSettings />);
    expect(await screen.findByText("/x/SKILL.md: Permission denied")).toBeTruthy();
  });

  it("removes on click", async () => {
    mockInvoke(handler(status({ state: "installed" })));
    render(<SkillsSettings />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove" }));
    expect(await screen.findByRole("button", { name: "Install" })).toBeTruthy();
    expect(callsOf("remove_skills").length).toBe(1);
  });

  it("copies the npx command", async () => {
    mockInvoke(handler(status()));
    render(<SkillsSettings />);
    fireEvent.click(await screen.findByRole("button", { name: "Copy" }));
    expect(writeText).toHaveBeenCalledWith("npx skills add dragthelake/ambient-context");
  });
});
