<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { defineProps } from 'vue';

defineProps<{
  show: boolean;
  bangsCount: number;
  availableBangs: [string, string][];
}>();

const bangsExampleList = [
  {
    bang: "g",
    description: "Google"
  },
  {
    bang: "w",
    description: "Wikipedia"
  },
  {
    bang: "a",
    description: "Amazon"
  },
  {
    bang: "yt",
    description: "YouTube"
  },
  {
    bang: "gh",
    description: "GitHub"
  },
]

async function openUrl(url: string) {
  await invoke("open_url", { url: url });
}

</script>

<template>
  <div
    v-if="show"
    class="mt-2 px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg shadow-lg transition-opacity duration-300 max-h-[400px] overflow-y-auto"
  >
    <!-- Info header -->
    <div class="mb-3">
      <h3 class="text-lg font-medium text-gray-900 dark:text-white">End your search with a <a @click="openUrl('https://duckduckgo.com/bangs')" class="text-blue-500 dark:text-blue-400 hover:underline">bang!</a></h3>
      <p class="text-sm text-gray-600 dark:text-gray-400">
        Add <span class="font-mono bg-gray-100 dark:bg-gray-700 px-1 rounded">!</span> to send your search to a specific website.
      </p>
    </div>
    
    <!-- Example bangs -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
      <div 
        v-for="(bang, index) in bangsExampleList" 
        :key="index"
        class="flex items-center p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
      >
        <span class="font-mono font-bold text-blue-600 dark:text-blue-400 mr-2">!{{ bang.bang }}</span>
        <span class="text-gray-800 dark:text-gray-200">{{ bang.description }}</span>
      </div>
    </div>
    
    <!-- Footer text -->
    <div v-if="bangsCount > 20" class="mt-3 text-center text-sm text-gray-500 dark:text-gray-400">
      {{ bangsCount - 20 }} more bangs available. Type ! in the search box to see more.
    </div>
  </div>
</template> 