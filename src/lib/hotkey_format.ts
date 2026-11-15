const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta", "OS"]);

/** Map a KeyboardEvent into a UI hotkey like `Ctrl+Shift+E`. */
export function format_hotkey_event(event: KeyboardEvent): string | null {
    if (MODIFIER_KEYS.has(event.key)) return null;

    const parts: string[] = [];
    if (event.ctrlKey || event.metaKey) parts.push("Ctrl");
    if (event.altKey) parts.push("Alt");
    if (event.shiftKey) parts.push("Shift");

    const key = normalize_key_token(event);
    if (!key) return null;
    // Require at least one modifier for letter/digit keys to avoid accidental single keys.
    if (parts.length === 0 && /^[A-Z0-9]$/.test(key)) return null;

    parts.push(key);
    return parts.join("+");
}

function normalize_key_token(event: KeyboardEvent): string | null {
    const { key, code } = event;
    if (key === " ") return "Space";
    if (key === "Escape") return "Esc";
    if (key.startsWith("Arrow")) return key.slice(5);
    if (key === "Enter" || key === "Tab" || key === "Backspace" || key === "Delete") {
        return key;
    }
    if (key.length === 1) {
        const upper = key.toUpperCase();
        if (/^[A-Z0-9]$/.test(upper)) return upper;
        // Punctuation etc. — prefer event.code Digit/Key when possible.
        if (code.startsWith("Key") && code.length === 4) return code.slice(3);
        if (code.startsWith("Digit") && code.length === 6) return code.slice(5);
        return upper;
    }
    if (code.startsWith("F") && /^F\d{1,2}$/.test(code)) return code;
    return key.length <= 16 ? key : null;
}

export function is_confirm_key(event: KeyboardEvent): boolean {
    return event.key === "Enter" && !event.ctrlKey && !event.metaKey && !event.altKey;
}

export function is_cancel_key(event: KeyboardEvent): boolean {
    return event.key === "Escape";
}
