<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGamepad } from "../composables/useGamepad";

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_executable: boolean;
  is_app_bundle: boolean;
}

interface Bookmark {
  label: string;
  path: string;
}

const props = defineProps<{
  title?: string;
  /** If set, the "Select" button only activates for files matching this predicate. */
  filter?: (entry: DirEntry) => boolean;
}>();

const emit = defineEmits<{
  select: [path: string];
  cancel: [];
}>();

// ── State ──────────────────────────────────────────────────────────────────

const currentPath = ref("");
const entries = ref<DirEntry[]>([]);
const bookmarks = ref<Bookmark[]>([]);
const focusedIdx = ref(0);
const loading = ref(false);
const loadError = ref("");
const listEl = ref<HTMLElement | null>(null);

// ── Navigation ─────────────────────────────────────────────────────────────

async function navigate(path: string) {
  loading.value = true;
  loadError.value = "";
  try {
    entries.value = await invoke<DirEntry[]>("list_directory", { path });
    currentPath.value = path;
    focusedIdx.value = 0;
  } catch (e) {
    loadError.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function goUp() {
  if (!currentPath.value) return;
  const sep = currentPath.value.includes("\\") ? "\\" : "/";
  const parts = currentPath.value.split(sep).filter(Boolean);
  if (parts.length === 0) {
    emit("cancel");
    return;
  }
  parts.pop();
  const parent = (currentPath.value.startsWith("/") ? "/" : "") + parts.join(sep) || sep;
  await navigate(parent);
}

async function activate(entry: DirEntry) {
  if (entry.is_dir && !entry.is_app_bundle) {
    await navigate(entry.path);
  } else {
    emit("select", entry.path);
  }
}

function activateFocused() {
  const entry = entries.value[focusedIdx.value];
  if (entry) activate(entry);
}

// ── Computed ───────────────────────────────────────────────────────────────

const focusedEntry = computed(() => entries.value[focusedIdx.value] ?? null);

const canSelect = computed(() => {
  if (!focusedEntry.value) return false;
  if (focusedEntry.value.is_dir) return false;
  if (props.filter) return props.filter(focusedEntry.value);
  return true;
});

const pathParts = computed(() => {
  const sep = currentPath.value.includes("\\") ? "\\" : "/";
  const parts = currentPath.value.split(sep).filter(Boolean);
  return parts.map((part, i) => ({
    label: part,
    path: (currentPath.value.startsWith("/") ? "/" : "") + parts.slice(0, i + 1).join(sep),
  }));
});

// ── Scroll focused item into view ─────────────────────────────────────────

async function scrollIntoView() {
  await nextTick();
  listEl.value
    ?.querySelector(`[data-idx="${focusedIdx.value}"]`)
    ?.scrollIntoView({ block: "nearest" });
}

// ── Keyboard handler ───────────────────────────────────────────────────────

function onKeyDown(e: KeyboardEvent) {
  const count = entries.value.length;

  switch (e.key) {
    case "ArrowUp":
      e.preventDefault();
      e.stopPropagation();
      focusedIdx.value = Math.max(focusedIdx.value - 1, 0);
      scrollIntoView();
      break;
    case "ArrowDown":
      e.preventDefault();
      e.stopPropagation();
      if (count > 0) focusedIdx.value = Math.min(focusedIdx.value + 1, count - 1);
      scrollIntoView();
      break;
    case "Enter":
      e.preventDefault();
      e.stopPropagation();
      activateFocused();
      break;
    case "Backspace":
      e.preventDefault();
      e.stopPropagation();
      goUp();
      break;
    case "Escape":
      e.stopPropagation();
      emit("cancel");
      break;
  }
}

// ── Gamepad navigation ─────────────────────────────────────────────────────

function jumpBookmark(delta: 1 | -1) {
  const idx = bookmarks.value.findIndex((b) => b.path === currentPath.value);
  const next = (idx + delta + bookmarks.value.length) % bookmarks.value.length;
  const bm = bookmarks.value[next];
  if (bm) navigate(bm.path);
}

useGamepad((action) => {
  const count = entries.value.length;
  switch (action) {
    case "up":
      focusedIdx.value = Math.max(focusedIdx.value - 1, 0);
      scrollIntoView();
      break;
    case "down":
      if (count > 0) focusedIdx.value = Math.min(focusedIdx.value + 1, count - 1);
      scrollIntoView();
      break;
    case "a":
      activateFocused();
      break;
    case "b":
      goUp();
      break;
    case "lb":
      jumpBookmark(-1);
      break;
    case "rb":
      jumpBookmark(1);
      break;
  }
}, { repeatDelay: 350, repeatInterval: 100 });

// ── Controller detection ───────────────────────────────────────────────────

const controllerConnected = ref(navigator.getGamepads().some((p) => p !== null));

function onGamepadConnected() {
  controllerConnected.value = true;
}
function onGamepadDisconnected() {
  controllerConnected.value = navigator.getGamepads().some((p) => p !== null);
}

// ── Lifecycle ──────────────────────────────────────────────────────────────

onMounted(async () => {
  // Capture phase so this runs before App.vue's handler
  window.addEventListener("keydown", onKeyDown, { capture: true });
  window.addEventListener("gamepadconnected", onGamepadConnected);
  window.addEventListener("gamepaddisconnected", onGamepadDisconnected);

  const bm = await invoke<Bookmark[]>("get_file_explorer_bookmarks");
  bookmarks.value = bm;
  if (bm.length > 0) {
    await navigate(bm[0].path);
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown, { capture: true });
  window.removeEventListener("gamepadconnected", onGamepadConnected);
  window.removeEventListener("gamepaddisconnected", onGamepadDisconnected);
});
</script>

<template>
  <div
    class="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 backdrop-blur-sm"
    @click.self="emit('cancel')"
  >
    <div class="flex flex-col bg-white dark:bg-zinc-950 rounded-lg border border-zinc-200 dark:border-zinc-800 shadow-2xl
                w-full max-w-2xl h-[70vh] overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800 shrink-0">
        <h2 class="text-sm font-semibold text-zinc-900 dark:text-white">{{ title ?? "Select File" }}</h2>
        <button
          @click="emit('cancel')"
          class="text-zinc-400 hover:text-zinc-900 dark:text-zinc-500 dark:hover:text-white transition-colors"
          aria-label="Close"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="flex flex-1 overflow-hidden">

        <!-- Bookmarks sidebar -->
        <nav class="w-32 shrink-0 border-r border-zinc-200 dark:border-zinc-800 flex flex-col overflow-y-auto py-2 bg-zinc-50 dark:bg-zinc-950">
          <p class="px-3 mb-1 text-[10px] font-medium uppercase tracking-wider text-zinc-400 dark:text-zinc-600">Locations</p>
          <button
            v-for="bm in bookmarks"
            :key="bm.path"
            @click="navigate(bm.path)"
            :title="bm.path"
            class="text-left px-3 py-1.5 text-xs truncate transition-colors"
            :class="currentPath === bm.path
              ? 'bg-zinc-200 text-zinc-900 dark:bg-zinc-800 dark:text-white font-medium'
              : 'text-zinc-500 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200'"
          >
            {{ bm.label }}
          </button>
        </nav>

        <!-- File list pane -->
        <div class="flex-1 flex flex-col overflow-hidden bg-white dark:bg-zinc-950">

          <!-- Breadcrumb -->
          <div class="flex items-center gap-1 px-3 py-2 border-b border-zinc-200 dark:border-zinc-800 shrink-0 overflow-x-auto">
            <button
              v-if="currentPath !== '/'"
              @click="goUp"
              class="shrink-0 text-zinc-400 hover:text-zinc-900 dark:text-zinc-600 dark:hover:text-white mr-1 transition-colors"
              title="Go up"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
            </button>

            <template v-for="(part, i) in pathParts" :key="part.path">
              <span v-if="i > 0" class="text-zinc-300 dark:text-zinc-700 text-xs">/</span>
              <button
                @click="navigate(part.path)"
                class="text-xs text-zinc-400 hover:text-zinc-900 dark:text-zinc-500 dark:hover:text-white transition-colors whitespace-nowrap shrink-0"
                :class="i === pathParts.length - 1 ? 'text-zinc-900 dark:text-zinc-300 font-medium' : ''"
              >
                {{ part.label }}
              </button>
            </template>
          </div>

          <!-- Entries -->
          <div ref="listEl" class="flex-1 overflow-y-auto focus:outline-none">
            <div v-if="loading" class="flex items-center justify-center h-full gap-2 text-zinc-400 dark:text-zinc-600 text-xs">
              <svg class="animate-spin w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"/>
              </svg>
              Loading…
            </div>

            <div v-else-if="loadError" class="px-4 py-3 text-xs text-red-500 dark:text-red-400">{{ loadError }}</div>

            <div v-else-if="entries.length === 0" class="flex items-center justify-center h-full text-zinc-400 dark:text-zinc-600 text-xs">
              Empty folder
            </div>

            <div
              v-else
              v-for="(entry, i) in entries"
              :key="entry.path"
              :data-idx="i"
              @click="activate(entry)"
              class="flex items-center gap-2.5 px-3 py-2 cursor-pointer select-none transition-colors"
              :class="i === focusedIdx
                ? 'bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-white'
                : 'text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200'"
            >
              <!-- Icon -->
              <span class="shrink-0 w-4 text-center">
                <svg v-if="entry.is_app_bundle" xmlns="http://www.w3.org/2000/svg"
                  class="w-3.5 h-3.5 inline text-zinc-400 dark:text-zinc-400"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
                <svg v-else-if="entry.is_dir" xmlns="http://www.w3.org/2000/svg"
                  class="w-3.5 h-3.5 inline text-zinc-400 dark:text-zinc-500"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" />
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg"
                  class="w-3.5 h-3.5 inline text-zinc-300 dark:text-zinc-600"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              </span>

              <!-- Name -->
              <span class="flex-1 text-xs truncate">{{ entry.name }}</span>

              <!-- Badges -->
              <span
                v-if="entry.is_app_bundle"
                class="shrink-0 text-[10px] px-1.5 py-0.5 rounded-sm bg-sky-100 text-sky-700 dark:bg-sky-900/60 dark:text-sky-400"
                :class="i === focusedIdx ? 'bg-sky-200 text-sky-800 dark:bg-sky-800/70 dark:text-sky-300' : ''"
              >.app</span>
              <span
                v-else-if="entry.is_executable"
                class="shrink-0 text-[10px] px-1.5 py-0.5 rounded-sm bg-emerald-100 text-emerald-700 dark:bg-emerald-900/60 dark:text-emerald-400"
                :class="i === focusedIdx ? 'bg-emerald-200 text-emerald-800 dark:bg-emerald-800/70 dark:text-emerald-300' : ''"
              >exec</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between gap-4 px-4 py-2.5 border-t border-zinc-200 dark:border-zinc-800 shrink-0 bg-zinc-50 dark:bg-zinc-950">

        <!-- Controller hints (only shown when a controller is connected) -->
        <div v-if="controllerConnected" class="flex items-center gap-3">
          <span class="flex items-center gap-1 text-[10px] text-zinc-400 dark:text-zinc-600">
            <kbd class="px-1 py-0.5 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded text-zinc-500 dark:text-zinc-400">↕</kbd> Navigate
          </span>
          <span class="flex items-center gap-1 text-[10px] text-zinc-400 dark:text-zinc-600">
            <kbd class="px-1 py-0.5 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded text-zinc-500 dark:text-zinc-400">A</kbd> Open/Select
          </span>
          <span class="flex items-center gap-1 text-[10px] text-zinc-400 dark:text-zinc-600">
            <kbd class="px-1 py-0.5 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded text-zinc-500 dark:text-zinc-400">B</kbd> Back
          </span>
          <span class="flex items-center gap-1 text-[10px] text-zinc-400 dark:text-zinc-600">
            <kbd class="px-1 py-0.5 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded text-zinc-500 dark:text-zinc-400">LB</kbd>
            <kbd class="px-1 py-0.5 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded text-zinc-500 dark:text-zinc-400">RB</kbd> Locations
          </span>
        </div>
        <div v-else />

        <!-- Actions -->
        <div class="flex gap-2 shrink-0">
          <button
            @click="emit('cancel')"
            class="px-3 py-1.5 text-xs rounded-md border border-zinc-200 dark:border-zinc-700
                   text-zinc-500 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 hover:text-zinc-900 dark:hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            v-if="focusedEntry?.is_dir"
            @click="activateFocused"
            class="px-3 py-1.5 text-xs font-medium rounded-md border border-zinc-200 dark:border-zinc-700
                   text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 hover:text-zinc-900 dark:hover:text-white transition-colors"
          >
            Open
          </button>
          <button
            v-else
            :disabled="!canSelect"
            @click="activateFocused"
            class="px-3 py-1.5 text-xs font-medium rounded-md transition-colors
                   bg-zinc-900 text-white dark:bg-white dark:text-zinc-950 hover:bg-zinc-800 dark:hover:bg-zinc-100
                   disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Select
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
