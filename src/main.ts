import { createApp } from "vue";
import App from "./App.vue";
import "./styles/tokens.css";
import { init_theme } from "./composables/use_theme";

init_theme();
createApp(App).mount("#app");
