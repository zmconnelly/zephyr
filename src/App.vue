<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface SearchEngine {
  engine_name: string;
  engine_url: string;
}

const searchQuery = ref("");
const isSearching = ref(false);
const availableEngines = ref<SearchEngine[]>([]);
const currentEngine = ref<SearchEngine>({ engine_name: "Google", engine_url: "" });

// Load available search engines and current engine on mount
onMounted(async () => {
    availableEngines.value = await invoke<SearchEngine[]>("get_available_engines");
    currentEngine.value = await invoke<SearchEngine>("get_current_engine");
});

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  isSearching.value = true;
    await invoke("search", { query: searchQuery.value });
    isSearching.value = false;
}
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center bg-gray-100 dark:bg-gray-900 p-4">
    <div class="w-full max-w-md">

      <form @submit.prevent="performSearch" class="w-full">
        <div class="relative">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="`Search with ${currentEngine.engine_name}...`"
            class="w-full px-4 py-3 pr-10 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
            :disabled="isSearching"
            autofocus
          />
          <button
            type="submit"
            class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400"
            :disabled="isSearching"
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
