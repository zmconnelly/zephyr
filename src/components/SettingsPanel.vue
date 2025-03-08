<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const runAtStartup = ref(false);
const isLoading = ref(true);

onMounted(async () => {
  await loadStartupStatus();
});

async function loadStartupStatus() {
  try {
    isLoading.value = true;
    const result = await invoke('get_startup_status');
    runAtStartup.value = (result as any).enabled;
  } catch (error) {
    console.error('Failed to get startup status:', error);
  } finally {
    isLoading.value = false;
  }
}

async function toggleRunAtStartup() {
  try {
    isLoading.value = true;
    const result = await invoke('toggle_run_at_startup', {
      enable: !runAtStartup.value
    });
    runAtStartup.value = (result as any).enabled;
  } catch (error) {
    console.error('Failed to toggle startup setting:', error);
  } finally {
    isLoading.value = false;
  }
}

async function checkForUpdates() {
  try {
    const result = await invoke('update');
    console.log('Update check result:', result);
  } catch (error) {
    console.error('Failed to check for updates:', error);
  }
}

</script>

<template>
  <div 
    v-if="show" 
    class="settings-panel bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 mt-2 w-full"
  >
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-white">Settings</h2>
      <button 
        @click="emit('close')" 
        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
    
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <span class="text-gray-700 dark:text-gray-300">Run at startup</span>
        <div class="relative inline-block w-10 mr-2 align-middle select-none">
          <input 
            type="checkbox" 
            :checked="runAtStartup" 
            @change="toggleRunAtStartup" 
            :disabled="isLoading"
            id="toggle-startup" 
            class="absolute block w-6 h-6 rounded-full bg-white border-4 border-gray-300 appearance-none cursor-pointer transition-all duration-300 ease-in-out checked:right-0 checked:border-indigo-500 right-0"
          />
          <label 
            for="toggle-startup" 
            class="block overflow-hidden h-6 rounded-full bg-gray-300 cursor-pointer transition-all duration-300 ease-in-out"
            :class="{ 'bg-indigo-500': runAtStartup }"
          ></label>
        </div>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-gray-700 dark:text-gray-300">Check for updates</span>
        <button @click="checkForUpdates" :disabled="isLoading" class="bg-indigo-500 text-white px-4 py-2 rounded-md hover:bg-indigo-600 transition-colors duration-300 ease-in-out">
          {{ isLoading ? 'Checking...' : 'Check' }}
        </button>
      </div>

      <!-- Add more settings here as needed -->
    </div>
  </div>
</template>