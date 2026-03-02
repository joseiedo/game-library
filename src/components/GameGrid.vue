<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import type { Game } from "../types/game";
import GameCard from "./GameCard.vue";

const props = defineProps<{
  games: Game[];
  focusedIndex: number;
}>();

const emit = defineEmits<{
  launch: [game: Game];
  edit: [game: Game];
  delete: [game: Game];
  "update:focusedIndex": [index: number];
}>();

const cardEls = ref<HTMLElement[]>([]);

function setCardRef(el: unknown, i: number) {
  if (el) cardEls.value[i] = (el as InstanceType<typeof GameCard>).$el as HTMLElement;
}

watch(() => props.focusedIndex, async (idx) => {
  await nextTick();
  cardEls.value[idx]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
});
</script>

<template>
  <div
    v-if="games.length > 0"
    class="grid gap-4"
    style="grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));"
  >
    <GameCard
      v-for="(game, i) in games"
      :key="game.key"
      :ref="(el) => setCardRef(el, i)"
      :game="game"
      :focused="i === props.focusedIndex"
      @launch="emit('launch', game)"
      @edit="emit('edit', game)"
      @delete="emit('delete', game)"
      @focus="emit('update:focusedIndex', i)"
    />
  </div>

  <div
    v-else
    class="flex flex-col items-center justify-center h-64 text-neutral-500 gap-3"
  >
    <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
        d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <p class="text-lg">No games found</p>
  </div>
</template>
