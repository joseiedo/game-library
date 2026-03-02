<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { info, warn, error as logError } from "@tauri-apps/plugin-log";
import Sidebar from "./components/Sidebar.vue";
import GameGrid from "./components/GameGrid.vue";
import AddGameModal from "./components/AddGameModal.vue";
import LaunchConfirmDialog from "./components/LaunchConfirmDialog.vue";
import VirtualKeyboard from "./components/VirtualKeyboard.vue";
import { useGamepad, type GamepadAction } from "./composables/useGamepad";
import {
  fromSteamGame,
  fromCustomGame,
  fromEpicGame,
  type Game,
  type CustomGame,
  type EpicGame,
  type SteamGame,
  type PlatformFilter,
  type SortOption,
} from "./types/game";

// ── State ──────────────────────────────────────────────────────────────────

const allGames = ref<Game[]>([]);
const customGamesData = ref<CustomGame[]>([]);
const loading = ref(true);
const loadError = ref("");
const showAddModal = ref(false);
const showEditModal = ref(false);
const editingGame = ref<CustomGame | null>(null);
const pendingLaunch = ref<Game | null>(null);
const pendingDelete = ref<CustomGame | null>(null);
const focusedIndex = ref(0);
const notification = ref<{ message: string; type: "error" | "info" } | null>(null);
let notificationTimer = 0;

function showNotification(message: string, type: "error" | "info" = "error") {
  clearTimeout(notificationTimer);
  notification.value = { message, type };
  notificationTimer = window.setTimeout(() => { notification.value = null; }, 5000);
}

const search = ref("");
const platformFilter = ref<PlatformFilter>("all");
const sortOption = ref<SortOption>("alpha");

// ── Sidebar focus state ────────────────────────────────────────────────────

const focusArea = ref<"grid" | "sidebar">("grid");
const sidebarFocusedIndex = ref(0);
const sidebarInputActive = ref(false);
const sidebarRef = ref<InstanceType<typeof Sidebar>>();
const searchKeyboardOpen = ref(false);

// Matches the visual top-to-bottom order in Sidebar.vue
const SIDEBAR_ITEMS = [
  "search",
  "filter-all",
  "filter-steam",
  "filter-epic",
  "filter-custom",
  "sort",
  "add-game",
] as const;
type SidebarItem = (typeof SIDEBAR_ITEMS)[number];

// ── Data loading ───────────────────────────────────────────────────────────

async function loadGames() {
  loading.value = true;
  loadError.value = "";
  info("Loading game library...");
  try {
    const [steamGames, epicGames, customGames] = await Promise.all([
      invoke<SteamGame[]>("get_steam_games").catch((e) => {
        warn(`Steam game discovery failed: ${e}`);
        return [] as SteamGame[];
      }),
      invoke<EpicGame[]>("get_epic_games").catch((e) => {
        warn(`Epic game discovery failed: ${e}`);
        return [] as EpicGame[];
      }),
      invoke<CustomGame[]>("get_custom_games"),
    ]);

    allGames.value = [
      ...steamGames.map(fromSteamGame),
      ...epicGames.map(fromEpicGame),
      ...customGames.map(fromCustomGame),
    ];
    customGamesData.value = customGames;
    info(`Library loaded: ${steamGames.length} Steam game(s), ${epicGames.length} Epic game(s), ${customGames.length} custom game(s)`);
  } catch (e) {
    logError(`Failed to load library: ${e}`);
    loadError.value = String(e);
  } finally {
    loading.value = false;
  }
}

// ── Filtered & sorted view ─────────────────────────────────────────────────

const steamCount = computed(() => allGames.value.filter((g) => g.platform === "steam").length);
const epicCount = computed(() => allGames.value.filter((g) => g.platform === "epic").length);
const customCount = computed(() => allGames.value.filter((g) => g.platform === "custom").length);

const filteredGames = computed<Game[]>(() => {
  let result = allGames.value;

  if (platformFilter.value !== "all") {
    result = result.filter((g) => g.platform === platformFilter.value);
  }

  if (search.value.trim()) {
    const q = search.value.toLowerCase();
    result = result.filter(
      (g) =>
        g.title.toLowerCase().includes(q) ||
        g.tags.some((t) => t.toLowerCase().includes(q))
    );
  }

  if (sortOption.value === "alpha") {
    result = [...result].sort((a, b) => a.title.localeCompare(b.title));
  }

  return result;
});

// ── Launch ─────────────────────────────────────────────────────────────────

function requestLaunch(game: Game) {
  info(`Launch requested: "${game.title}" [${game.platform}]`);
  pendingLaunch.value = game;
}

async function confirmLaunch() {
  const game = pendingLaunch.value;
  if (!game) return;
  pendingLaunch.value = null;
  info(`Launching: "${game.title}" [${game.platform}]`);
  try {
    await invoke("launch_game", {
      key: game.key,
      appId: game.appId ?? null,
      isShortcut: game.isShortcut ?? null,
      executable: game.executable ?? null,
      epicLaunchUri: game.epicLaunchUri ?? null,
    });
  } catch (e) {
    logError(`Failed to launch "${game.title}": ${e}`);
    showNotification(String(e));
  }
}

function cancelLaunch() {
  if (pendingLaunch.value) {
    info(`Launch cancelled: "${pendingLaunch.value.title}"`);
  }
  pendingLaunch.value = null;
}

// ── Add game ───────────────────────────────────────────────────────────────

function onGameAdded(custom: CustomGame) {
  info(`Custom game added: "${custom.title}" (id=${custom.id})`);
  allGames.value.push(fromCustomGame(custom));
  customGamesData.value.push(custom);
  showAddModal.value = false;
}

function onGameUpdated(custom: CustomGame) {
  info(`Custom game updated: "${custom.title}" (id=${custom.id})`);
  const index = allGames.value.findIndex((g) => g.key === `custom-${custom.id}`);
  if (index !== -1) {
    allGames.value[index] = fromCustomGame(custom);
  }
  const dataIndex = customGamesData.value.findIndex((g) => g.id === custom.id);
  if (dataIndex !== -1) {
    customGamesData.value[dataIndex] = custom;
  }
  showEditModal.value = false;
  editingGame.value = null;
}

function requestEditGame(game: Game) {
  if (game.platform !== "custom") return;
  const id = game.key.replace("custom-", "");
  const customData = customGamesData.value.find((g) => g.id === id);
  if (customData) {
    editingGame.value = customData;
    showEditModal.value = true;
  }
}

function requestDeleteGame(game: Game) {
  if (game.platform !== "custom") return;
  const id = game.key.replace("custom-", "");
  const customData = customGamesData.value.find((g) => g.id === id);
  if (customData) {
    pendingDelete.value = customData;
  }
}

function confirmDeleteGame() {
  if (pendingDelete.value) {
    invoke("remove_game", { id: pendingDelete.value.id })
      .then(() => {
        info(`Custom game deleted: id=${pendingDelete.value?.id}`);
        allGames.value = allGames.value.filter(
          (g) => g.key !== `custom-${pendingDelete.value?.id}`
        );
        customGamesData.value = customGamesData.value.filter(
          (g) => g.id !== pendingDelete.value?.id
        );
        showEditModal.value = false;
        editingGame.value = null;
      })
      .catch((e) => {
        logError(`Failed to delete game: ${e}`);
        showNotification(`Failed to delete game: ${e}`);
      })
      .finally(() => {
        pendingDelete.value = null;
      });
  }
}

// ── Sidebar navigation ─────────────────────────────────────────────────────

function activateSidebarItem() {
  const item: SidebarItem = SIDEBAR_ITEMS[sidebarFocusedIndex.value];
  switch (item) {
    case "search":
      searchKeyboardOpen.value = true;
      break;
    case "filter-all":
      platformFilter.value = "all";
      focusedIndex.value = 0;
      break;
    case "filter-steam":
      platformFilter.value = "steam";
      focusedIndex.value = 0;
      break;
    case "filter-epic":
      platformFilter.value = "epic";
      focusedIndex.value = 0;
      break;
    case "filter-custom":
      platformFilter.value = "custom";
      focusedIndex.value = 0;
      break;
    case "sort":
      sortOption.value = sortOption.value === "alpha" ? "recentlyAdded" : "alpha";
      break;
    case "add-game":
      showAddModal.value = true;
      break;
  }
}

function navigateSidebar(action: GamepadAction) {
  if (sidebarInputActive.value) {
    if (action === "b") {
      sidebarRef.value?.blurActive();
      sidebarInputActive.value = false;
    }
    return;
  }
  switch (action) {
    case "up":
      sidebarFocusedIndex.value = Math.max(sidebarFocusedIndex.value - 1, 0);
      break;
    case "down":
      sidebarFocusedIndex.value = Math.min(sidebarFocusedIndex.value + 1, SIDEBAR_ITEMS.length - 1);
      break;
    case "right":
      focusArea.value = "grid";
      break;
    case "a":
      activateSidebarItem();
      break;
    case "b":
      focusArea.value = "grid";
      break;
  }
}

function navigateGrid(action: GamepadAction) {
  const count = filteredGames.value.length;
  // Match CSS: repeat(auto-fill, minmax(150px, 1fr)) with gap-4 (16px).
  // Container has px-6 padding (48px total), so grid content width = clientWidth - 48.
  // Cols = floor((contentWidth + gap) / (minCard + gap)) = floor((clientWidth - 32) / 166)
  const areaWidth = document.getElementById("game-grid-area")?.clientWidth ?? 848;
  const cols = Math.max(1, Math.floor((areaWidth - 32) / 166));
  if (count > 0) {
    switch (action) {
      case "left":
        if (focusedIndex.value % cols === 0) {
          focusArea.value = "sidebar";
        } else {
          focusedIndex.value = Math.max(focusedIndex.value - 1, 0);
        }
        break;
      case "right":
        focusedIndex.value = Math.min(focusedIndex.value + 1, count - 1);
        break;
      case "down":
        focusedIndex.value = Math.min(focusedIndex.value + cols, count - 1);
        break;
      case "up":
        focusedIndex.value = Math.max(focusedIndex.value - cols, 0);
        break;
      case "a":
        if (filteredGames.value[focusedIndex.value]) {
          requestLaunch(filteredGames.value[focusedIndex.value]);
        }
        break;
    }
  } else if (action === "left") {
    focusArea.value = "sidebar";
  }
}

// ── Keyboard navigation ────────────────────────────────────────────────────

function onKeyDown(e: KeyboardEvent) {
  if (pendingLaunch.value) return;
  if (showAddModal.value) return;

  if (focusArea.value === "sidebar") {
    if (sidebarInputActive.value) {
      if (e.key === "Escape") {
        sidebarRef.value?.blurActive();
        sidebarInputActive.value = false;
      }
      return;
    }
    switch (e.key) {
      case "ArrowDown":  e.preventDefault(); navigateSidebar("down");  break;
      case "ArrowUp":    e.preventDefault(); navigateSidebar("up");    break;
      case "ArrowRight": e.preventDefault(); navigateSidebar("right"); break;
      case "Enter":      e.preventDefault(); navigateSidebar("a");     break;
      case "Escape":     navigateSidebar("b"); break;
    }
    return;
  }

  // Grid navigation
  switch (e.key) {
    case "ArrowLeft":  e.preventDefault(); navigateGrid("left");  break;
    case "ArrowRight": e.preventDefault(); navigateGrid("right"); break;
    case "ArrowDown":  e.preventDefault(); navigateGrid("down");  break;
    case "ArrowUp":    e.preventDefault(); navigateGrid("up");    break;
    case "Enter":      e.preventDefault(); navigateGrid("a");     break;
  }
}

// ── Gamepad navigation ─────────────────────────────────────────────────────

const gamepadEnabled = computed(
  () =>
    !showAddModal.value &&
    !searchKeyboardOpen.value &&
    pendingLaunch.value === null
);

useGamepad((action) => {
  if (focusArea.value === "sidebar") {
    navigateSidebar(action);
  } else {
    navigateGrid(action);
  }
}, { enabled: gamepadEnabled });

// ── Lifecycle ──────────────────────────────────────────────────────────────

onMounted(() => {
  loadGames();
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div class="flex h-screen bg-zinc-950 text-white overflow-hidden">
    <Sidebar
      ref="sidebarRef"
      :search="search"
      :platform-filter="platformFilter"
      :sort-option="sortOption"
      :total-games="allGames.length"
      :steam-count="steamCount"
      :epic-count="epicCount"
      :custom-count="customCount"
      :sidebar-focused-index="focusArea === 'sidebar' ? sidebarFocusedIndex : -1"
      @update:search="search = $event; focusedIndex = 0"
      @update:platform-filter="platformFilter = $event; focusedIndex = 0"
      @update:sort-option="sortOption = $event"
      @add-game="showAddModal = true"
      @input-blur="sidebarInputActive = false"
    />

    <main id="game-grid-area" class="flex-1 overflow-y-auto px-6 py-6 relative">
      <!-- Notification banner -->
      <Transition name="slide-down">
        <div
          v-if="notification"
          class="flex items-start gap-3 mb-4 px-4 py-3 rounded-md text-sm border"
          :class="notification.type === 'error'
            ? 'bg-red-950/50 border-red-900 text-red-300'
            : 'bg-zinc-900 border-zinc-700 text-zinc-300'"
        >
          <span class="flex-1">{{ notification.message }}</span>
          <button @click="notification = null" class="text-zinc-500 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </Transition>

      <!-- Loading -->
      <div v-if="loading" class="flex items-center justify-center h-full">
        <svg class="animate-spin w-6 h-6 text-zinc-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
        </svg>
      </div>

      <!-- Error -->
      <div v-else-if="loadError" class="flex flex-col items-center justify-center h-full gap-3 text-zinc-400">
        <p class="text-sm font-medium text-white">Failed to load library</p>
        <p class="text-sm text-zinc-500">{{ loadError }}</p>
        <button
          @click="loadGames"
          class="mt-2 px-4 py-2 text-sm rounded-md border border-zinc-700 hover:bg-zinc-800 transition-colors"
        >
          Retry
        </button>
      </div>

      <!-- Library header + grid -->
      <template v-else>
        <div class="flex items-center justify-between mb-6">
          <h1 class="text-sm font-medium text-zinc-400">
            {{ platformFilter === "all" ? "All Games" : platformFilter === "steam" ? "Steam" : platformFilter === "epic" ? "Epic" : "Custom" }}
          </h1>
          <span class="text-xs text-zinc-600">{{ filteredGames.length }} game{{ filteredGames.length !== 1 ? "s" : "" }}</span>
        </div>

        <GameGrid
          :games="filteredGames"
          :focused-index="focusedIndex"
          @launch="requestLaunch"
          @edit="requestEditGame"
          @delete="requestDeleteGame"
          @update:focused-index="focusedIndex = $event"
        />
      </template>
    </main>

    <AddGameModal
      v-if="showAddModal"
      mode="add"
      @close="showAddModal = false"
      @added="onGameAdded"
    />

    <AddGameModal
      v-if="showEditModal"
      mode="edit"
      :edit-data="editingGame"
      @close="showEditModal = false; editingGame = null"
      @updated="onGameUpdated"
      @request-delete="pendingDelete = editingGame"
    />

    <LaunchConfirmDialog
      v-if="pendingLaunch"
      :game="pendingLaunch"
      @confirm="confirmLaunch"
      @cancel="cancelLaunch"
    />

    <!-- Delete Confirmation Dialog -->
    <div
      v-if="pendingDelete"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      @click.self="pendingDelete = null"
    >
      <div class="w-full max-w-sm bg-zinc-950 rounded-lg shadow-2xl border border-zinc-800 p-5">
        <h3 class="text-white text-sm font-semibold mb-2">Delete Game?</h3>
        <p class="text-zinc-400 text-sm mb-4">
          Are you sure you want to delete "{{ pendingDelete?.title }}"?
          This action cannot be undone.
        </p>
        <div class="flex justify-end gap-2">
          <button
            @click="pendingDelete = null"
            class="px-4 py-1.5 text-sm text-zinc-400 hover:text-white rounded-md
                   border border-zinc-700 hover:bg-zinc-800 transition-colors"
          >
            Cancel
          </button>
          <button
            @click="confirmDeleteGame"
            class="px-4 py-1.5 text-sm font-medium bg-red-600 text-white
                   rounded-md hover:bg-red-500 transition-colors"
          >
            Delete
          </button>
        </div>
      </div>
    </div>

    <VirtualKeyboard
      v-if="searchKeyboardOpen"
      :model-value="search"
      @update:model-value="search = $event; focusedIndex = 0"
      @confirm="searchKeyboardOpen = false"
    />
  </div>
</template>

<style>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
