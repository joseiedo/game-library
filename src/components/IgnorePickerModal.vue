<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useGamepad } from "../composables/useGamepad";
import type { Game } from "../types/game";

const props = defineProps<{
  /** All games that are NOT already ignored — only these are shown. */
  games: Game[];
}>();

const emit = defineEmits<{
  ignore: [key: string];
  close: [];
}>();

const search = ref("");
const focusedIndex = ref(0);
const rowRefs = ref<HTMLElement[]>([]);

const filtered = computed(() => {
  const q = search.value.toLowerCase().trim();
  return q
    ? props.games.filter((g) => g.title.toLowerCase().includes(q))
    : props.games;
});

// Keep focus in bounds when the list shrinks (e.g. after ignoring an item).
watch(
  () => filtered.value.length,
  (len) => {
    if (focusedIndex.value >= len) {
      focusedIndex.value = Math.max(len - 1, 0);
    }
  },
);

// Scroll focused row into view.
watch(focusedIndex, (idx) => {
  nextTick(() => rowRefs.value[idx]?.scrollIntoView({ block: "nearest" }));
});

function ignoreGame() {
  const game = filtered.value[focusedIndex.value];
  if (game) emit("ignore", game.key);
}

const PLATFORM_LABEL: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
  custom: "Custom",
};

useGamepad((action) => {
  switch (action) {
    case "up":
      focusedIndex.value = Math.max(focusedIndex.value - 1, 0);
      break;
    case "down":
      focusedIndex.value = Math.min(focusedIndex.value + 1, filtered.value.length - 1);
      break;
    case "a":
      ignoreGame();
      break;
    case "b":
      emit("close");
      break;
  }
});
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div class="w-full max-w-md bg-white dark:bg-zinc-950 rounded-lg shadow-2xl border border-zinc-200 dark:border-zinc-800 flex flex-col transition-colors duration-200"
         style="max-height: 70vh">
      <!-- Header -->
      <div class="px-4 pt-4 pb-3 border-b border-zinc-100 dark:border-zinc-800 shrink-0">
        <h2 class="text-sm font-semibold text-zinc-900 dark:text-white mb-3">Add game to ignore list</h2>
        <div class="relative">
          <svg xmlns="http://www.w3.org/2000/svg"
               class="absolute left-2.5 top-2.5 w-3.5 h-3.5 text-zinc-400 dark:text-zinc-500 pointer-events-none"
               fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M21 21l-4.35-4.35M11 19a8 8 0 100-16 8 8 0 000 16z" />
          </svg>
          <input
            v-model="search"
            type="text"
            placeholder="Filter games…"
            class="w-full pl-8 pr-3 py-1.5 text-sm bg-zinc-50 text-zinc-900 placeholder-zinc-400 dark:bg-zinc-900 dark:text-white dark:placeholder-zinc-500
                   rounded-md border border-zinc-200 dark:border-zinc-800 focus:outline-none focus:border-zinc-400 dark:focus:border-zinc-600
                   focus:ring-1 focus:ring-zinc-400 dark:focus:ring-zinc-600 transition-colors"
            @keydown.esc.prevent="emit('close')"
          />
        </div>
      </div>

      <!-- Game list -->
      <div class="overflow-y-auto flex-1 p-2">
        <p v-if="filtered.length === 0" class="text-xs text-zinc-500 dark:text-zinc-500 px-2 py-4 text-center">
          No games found
        </p>

        <button
          v-for="(game, idx) in filtered"
          :key="game.key"
          :ref="(el) => { if (el) rowRefs[idx] = el as HTMLElement }"
          class="flex items-center justify-between w-full px-3 py-2 rounded-md text-sm transition-colors text-left"
          :class="focusedIndex === idx
            ? 'bg-zinc-100 dark:bg-zinc-800 ring-2 ring-zinc-400 dark:ring-zinc-500 text-zinc-900 dark:text-white'
            : 'text-zinc-600 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-white'"
          @click="focusedIndex = idx; ignoreGame()"
          @mouseenter="focusedIndex = idx"
        >
          <span class="truncate">{{ game.title }}</span>
          <span
            class="ml-3 shrink-0 text-xs px-1.5 py-0.5 rounded bg-zinc-100 dark:bg-zinc-700 text-zinc-500 dark:text-zinc-400"
          >{{ PLATFORM_LABEL[game.platform] ?? game.platform }}</span>
        </button>
      </div>

      <!-- Footer hint -->
      <div class="px-4 py-2.5 border-t border-zinc-100 dark:border-zinc-800 shrink-0 flex items-center justify-between">
        <span class="text-xs text-zinc-400 dark:text-zinc-600">
          <kbd class="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-mono">↑↓</kbd> navigate
          &nbsp;·&nbsp;
          <kbd class="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-mono">A</kbd> ignore
          &nbsp;·&nbsp;
          <kbd class="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-mono">B / Esc</kbd> close
        </span>
        <span class="text-xs text-zinc-400 dark:text-zinc-600">{{ filtered.length }} game{{ filtered.length !== 1 ? 's' : '' }}</span>
      </div>
    </div>
  </div>
</template>
