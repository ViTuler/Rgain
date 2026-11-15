import { computed, ref } from "vue";
import type { LocaleOption, MessageTree, UiLocale } from "./types";
import { DEFAULT_UI_LOCALE, UI_LOCALES } from "./types";
import zh from "./locales/zh";
import en from "./locales/en";
import ja from "./locales/ja";
import ko from "./locales/ko";

const catalogs: Record<UiLocale, MessageTree> = { zh, en, ja, ko };

const locale = ref<UiLocale>(DEFAULT_UI_LOCALE);

function lookup(tree: MessageTree, path: string): string | undefined {
    const parts = path.split(".");
    let cur: string | MessageTree | undefined = tree;
    for (const part of parts) {
        if (!cur || typeof cur === "string") return undefined;
        cur = cur[part];
    }
    return typeof cur === "string" ? cur : undefined;
}

function format(template: string, params?: Record<string, string | number>): string {
    if (!params) return template;
    return template.replace(/\{(\w+)\}/g, (_, key: string) => {
        const value = params[key];
        return value == null ? `{${key}}` : String(value);
    });
}

export function is_ui_locale(value: unknown): value is UiLocale {
    return typeof value === "string" && (UI_LOCALES as string[]).includes(value);
}

export function normalize_ui_locale(value: unknown): UiLocale {
    return is_ui_locale(value) ? value : DEFAULT_UI_LOCALE;
}

export function t(key: string, params?: Record<string, string | number>): string {
    const primary = lookup(catalogs[locale.value], key);
    if (primary != null) return format(primary, params);
    const fallback = lookup(catalogs[DEFAULT_UI_LOCALE], key);
    if (fallback != null) return format(fallback, params);
    return key;
}

export function set_locale(next: UiLocale) {
    locale.value = normalize_ui_locale(next);
    if (typeof document !== "undefined") {
        document.documentElement.lang = locale.value;
    }
}

export function get_locale(): UiLocale {
    return locale.value;
}

export const locale_options = computed<LocaleOption[]>(() =>
    UI_LOCALES.map((value) => ({
        value,
        label: t(`locale.${value}`),
    })),
);

export function use_i18n() {
    return {
        locale,
        t,
        set_locale,
        locale_options,
        locales: UI_LOCALES,
    };
}

export type { UiLocale, LocaleOption, MessageTree };
export { DEFAULT_UI_LOCALE, UI_LOCALES };
