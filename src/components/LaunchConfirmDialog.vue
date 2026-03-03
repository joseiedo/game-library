<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { info } from "@tauri-apps/plugin-log";
import type { Game } from "../types/game";
import { useGamepad } from "../composables/useGamepad";

const props = defineProps<{ game: Game }>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    info(`Launch confirmed via keyboard: ${props.game.title}`);
    emit("confirm");
  } else if (e.key === "Escape") {
    e.preventDefault();
    info(`Launch cancelled via keyboard: ${props.game.title}`);
    emit("cancel");
  }
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    info(`Launch cancelled via backdrop click: ${props.game.title}`);
    emit("cancel");
  }
}

useGamepad((action) => {
  if (action === "a") emit("confirm");
  else if (action === "b") emit("cancel");
});

onMounted(() => window.addEventListener("keydown", onKeyDown));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown));
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    @click="onBackdropClick"
  >
    <div class="flex flex-col items-center gap-5 bg-white dark:bg-zinc-950 rounded-lg border border-zinc-200 dark:border-zinc-800
                shadow-2xl p-6 w-64 transition-colors duration-200">

      <!-- Cover art -->
      <div class="w-28 rounded-md overflow-hidden aspect-[2/3] bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 shrink-0">
        <img
          v-if="game.coverImage"
          :src="game.coverImage"
          :alt="game.title"
          class="w-full h-full object-cover"
        />
        <div v-else class="w-full h-full flex items-center justify-center text-zinc-400 dark:text-zinc-700">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1"
              d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
          </svg>
        </div>
      </div>

      <!-- Title -->
      <div class="flex flex-col items-center gap-1.5 text-center">
        <p class="text-zinc-900 dark:text-white font-semibold text-sm leading-tight">{{ game.title }}</p>
        <span class="text-[10px] font-medium px-1.5 py-0.5 rounded-sm bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400">
          {{ game.platform === "steam" ? "Steam" : "Custom" }}
        </span>
      </div>

      <!-- Actions -->
      <div class="flex gap-2 w-full">
        <button
          @click="info(`Launch cancelled via button: ${game.title}`); emit('cancel')"
          class="flex-1 py-1.5 text-sm rounded-md border border-zinc-200 dark:border-zinc-700
                 text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 hover:text-zinc-900 dark:hover:text-white transition-colors"
        >
          Cancel
        </button>
        <button
          autofocus
          @click="info(`Launch confirmed via button: ${game.title}`); emit('confirm')"
          class="flex-1 py-1.5 text-sm font-medium rounded-md
                 bg-zinc-900 text-white dark:bg-white dark:text-zinc-950 hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
        >
          Launch
        </button>
      </div>

    </div>
  </div>
</template>
