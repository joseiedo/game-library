<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import type { Game } from "../types/game";

defineProps<{
  game: Game;
  focused: boolean;
}>();

const emit = defineEmits<{
  launch: [];
  focus: [];
  edit: [];
  delete: [];
}>();

const showMenu = ref(false);
const menuRef = ref<HTMLElement>();

function toggleMenu(e: Event) {
  e.stopPropagation();
  showMenu.value = !showMenu.value;
}

function handleEdit(e: Event) {
  e.stopPropagation();
  showMenu.value = false;
  emit("edit");
}

function handleDelete(e: Event) {
  e.stopPropagation();
  showMenu.value = false;
  emit("delete");
}

function handleClickOutside(e: MouseEvent) {
  if (showMenu.value && menuRef.value && !menuRef.value.contains(e.target as Node)) {
    showMenu.value = false;
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    emit("launch");
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});
</script>

<template>
  <div
    tabindex="0"
    role="button"
    :aria-label="`Launch ${game.title}`"
    class="group relative flex flex-col rounded-md overflow-hidden cursor-pointer select-none
           bg-zinc-900 border border-zinc-800 transition-all duration-150
           hover:border-zinc-600
           focus:outline-none"
    :class="focused ? 'ring-2 ring-zinc-500 border-zinc-600' : ''"
    @click="emit('launch')"
    @keydown="onKeyDown"
    @focus="emit('focus')"
  >
    <!-- Cover art -->
    <div class="aspect-[2/3] w-full bg-zinc-800 overflow-visible">
      <img
        v-if="game.coverImage"
        :src="game.coverImage"
        :alt="game.title"
        class="w-full h-full object-cover"
        loading="lazy"
      />
      <div
        v-else
        class="w-full h-full flex items-center justify-center text-zinc-700"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-10 h-10" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1"
            d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
        </svg>
      </div>

      <!-- Three-dot menu button (custom games only) -->
      <div
        v-if="game.platform === 'custom'"
        ref="menuRef"
        class="absolute top-1 right-1 z-20"
      >
        <button
          @click.stop="toggleMenu"
          class="p-1 rounded-md bg-zinc-900/80 hover:bg-zinc-800 text-zinc-400 hover:text-white
                 opacity-0 group-hover:opacity-100 focus:opacity-100 transition-all duration-150"
          :class="focused ? 'opacity-100' : ''"
          aria-label="Game options"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
          </svg>
        </button>

        <!-- Dropdown menu -->
        <div
          v-if="showMenu"
          class="absolute top-full right-0 mt-1 w-28 bg-zinc-800 rounded-md shadow-lg border border-zinc-700 py-1 z-10"
        >
          <button
            @click="handleEdit"
            class="w-full px-3 py-2 text-left text-sm text-white hover:bg-zinc-700 transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
            Edit
          </button>
          <button
            @click="handleDelete"
            class="w-full px-3 py-2 text-left text-sm text-red-400 hover:bg-zinc-700 transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Bottom bar -->
    <div class="px-2.5 py-2 flex flex-col gap-1 border-t border-zinc-800">
      <p class="text-white text-xs font-medium leading-tight line-clamp-2">{{ game.title }}</p>
      <span
        class="self-start text-[10px] font-medium px-1.5 py-0.5 rounded-sm text-zinc-400 bg-zinc-800"
      >
        {{ game.platform === "steam" ? "Steam" : game.platform === "epic" ? "Epic" : "Custom" }}
      </span>
    </div>

    <!-- Hover overlay with play icon -->
    <div
      class="absolute inset-0 flex items-center justify-center
             bg-black/0 group-hover:bg-black/50 transition-colors duration-150"
      :class="focused ? 'bg-black/50' : ''"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-10 h-10 text-white drop-shadow opacity-0 group-hover:opacity-100 transition-opacity duration-150"
        :class="focused ? 'opacity-100' : ''"
        viewBox="0 0 24 24"
        fill="currentColor"
      >
        <path d="M8 5v14l11-7z" />
      </svg>
    </div>
  </div>
</template>
