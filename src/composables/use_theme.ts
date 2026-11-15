import { ref } from "vue";

export type Theme_name = "light" | "dark";

const storage_key = "rgain_theme";

const theme_color: Record<Theme_name, string> = {
  light: "#f7f3f3",
  dark: "#1c1716",
};

const theme = ref<Theme_name>("light");

export function normalize_theme(value: string | null | undefined): Theme_name {
  return value === "dark" ? "dark" : "light";
}

export function paint_theme(next: Theme_name) {
  const root = document.documentElement;
  root.dataset.theme = next;
  root.style.colorScheme = next;

  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) meta.setAttribute("content", theme_color[next]);

  try {
    localStorage.setItem(storage_key, next);
  } catch {
    // private mode may block storage
  }

  theme.value = next;
}

export function read_stored_theme(): Theme_name {
  try {
    return normalize_theme(localStorage.getItem(storage_key));
  } catch {
    return "light";
  }
}

export function init_theme() {
  paint_theme(read_stored_theme());
}

export function toggle_theme() {
  paint_theme(theme.value === "dark" ? "light" : "dark");
}

export function use_theme() {
  return {
    theme,
    paint_theme,
    toggle_theme,
    init_theme,
  };
}
