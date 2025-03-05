<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface SearchEngine {
  engine_name: string;
  engine_url: string;
}

const searchQuery = ref("");
const isSearching = ref(false);
const errorMessage = ref("");
const showSettings = ref(false);
const availableEngines = ref<SearchEngine[]>([]);
const currentEngine = ref<SearchEngine>({ engine_name: "Google", engine_url: "" });

// Load available search engines and current engine on mount
onMounted(async () => {
  try {
    availableEngines.value = await invoke<SearchEngine[]>("get_available_engines");
    currentEngine.value = await invoke<SearchEngine>("get_current_engine");
  } catch (error) {
    errorMessage.value = `Error loading search engines: ${error}`;
  }
});

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  isSearching.value = true;
  errorMessage.value = "";
  
  try {
    await invoke("search", { query: searchQuery.value });
    // Clear the search query after successful search
    searchQuery.value = "";
  } catch (error) {
    errorMessage.value = `Error: ${error}`;
  } finally {
    isSearching.value = false;
  }
}

async function changeSearchEngine(engineName: string) {
  try {
    currentEngine.value = await invoke<SearchEngine>("set_search_engine", { engineName });
    showSettings.value = false;
  } catch (error) {
    errorMessage.value = `Error changing search engine: ${error}`;
  }
}
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center bg-gray-100 dark:bg-gray-900 p-4">
    <div class="w-full max-w-md">
      <div class="flex justify-between items-center mb-8">
        <h1 class="text-3xl font-bold text-gray-800 dark:text-white">Haku</h1>
        <button 
          @click="showSettings = !showSettings" 
          class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
      
      <div v-if="showSettings" class="mb-6 p-4 bg-white dark:bg-gray-800 rounded-lg shadow">
        <h2 class="text-lg font-medium text-gray-800 dark:text-white mb-3">Search Engine</h2>
        <div class="space-y-2">
          <div 
            v-for="engine in availableEngines" 
            :key="engine.engine_name"
            class="flex items-center"
          >
            <button 
              @click="changeSearchEngine(engine.engine_name)"
              class="w-full text-left p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
              :class="{'bg-blue-50 dark:bg-blue-900/20': currentEngine.engine_name === engine.engine_name}"
            >
              <span class="mr-2" v-if="currentEngine.engine_name === engine.engine_name">✓</span>
              <span v-else class="mr-2 opacity-0">✓</span>
              {{ engine.engine_name }}
            </button>
          </div>
        </div>
      </div>
      
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
      
      <div v-if="errorMessage" class="mt-4 p-3 bg-red-100 text-red-700 rounded-lg">
        {{ errorMessage }}
      </div>
      
      <p class="mt-6 text-sm text-center text-gray-600 dark:text-gray-400">
        Press Enter to search with {{ currentEngine.engine_name }}
      </p>
    </div>
  </main>
</template>
