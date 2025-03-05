<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const appWindow = WebviewWindow.getCurrent();

appWindow.listen("tauri://focus", () => {
  searchQuery.value = "";
  searchInput.value?.focus();
});

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  await invoke("search", { query: searchQuery.value.trim() });
}
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center bg-gray-100 dark:bg-gray-900 p-4">
    <div class="w-full max-w-md">
      <form @submit.prevent="performSearch" class="w-full">
        <div class="relative">
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Search..."
            class="w-full px-4 py-3 pr-10 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
          />
          <button
            type="submit"
            class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
        </div>
      </form>
    </div>
  </main>
</template>
