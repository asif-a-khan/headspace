export const activityTypeColors: Record<string, string> = {
  email: "#A5D6A7",
  note: "#FFCC80",
  call: "#80DEEA",
  meeting: "#90CAF9",
  lunch: "#90CAF9",
  file: "#A5D6A7",
  system: "#FFF59D",
  task: "#A5D6A7",
};

export const activityTypeIcons: Record<string, string> = {
  email: "mdi-email-outline",
  note: "mdi-note-text-outline",
  call: "mdi-phone-outline",
  meeting: "mdi-calendar-clock",
  lunch: "mdi-silverware-fork-knife",
  file: "mdi-file-outline",
  system: "mdi-cog-outline",
  task: "mdi-checkbox-marked-circle-outline",
};

export function getActivityColor(type: string): string {
  return activityTypeColors[type] || "#E0E0E0";
}

export function getActivityIcon(type: string): string {
  return activityTypeIcons[type] || "mdi-calendar-check";
}
