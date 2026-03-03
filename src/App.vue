<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { info, warn, error as logError } from "@tauri-apps/plugin-log";
import { getVersion } from "@tauri-apps/api/app";
import Sidebar from "./components/Sidebar.vue";
import GameGrid from "./components/GameGrid.vue";
import SettingsView from "./components/SettingsView.vue";
import IgnorePickerModal from "./components/IgnorePickerModal.vue";
import AddGameModal from "./components/AddGameModal.vue";
import LaunchConfirmDialog from "./components/LaunchConfirmDialog.vue";
import VirtualKeyboard from "./components/VirtualKeyboard.vue";
import UpdateAvailableModal from "./components/UpdateAvailableModal.vue";
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

const showUpdateModal = ref(false);
const updateInfo = ref<{ version: string; assetUrl: string; releaseUrl: string } | null>(null);
const checkingForUpdates = ref(false);

const activeTab = ref<"library" | "settings">("library");
const autostartEnabled = ref(false);
const ignoredKeys = ref<Set<string>>(new Set());
const showIgnorePicker = ref(false);

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

const focusArea = ref<"grid" | "sidebar" | "settings">("grid");
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
  "settings",
] as const;
type SidebarItem = (typeof SIDEBAR_ITEMS)[number];

const settingsFocusedIndex = ref(0);

// Ignored games that still exist in allGames (shown in settings list).
const ignoredGamesInfo = computed(() =>
  allGames.value
    .filter((g) => ignoredKeys.value.has(g.key))
    .map((g) => ({ key: g.key, title: g.title, platform: g.platform as string })),
);

// Games the user can still add to the ignore list.
const availableForIgnore = computed(() =>
  allGames.value.filter((g) => !ignoredKeys.value.has(g.key)),
);

// Settings rows: 0=autostart, 1..n=ignored game rows, n+1=add button, n+2=check-for-updates
const settingsItemCount = computed(() => 3 + ignoredGamesInfo.value.length);

// Keep focus in bounds when ignored-games list changes length.
watch(settingsItemCount, (count) => {
  if (settingsFocusedIndex.value >= count) {
    settingsFocusedIndex.value = Math.max(count - 1, 0);
  }
});

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

// ── Autostart ──────────────────────────────────────────────────────────────

async function loadAutostart() {
  try {
    autostartEnabled.value = await invoke<boolean>("get_autostart");
  } catch (e) {
    warn(`Could not read autostart state: ${e}`);
  }
}

async function toggleAutostart() {
  const next = !autostartEnabled.value;
  try {
    await invoke("set_autostart", { enabled: next });
    autostartEnabled.value = next;
    info(`Autostart ${next ? "enabled" : "disabled"}`);
  } catch (e) {
    logError(`Failed to set autostart: ${e}`);
    showNotification(String(e));
  }
}

// ── Ignored games ──────────────────────────────────────────────────────────

async function loadIgnoredGames() {
  try {
    const keys = await invoke<string[]>("get_ignored_games");
    ignoredKeys.value = new Set(keys);
    info(`Ignored games loaded: ${keys.length} key(s)`);
  } catch (e) {
    warn(`Could not load ignored games: ${e}`);
  }
}

async function addIgnoredGame(key: string) {
  try {
    await invoke("add_ignored_game", { key });
    ignoredKeys.value = new Set([...ignoredKeys.value, key]);
    info(`Game ignored: ${key}`);
  } catch (e) {
    logError(`Failed to ignore game: ${e}`);
    showNotification(String(e));
  }
}

async function removeIgnoredGame(key: string) {
  try {
    await invoke("remove_ignored_game", { key });
    const next = new Set(ignoredKeys.value);
    next.delete(key);
    ignoredKeys.value = next;
    info(`Game unignored: ${key}`);
  } catch (e) {
    logError(`Failed to unignore game: ${e}`);
    showNotification(String(e));
  }
}

// ── Filtered & sorted view ─────────────────────────────────────────────────

const visibleGames = computed(() => allGames.value.filter((g) => !ignoredKeys.value.has(g.key)));
const steamCount = computed(() => visibleGames.value.filter((g) => g.platform === "steam").length);
const epicCount = computed(() => visibleGames.value.filter((g) => g.platform === "epic").length);
const customCount = computed(() => visibleGames.value.filter((g) => g.platform === "custom").length);

const filteredGames = computed<Game[]>(() => {
  let result = visibleGames.value;

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
      switchToLibrary();
      searchKeyboardOpen.value = true;
      break;
    case "filter-all":
      switchToLibrary();
      platformFilter.value = "all";
      focusedIndex.value = 0;
      break;
    case "filter-steam":
      switchToLibrary();
      platformFilter.value = "steam";
      focusedIndex.value = 0;
      break;
    case "filter-epic":
      switchToLibrary();
      platformFilter.value = "epic";
      focusedIndex.value = 0;
      break;
    case "filter-custom":
      switchToLibrary();
      platformFilter.value = "custom";
      focusedIndex.value = 0;
      break;
    case "sort":
      switchToLibrary();
      sortOption.value = sortOption.value === "alpha" ? "recentlyAdded" : "alpha";
      break;
    case "add-game":
      showAddModal.value = true;
      break;
    case "settings":
      openSettings();
      break;
  }
}

function navigateSettings(action: GamepadAction) {
  const itemCount = settingsItemCount.value;
  switch (action) {
    case "up":
      settingsFocusedIndex.value = Math.max(settingsFocusedIndex.value - 1, 0);
      break;
    case "down":
      settingsFocusedIndex.value = Math.min(settingsFocusedIndex.value + 1, itemCount - 1);
      break;
    case "a":
      if (settingsFocusedIndex.value === 0) {
        toggleAutostart();
      } else if (settingsFocusedIndex.value === itemCount - 2) {
        showIgnorePicker.value = true;
      } else if (settingsFocusedIndex.value === itemCount - 1) {
        checkForUpdates();
      } else {
        // Ignored game row — unignore it
        const game = ignoredGamesInfo.value[settingsFocusedIndex.value - 1];
        if (game) removeIgnoredGame(game.key);
      }
      break;
    case "b":
      activeTab.value = "library";
      focusArea.value = "grid";
      break;
    case "left":
      activeTab.value = "library";
      focusArea.value = "sidebar";
      sidebarFocusedIndex.value = SIDEBAR_ITEMS.indexOf("settings");
      break;
  }
}

function openSettings() {
  activeTab.value = "settings";
  focusArea.value = "settings";
  settingsFocusedIndex.value = 0;
  sidebarFocusedIndex.value = SIDEBAR_ITEMS.indexOf("settings");
}

function switchToLibrary() {
  activeTab.value = "library";
  focusArea.value = "grid";
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
      focusArea.value = activeTab.value === "settings" ? "settings" : "grid";
      break;
    case "a":
      activateSidebarItem();
      break;
    case "b":
      focusArea.value = activeTab.value === "settings" ? "settings" : "grid";
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

  if (focusArea.value === "settings") {
    switch (e.key) {
      case "ArrowDown":  e.preventDefault(); navigateSettings("down");  break;
      case "ArrowUp":    e.preventDefault(); navigateSettings("up");    break;
      case "ArrowLeft":  e.preventDefault(); navigateSettings("left");  break;
      case "Enter":      e.preventDefault(); navigateSettings("a");     break;
      case "Escape":     navigateSettings("b"); break;
    }
    return;
  }

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
    !showUpdateModal.value &&
    !showAddModal.value &&
    !searchKeyboardOpen.value &&
    !showIgnorePicker.value &&
    pendingLaunch.value === null
);

useGamepad((action) => {
  if (focusArea.value === "sidebar") {
    navigateSidebar(action);
  } else if (focusArea.value === "settings") {
    navigateSettings(action);
  } else {
    navigateGrid(action);
  }
}, { enabled: gamepadEnabled });

// ── Update check ───────────────────────────────────────────────────────────

function findAssetUrl(
  assets: Array<{ name: string; browser_download_url: string }>,
  os: string,
  cpuArch: string,
): string | null {
  const candidates: string[] = [];
  if (os === "macos") {
    candidates.push(cpuArch === "aarch64" ? "_aarch64.dmg" : "_x86_64.dmg", "_universal.dmg");
  } else if (os === "windows") {
    candidates.push("_x64-setup.exe", "_x64-setup.msi");
  } else if (os === "linux") {
    candidates.push("_amd64.AppImage");
  }
  for (const suffix of candidates) {
    const asset = assets.find((a) => a.name.includes(suffix));
    if (asset) return asset.browser_download_url;
  }
  return null;
}

function isNewerVersion(remote: string, current: string): boolean {
  const parse = (v: string) => v.split(".").map(Number);
  const [ra, rb, rc = 0] = parse(remote);
  const [ca, cb, cc = 0] = parse(current);
  return ra !== ca ? ra > ca : rb !== cb ? rb > cb : rc > cc;
}

function detectOS(): { os: string; arch: string } {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return { os: "windows", arch: "x86_64" };
  if (ua.includes("mac")) return { os: "macos", arch: "aarch64" };
  return { os: "linux", arch: "x86_64" };
}

async function checkForUpdates(silent = false) {
  if (checkingForUpdates.value) return;
  checkingForUpdates.value = true;
  try {
    const currentVersion = await getVersion();
    const { os, arch: cpuArch } = detectOS();
    const res = await fetch(
      "https://api.github.com/repos/joseiedo/game-library/releases/latest",
      { headers: { Accept: "application/vnd.github+json" } },
    );
    if (!res.ok) {
      showNotification("Could not reach update server.", "error");
      return;
    }
    const data = await res.json();
    const remoteVersion = ((data.tag_name as string) ?? "").replace(/^v/, "");
    if (!isNewerVersion(remoteVersion, currentVersion)) {
      if (!silent) showNotification("You're up to date!", "info");
      return;
    }

    const assetUrl = findAssetUrl(data.assets ?? [], os, cpuArch);
    updateInfo.value = {
      version: remoteVersion,
      assetUrl: assetUrl ?? "",
      releaseUrl: data.html_url ?? "https://github.com/joseiedo/game-library/releases",
    };
    showUpdateModal.value = true;
  } catch {
    showNotification("Update check failed. Check your connection.", "error");
  } finally {
    checkingForUpdates.value = false;
  }
}

// ── Lifecycle ──────────────────────────────────────────────────────────────

onMounted(() => {
  loadGames();
  loadAutostart();
  loadIgnoredGames();
  checkForUpdates(true);
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
      :total-games="visibleGames.length"
      :steam-count="steamCount"
      :epic-count="epicCount"
      :custom-count="customCount"
      :sidebar-focused-index="focusArea === 'sidebar' || activeTab === 'settings' ? sidebarFocusedIndex : -1"
      @update:search="search = $event; focusedIndex = 0; switchToLibrary()"
      @update:platform-filter="platformFilter = $event; focusedIndex = 0; switchToLibrary()"
      @update:sort-option="sortOption = $event; switchToLibrary()"
      @add-game="showAddModal = true"
      @open-settings="openSettings"
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

      <!-- Settings view -->
      <SettingsView
        v-else-if="activeTab === 'settings'"
        :autostart-enabled="autostartEnabled"
        :ignored-games="ignoredGamesInfo"
        :focused-index="focusArea === 'settings' ? settingsFocusedIndex : -1"
        :checking-for-updates="checkingForUpdates"
        @toggle-autostart="toggleAutostart"
        @remove-ignored="removeIgnoredGame"
        @open-ignore-picker="showIgnorePicker = true"
        @check-for-updates="checkForUpdates"
      />

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

    <IgnorePickerModal
      v-if="showIgnorePicker"
      :games="availableForIgnore"
      @ignore="addIgnoredGame"
      @close="showIgnorePicker = false"
    />

    <UpdateAvailableModal
      v-if="showUpdateModal && updateInfo && updateInfo.assetUrl"
      :version="updateInfo.version"
      :asset-url="updateInfo.assetUrl"
      :release-url="updateInfo.releaseUrl"
      @dismiss="showUpdateModal = false; updateInfo = null"
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
