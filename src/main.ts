import { createApp } from "vue";
import App from "./App.vue";
import { setupConsoleRedirection } from "./utils/logger";

// Set up console redirection to Rust backend
setupConsoleRedirection();

// Log a test message
console.log("Vue app starting - logs will appear in terminal");

createApp(App).mount("#app");
