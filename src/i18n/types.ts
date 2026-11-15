export type UiLocale = "zh" | "en" | "ja" | "ko";

export const UI_LOCALES: UiLocale[] = ["zh", "en", "ja", "ko"];

export const DEFAULT_UI_LOCALE: UiLocale = "zh";

export type MessageTree = {
    [key: string]: string | MessageTree;
};

export type LocaleOption = {
    value: UiLocale;
    label: string;
};
