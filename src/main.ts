import { createApp } from "vue";
import App from "./App.vue";
import { setupConsoleRedirect } from "./utils/logger";

setupConsoleRedirect();

createApp(App).mount("#app");
