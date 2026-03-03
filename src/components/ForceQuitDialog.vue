<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import type { Game } from "../types/game";

defineProps<{ game: Game }>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    emit("confirm");
  } else if (e.key === "Escape") {
    e.preventDefault();
    emit("cancel");
  }
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) emit("cancel");
}

onMounted(() => window.addEventListener("keydown", onKeyDown));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown));
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    @click="onBackdropClick"
  >
    <div class="flex flex-col items-center gap-5 bg-white dark:bg-zinc-950 rounded-2xl border border-zinc-200 dark:border-zinc-800
                shadow-2xl p-8 w-72 transition-colors duration-200">

      <!-- Cover art with running overlay -->
      <div class="relative w-32 rounded-lg overflow-hidden shadow-lg aspect-[2/3] bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 shrink-0">
        <img
          v-if="game.coverImage"
          :src="game.coverImage"
          :alt="game.title"
          class="w-full h-full object-cover"
        />
        <div v-else class="w-full h-full flex items-center justify-center text-zinc-400 dark:text-zinc-700">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-10 h-10" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
          </svg>
        </div>

        <!-- Running badge on cover -->
        <div class="absolute top-2 left-1/2 -translate-x-1/2 flex items-center gap-1.5
                    bg-black/70 rounded-full px-2 py-0.5">
          <span class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
          <span class="text-green-400 text-xs font-semibold leading-none whitespace-nowrap">Running</span>
        </div>
      </div>

      <!-- Title + badge -->
      <div class="flex flex-col items-center gap-2 text-center">
        <p class="text-zinc-900 dark:text-white font-bold text-lg leading-tight">{{ game.title }}</p>
        <span
          class="text-xs font-semibold px-2 py-0.5 rounded shadow-sm"
          :class="game.platform === 'steam' ? 'bg-sky-600 text-white' : 'bg-violet-600 text-white'"
        >
          {{ game.platform === "steam" ? "Steam" : "Custom" }}
        </span>
      </div>

      <p class="text-zinc-500 dark:text-zinc-400 text-sm text-center -mt-1">
        Force quit this game? Unsaved progress may be lost.
      </p>

      <!-- Actions -->
      <div class="flex gap-3 w-full">
        <button
          autofocus
          @click="emit('cancel')"
          class="flex-1 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700
                 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:border-zinc-400 dark:hover:border-zinc-500 transition-colors"
        >
          Dismiss
        </button>
        <button
          @click="emit('confirm')"
          class="flex-1 py-2 text-sm font-semibold rounded-lg
                 bg-red-600 hover:bg-red-500 text-white transition-colors shadow-md shadow-red-900/10"
        >
          Force Quit
        </button>
      </div>

    </div>
  </div>
</template>
