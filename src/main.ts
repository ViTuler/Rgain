import { createApp } from "vue";
import App from "./App.vue";
import "./styles/tokens.css";
import { init_theme } from "./composables/use_theme";
import { DEFAULT_UI_LOCALE, set_locale } from "./i18n";

init_theme();
set_locale(DEFAULT_UI_LOCALE);
createApp(App).mount("#app");
