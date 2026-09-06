export type SkillsState =
  | "never_installed"
  | "installed"
  | "update_available"
  | "partial"
  | "removed";

export type SkillInfo = {
  name: string;
  description: string;
  edited_in: string[];
  missing_in: string[];
};

export type SkillsTarget = { path: string; read_by: string };

export type SkillsStatus = {
  state: SkillsState;
  skills: SkillInfo[];
  targets: SkillsTarget[];
  npx_command: string;
  skipped: string[];
  errors: string[];
};

/// The one button's label. Install covers a first install, a repair and a
/// reinstall after Remove; Update is the only other verb the panel needs.
export function primaryLabel(state: SkillsState): "Install" | "Update" | "Installed" {
  switch (state) {
    case "update_available":
      return "Update";
    case "installed":
      return "Installed";
    default:
      return "Install";
  }
}

/// What the Overview says, or nothing. Removed is deliberate and is not
/// nagged about.
export function nudgeLabel(state: SkillsState): string | null {
  switch (state) {
    case "never_installed":
    case "partial":
      return "Install agent skills";
    case "update_available":
      return "Update agent skills";
    default:
      return null;
  }
}

export function canRemove(state: SkillsState): boolean {
  return state === "installed" || state === "update_available" || state === "partial";
}
