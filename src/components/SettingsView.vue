<script setup lang="ts">
export interface IgnoredGameInfo {
  key: string;
  title: string;
  platform: string;
}

defineProps<{
  autostartEnabled: boolean;
  ignoredGames: IgnoredGameInfo[];
  /**
   * Controller focus index across all settings rows:
   *   -1                      = no focus (sidebar is focused)
   *    0                      = autostart row
   *    1 .. ignoredGames.length = ignored-game rows
   *    ignoredGames.length + 1 = "Add to ignore list" button
   */
  focusedIndex: number;
}>();

const emit = defineEmits<{
  toggleAutostart: [];
  removeIgnored: [key: string];
  openIgnorePicker: [];
  checkForUpdates: [];
}>();

const PLATFORM_LABEL: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
  custom: "Custom",
};
</script>

<template>
  <div class="max-w-lg">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-sm font-medium text-zinc-400">Settings</h1>
    </div>

    <!-- General section -->
    <div class="flex flex-col gap-4 mb-8">
      <p class="text-xs text-zinc-500 font-medium uppercase tracking-wider">General</p>

      <!-- Autostart toggle -->
      <div
        class="flex items-center justify-between px-4 py-3 rounded-md bg-zinc-900 border transition-colors cursor-pointer"
        :class="focusedIndex === 0 ? 'border-zinc-500 ring-2 ring-zinc-500' : 'border-zinc-800'"
        @click="emit('toggleAutostart')"
      >
        <div>
          <p class="text-sm font-medium text-white">Launch at startup</p>
          <p class="text-xs text-zinc-500 mt-0.5">Start Game Library automatically when you log in</p>
        </div>

        <button
          class="relative inline-flex h-5 w-9 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none"
          :class="autostartEnabled ? 'bg-zinc-300' : 'bg-zinc-700'"
          tabindex="-1"
          @click.stop="emit('toggleAutostart')"
        >
          <span
            class="pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow-lg transform transition duration-200"
            :class="autostartEnabled ? 'translate-x-4' : 'translate-x-0'"
          />
        </button>
      </div>
    </div>

    <!-- Ignored games section -->
    <div class="flex flex-col gap-3">
      <p class="text-xs text-zinc-500 font-medium uppercase tracking-wider">Ignored Games</p>
      <p class="text-xs text-zinc-600 -mt-1">
        These games are hidden from your library.
      </p>

      <!-- List of currently ignored games -->
      <div v-if="ignoredGames.length > 0" class="flex flex-col gap-1.5">
        <div
          v-for="(game, idx) in ignoredGames"
          :key="game.key"
          class="flex items-center justify-between px-3 py-2 rounded-md bg-zinc-900 border transition-colors"
          :class="focusedIndex === idx + 1 ? 'border-zinc-500 ring-2 ring-zinc-500' : 'border-zinc-800'"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-400 shrink-0">
              {{ PLATFORM_LABEL[game.platform] ?? game.platform }}
            </span>
            <span class="text-sm text-zinc-300 truncate">{{ game.title }}</span>
          </div>
          <button
            class="ml-3 shrink-0 text-zinc-600 hover:text-zinc-300 transition-colors"
            tabindex="-1"
            title="Remove from ignore list"
            @click="emit('removeIgnored', game.key)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <p v-else class="text-xs text-zinc-600 italic">No games are currently ignored.</p>

      <!-- Add to ignore list button -->
      <button
        class="flex items-center gap-2 px-3 py-2 rounded-md border text-sm font-medium transition-colors w-fit"
        :class="focusedIndex === ignoredGames.length + 1
          ? 'border-zinc-500 ring-2 ring-zinc-500 text-zinc-200 bg-zinc-900'
          : 'border-zinc-700 text-zinc-400 hover:bg-zinc-900 hover:text-zinc-200'"
        @click="emit('openIgnorePicker')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add game to ignore list
      </button>
    </div>

    <!-- Updates section -->
    <div class="flex flex-col gap-3 mt-8">
      <p class="text-xs text-zinc-500 font-medium uppercase tracking-wider">Updates</p>

      <button
        class="flex items-center gap-2 px-3 py-2 rounded-md border text-sm font-medium transition-colors w-fit"
        :class="focusedIndex === ignoredGames.length + 2
          ? 'border-zinc-500 ring-2 ring-zinc-500 text-zinc-200 bg-zinc-900'
          : 'border-zinc-700 text-zinc-400 hover:bg-zinc-900 hover:text-zinc-200'"
        @click="emit('checkForUpdates')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        Check for updates
      </button>
    </div>

    <!-- Controller hint -->
    <p class="mt-10 text-xs text-zinc-600">
      Press <kbd class="px-1 py-0.5 rounded bg-zinc-800 text-zinc-400 font-mono">B</kbd>
      or <kbd class="px-1 py-0.5 rounded bg-zinc-800 text-zinc-400 font-mono">Esc</kbd>
      to return to the library
    </p>
  </div>
</template>
