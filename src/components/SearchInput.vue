<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'search'): void;
  (e: 'keydown', event: KeyboardEvent): void;
  (e: 'blur'): void;
  (e: 'toggleInfo'): void;
}>();

const searchInput = ref<HTMLInputElement | null>(null);

function updateValue(event: Event) {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.value);
}

function handleKeyDown(event: KeyboardEvent) {
  emit('keydown', event);
}

function handleBlur() {
  emit('blur');
}

function toggleInfo() {
  emit('toggleInfo');
}

function focus() {
  searchInput.value?.focus();
}

defineExpose({
  focus
});
</script>

<template>
  <form @submit.prevent="$emit('search')" class="w-full">
    <div class="relative">
      <button
        type="submit"
        class="absolute left-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </button>
      <input
        ref="searchInput"
        :value="modelValue"
        @input="updateValue"
        type="text"
        placeholder="Search or type a URL"
        class="w-full px-4 py-3 pl-10 pr-10 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white shadow-lg"
        @keydown="handleKeyDown"
        @blur="handleBlur"
      />
      <button
        type="button"
        @click.prevent="toggleInfo"
        class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 focus:outline-none"
        title="Learn more"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </button>
    </div>
  </form>
</template>
