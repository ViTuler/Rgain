import { onBeforeUnmount, ref, watch, type Ref } from "vue";
import {
  isRegistered,
  register,
  unregister,
} from "@tauri-apps/plugin-global-shortcut";

/** Normalize UI hotkey (Ctrl+Shift+E) to Tauri shortcut string. */
export function to_tauri_shortcut(hotkey: string): string {
  return hotkey
    .trim()
    .replace(/\s+/g, "")
    .replace(/Control/gi, "Ctrl")
    .replace(/Ctrl/gi, "CommandOrControl")
    .replace(/CmdOrCtrl/gi, "CommandOrControl")
    .replace(/CommandOrControlOrControl/gi, "CommandOrControl");
}

export function use_explain_hotkey(
  hotkey: Ref<string>,
  on_trigger: () => void | Promise<void>,
) {
  const registered = ref("");
  const error = ref("");

  async function clear_registered() {
    if (!registered.value) return;
    try {
      if (await isRegistered(registered.value)) {
        await unregister(registered.value);
      }
    } catch {
      // ignore unregister races
    }
    registered.value = "";
  }

  async function sync_hotkey(next_raw: string) {
    error.value = "";
    const next = to_tauri_shortcut(next_raw || "Ctrl+Shift+E");
    if (!next) {
      await clear_registered();
      return;
    }
    if (registered.value === next) return;
    await clear_registered();
    try {
      await register(next, async (event) => {
        if (event.state !== "Pressed") return;
        await on_trigger();
      });
      registered.value = next;
    } catch (err) {
      error.value = String(err);
      registered.value = "";
    }
  }

  const stop = watch(
    hotkey,
    (value) => {
      void sync_hotkey(value);
    },
    { immediate: true },
  );

  onBeforeUnmount(() => {
    stop();
    void clear_registered();
  });

  return { registered, error, sync_hotkey, clear_registered };
}
