<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { useGamepad } from "../composables/useGamepad";

const props = defineProps<{
  version: string;
  assetUrl: string;
  releaseUrl: string;
}>();
const emit = defineEmits<{ dismiss: [] }>();

type Stage = "available" | "downloading" | "downloaded" | "error";
const stage = ref<Stage>("available");
const downloadedPath = ref("");
const errorMessage = ref("");

async function startDownload() {
  stage.value = "downloading";
  try {
    downloadedPath.value = await invoke<string>("download_update", {
      assetUrl: props.assetUrl,
    });
    stage.value = "downloaded";
  } catch (e) {
    errorMessage.value = String(e);
    stage.value = "error";
  }
}

async function openInstaller() {
  await openPath(downloadedPath.value);
  emit("dismiss");
}

function dismiss() {
  if (stage.value !== "downloading") emit("dismiss");
}

function onKeyDown(e: KeyboardEvent) {
  if (stage.value === "downloading") return;
  if (e.key === "Escape") { e.preventDefault(); dismiss(); }
  if (e.key === "Enter") {
    e.preventDefault();
    if (stage.value === "available") startDownload();
    else if (stage.value === "downloaded") openInstaller();
    else if (stage.value === "error") startDownload();
  }
}

useGamepad((action) => {
  if (stage.value === "downloading") return;
  if (action === "a") {
    if (stage.value === "available" || stage.value === "error") startDownload();
    else if (stage.value === "downloaded") openInstaller();
  } else if (action === "b") dismiss();
});

onMounted(() => window.addEventListener("keydown", onKeyDown));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown));
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    @click.self="dismiss"
  >
    <div class="w-full max-w-sm bg-white dark:bg-zinc-950 rounded-lg shadow-2xl border border-zinc-200 dark:border-zinc-800 p-5 transition-colors duration-200">

      <!-- Header (all stages) -->
      <div class="flex items-center gap-3 mb-4">
        <div class="w-8 h-8 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-zinc-500 dark:text-zinc-300"
               fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        <div>
          <h3 class="text-zinc-900 dark:text-white text-sm font-semibold">Update Available</h3>
          <p class="text-zinc-400 dark:text-zinc-500 text-xs">Version {{ version }}</p>
        </div>
      </div>

      <!-- Stage: available -->
      <template v-if="stage === 'available'">
        <p class="text-xs text-zinc-500 dark:text-zinc-400 mb-4">
          A new version is available. Click "Download" to save the installer to ~/Downloads.
        </p>
        <div class="flex justify-end gap-2">
          <button @click="dismiss"
            class="px-4 py-1.5 text-sm text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white rounded-md
                   border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors">
            Later
          </button>
          <button @click="startDownload"
            class="px-4 py-1.5 text-sm font-medium bg-zinc-900 text-white dark:bg-white dark:text-zinc-950
                   rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors">
            Download
          </button>
        </div>
      </template>

      <!-- Stage: downloading -->
      <template v-else-if="stage === 'downloading'">
        <div class="flex items-center gap-3 py-2">
          <svg class="animate-spin w-4 h-4 text-zinc-400 dark:text-zinc-500 shrink-0"
               xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10"
                    stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
          </svg>
          <span class="text-xs text-zinc-500 dark:text-zinc-400">Downloading installer…</span>
        </div>
      </template>

      <!-- Stage: downloaded -->
      <template v-else-if="stage === 'downloaded'">
        <p class="text-xs text-zinc-500 dark:text-zinc-400 mb-1">Download complete.</p>
        <p class="text-xs text-zinc-400 dark:text-zinc-600 mb-4 truncate" :title="downloadedPath">
          {{ downloadedPath }}
        </p>
        <div class="flex justify-end gap-2">
          <button @click="dismiss"
            class="px-4 py-1.5 text-sm text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white rounded-md
                   border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors">
            Close
          </button>
          <button @click="openInstaller"
            class="px-4 py-1.5 text-sm font-medium bg-zinc-900 text-white dark:bg-white dark:text-zinc-950
                   rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors">
            Open Installer
          </button>
        </div>
      </template>

      <!-- Stage: error -->
      <template v-else-if="stage === 'error'">
        <p class="text-xs text-red-500 dark:text-red-400 mb-4">{{ errorMessage }}</p>
        <div class="flex justify-end gap-2">
          <button @click="dismiss"
            class="px-4 py-1.5 text-sm text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white rounded-md
                   border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors">
            Close
          </button>
          <button @click="startDownload"
            class="px-4 py-1.5 text-sm font-medium bg-zinc-900 text-white dark:bg-white dark:text-zinc-950
                   rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors">
            Retry
          </button>
        </div>
      </template>

      <!-- Controller hint -->
      <p class="mt-3 text-xs text-zinc-400 dark:text-zinc-600 text-right" v-if="stage !== 'downloading'">
        <kbd class="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-mono">A</kbd> confirm
        &nbsp;·&nbsp;
        <kbd class="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-mono">B / Esc</kbd> dismiss
      </p>
    </div>
  </div>
</template>
