export type AppSettings = {
  api_base_url: string;
  api_key: string;
  model: string;
  python_path: string;
  explain_hotkey: string;
};

export const default_settings: AppSettings = {
  api_base_url: "https://api.openai.com/v1",
  api_key: "",
  model: "gpt-4o-mini",
  python_path: "E:\\Conda\\envs\\vi\\python.exe",
  explain_hotkey: "Ctrl+Shift+E",
};
