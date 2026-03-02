<script setup lang="ts">
import { reactive, ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import FileExplorer from "./FileExplorer.vue";
import VirtualKeyboard from "./VirtualKeyboard.vue";
import { useGamepad } from "../composables/useGamepad";
import type { CustomGame } from "../types/game";

const props = defineProps<{
  mode: "add" | "edit";
  editData?: CustomGame | null;
}>();

const emit = defineEmits<{
  close: [];
  added: [game: CustomGame];
  updated: [game: CustomGame];
}>();

const isEdit = computed(() => props.mode === "edit");

const form = reactive({
  id: "",
  title: "",
  executable: "",
  coverImage: "",
  tags: "",
  notes: "",
  submitting: false,
  error: "",
});

const explorerMode = ref<"executable" | "cover" | null>(null);
const virtualKeyboardTarget = ref<"title" | "executable" | "coverImage" | "tags" | "notes" | null>(null);
const mouseDownOnOverlay = ref(false);

async function submit() {
  if (!form.title.trim() || !form.executable.trim()) {
    form.error = "Title and executable are required.";
    return;
  }

  form.submitting = true;
  form.error = "";
  try {
    if (isEdit.value) {
      const game = await invoke<CustomGame>("edit_game", {
        id: form.id,
        title: form.title.trim(),
        executable: form.executable.trim(),
        coverImage: form.coverImage.trim() || null,
        tags: form.tags.split(",").map((t) => t.trim()).filter(Boolean),
        notes: form.notes.trim() || null,
      });
      emit("updated", game);
    } else {
      const game = await invoke<CustomGame>("add_game", {
        title: form.title.trim(),
        executable: form.executable.trim(),
        coverImage: form.coverImage.trim() || null,
        tags: form.tags.split(",").map((t) => t.trim()).filter(Boolean),
        notes: form.notes.trim() || null,
      });
      emit("added", game);
    }
  } catch (e) {
    form.error = String(e);
  } finally {
    form.submitting = false;
  }
}

// ── Form controller navigation ─────────────────────────────────────────────

// Indices map to the visual top-to-bottom order of focusable elements
const FORM_ITEMS = [
  "title", "executable", "browse-executable", "coverImage", "browse-cover",
  "tags", "notes", "cancel", "submit"
] as const;
const formFocusedIndex = ref(0);
const formInputActive = ref(false);

const titleRef = ref<HTMLInputElement>();
const executableRef = ref<HTMLInputElement>();
const browseExecRef = ref<HTMLButtonElement>();
const coverImageRef = ref<HTMLInputElement>();
const browseCoverRef = ref<HTMLButtonElement>();
const tagsRef = ref<HTMLInputElement>();
const notesRef = ref<HTMLTextAreaElement>();
const cancelRef = ref<HTMLButtonElement>();
const submitRef = ref<HTMLButtonElement>();

function blurActiveInput() {
  (document.activeElement as HTMLElement)?.blur();
  formInputActive.value = false;
}

function activateFormItem() {
  const item = FORM_ITEMS[formFocusedIndex.value];
  switch (item) {
    case "title":
      formInputActive.value = true;
      titleRef.value?.focus();
      break;
    case "executable":
      formInputActive.value = true;
      executableRef.value?.focus();
      break;
    case "browse-executable":
      explorerMode.value = "executable";
      break;
    case "coverImage":
      formInputActive.value = true;
      coverImageRef.value?.focus();
      break;
    case "browse-cover":
      explorerMode.value = "cover";
      break;
    case "tags":
      formInputActive.value = true;
      tagsRef.value?.focus();
      break;
    case "notes":
      formInputActive.value = true;
      notesRef.value?.focus();
      break;
    case "cancel":
      emit("close");
      break;
    case "submit":
      submit();
      break;
  }
}

function navigateForm(dir: "up" | "down") {
  formFocusedIndex.value =
    dir === "up"
      ? Math.max(formFocusedIndex.value - 1, 0)
      : Math.min(formFocusedIndex.value + 1, FORM_ITEMS.length - 1);
}

function onKeyDown(e: KeyboardEvent) {
  if (explorerMode.value) return;

  if (formInputActive.value) {
    if (e.key === "Escape") {
      e.stopPropagation();
      blurActiveInput();
    }
    return;
  }

  switch (e.key) {
    case "Escape":
      emit("close");
      break;
    case "ArrowUp":
      e.preventDefault();
      navigateForm("up");
      break;
    case "ArrowDown":
      e.preventDefault();
      navigateForm("down");
      break;
    case "Enter":
      e.preventDefault();
      activateFormItem();
      break;
  }
}

// ── Gamepad navigation ─────────────────────────────────────────────────────

// Disable our gamepad handler while FileExplorer or VirtualKeyboard is open —
// they own input then. Ghost press prevention is automatic: when disabled, the
// composable still tracks physical state so no buttons fire spuriously on re-enable.
const gamepadEnabled = computed(
  () => explorerMode.value === null && virtualKeyboardTarget.value === null
);

const TEXT_INPUTS = new Set(["title", "executable", "coverImage", "tags", "notes"]);

useGamepad((action) => {
  if (formInputActive.value) {
    if (action === "b") blurActiveInput();
    return;
  }
  switch (action) {
    case "up":   navigateForm("up");   break;
    case "down": navigateForm("down"); break;
    case "a": {
      const focused = FORM_ITEMS[formFocusedIndex.value];
      if (TEXT_INPUTS.has(focused)) {
        virtualKeyboardTarget.value = focused as typeof virtualKeyboardTarget.value;
      } else {
        activateFormItem();
      }
      break;
    }
    case "b":    emit("close");        break;
  }
}, { enabled: gamepadEnabled });

function onVirtualKeyboardConfirm() {
  virtualKeyboardTarget.value = null;
}

function onExplorerSelect(path: string) {
  if (explorerMode.value === "executable") {
    form.executable = path;
  } else if (explorerMode.value === "cover") {
    form.coverImage = path;
  }
  explorerMode.value = null;
}

function onBackdropMouseDown(e: MouseEvent) {
  mouseDownOnOverlay.value = e.target === e.currentTarget;
}

function onBackdropMouseUp(e: MouseEvent) {
  if (mouseDownOnOverlay.value && e.target === e.currentTarget) emit("close");
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  if (props.editData) {
    form.id = props.editData.id;
    form.title = props.editData.title;
    form.executable = props.editData.executable;
    form.coverImage = props.editData.cover_image || "";
    form.tags = props.editData.tags.join(", ");
    form.notes = props.editData.notes || "";
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    @mousedown="onBackdropMouseDown"
    @mouseup="onBackdropMouseUp"
  >
    <div class="w-full max-w-md bg-zinc-950 rounded-lg shadow-2xl border border-zinc-800 p-5">

      <div class="flex items-center justify-between mb-5">
        <h2 class="text-white text-sm font-semibold">{{ isEdit ? "Edit Game" : "Add Custom Game" }}</h2>
        <button
          @click="emit('close')"
          class="text-zinc-500 hover:text-white transition-colors"
          aria-label="Close"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form @submit.prevent="submit" class="flex flex-col gap-4">

        <!-- Title -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs text-zinc-400 font-medium">
            Title <span class="text-zinc-600">*</span>
          </label>
          <input
            ref="titleRef"
            v-model="form.title"
            type="text"
            placeholder="e.g. Hollow Knight"
            class="w-full px-3 py-1.5 bg-zinc-900 text-white text-sm rounded-md border border-zinc-800
                   focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                   placeholder-zinc-600 transition-colors"
            :class="FORM_ITEMS[formFocusedIndex] === 'title' ? 'ring-2 ring-zinc-500' : ''"
            @focus="formInputActive = true; formFocusedIndex = FORM_ITEMS.indexOf('title')"
            @blur="formInputActive = false"
          />
        </div>

        <!-- Executable -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs text-zinc-400 font-medium">
            Executable <span class="text-zinc-600">*</span>
          </label>
          <div class="flex gap-2">
            <input
              ref="executableRef"
              v-model="form.executable"
              type="text"
              placeholder="Path to executable or .app bundle"
              class="flex-1 min-w-0 px-3 py-1.5 bg-zinc-900 text-white text-sm rounded-md border border-zinc-800
                     focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                     placeholder-zinc-600 transition-colors"
            :class="FORM_ITEMS[formFocusedIndex] === 'executable' ? 'ring-2 ring-zinc-500' : ''"
            @focus="formInputActive = true; formFocusedIndex = FORM_ITEMS.indexOf('executable')"
              @blur="formInputActive = false"
            />
            <button
              ref="browseExecRef"
              type="button"
              @click="explorerMode = 'executable'"
              class="shrink-0 px-3 py-1.5 text-sm rounded-md border border-zinc-700
                     text-zinc-400 hover:bg-zinc-800 hover:text-white transition-colors"
              :class="FORM_ITEMS[formFocusedIndex] === 'browse-executable' ? 'ring-2 ring-zinc-500' : ''"
              title="Browse"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Cover image -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs text-zinc-400 font-medium">Cover image</label>
          <div class="flex gap-2">
            <input
              ref="coverImageRef"
              v-model="form.coverImage"
              type="text"
              placeholder="Path to image (optional)"
              class="flex-1 min-w-0 px-3 py-1.5 bg-zinc-900 text-white text-sm rounded-md border border-zinc-800
                     focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                     placeholder-zinc-600 transition-colors"
              :class="FORM_ITEMS[formFocusedIndex] === 'coverImage' ? 'ring-2 ring-zinc-500' : ''"
              @focus="formInputActive = true; formFocusedIndex = FORM_ITEMS.indexOf('coverImage')"
              @blur="formInputActive = false"
            />
            <button
              ref="browseCoverRef"
              type="button"
              @click="explorerMode = 'cover'"
              class="shrink-0 px-3 py-1.5 text-sm rounded-md border border-zinc-700
                     text-zinc-400 hover:bg-zinc-800 hover:text-white transition-colors"
              :class="FORM_ITEMS[formFocusedIndex] === 'browse-cover' ? 'ring-2 ring-zinc-500' : ''"
              title="Browse"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14
                     m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Tags -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs text-zinc-400 font-medium">Tags</label>
          <input
            ref="tagsRef"
            v-model="form.tags"
            type="text"
            placeholder="rpg, indie, metroidvania (comma-separated)"
            class="w-full px-3 py-1.5 bg-zinc-900 text-white text-sm rounded-md border border-zinc-800
                   focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                   placeholder-zinc-600 transition-colors"
            :class="FORM_ITEMS[formFocusedIndex] === 'tags' ? 'ring-2 ring-zinc-500' : ''"
            @focus="formInputActive = true; formFocusedIndex = FORM_ITEMS.indexOf('tags')"
            @blur="formInputActive = false"
          />
        </div>

        <!-- Notes -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs text-zinc-400 font-medium">Notes</label>
          <textarea
            ref="notesRef"
            v-model="form.notes"
            rows="2"
            placeholder="Optional notes…"
            class="w-full px-3 py-1.5 bg-zinc-900 text-white text-sm rounded-md border border-zinc-800
                   focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                   placeholder-zinc-600 resize-none transition-colors"
            :class="FORM_ITEMS[formFocusedIndex] === 'notes' ? 'ring-2 ring-zinc-500' : ''"
            @focus="formInputActive = true; formFocusedIndex = FORM_ITEMS.indexOf('notes')"
            @blur="formInputActive = false"
          />
        </div>

        <p v-if="form.error" class="text-red-400 text-xs">{{ form.error }}</p>

        <div class="flex justify-end gap-2 pt-1">
          <button
            ref="cancelRef"
            type="button"
            @click="emit('close')"
            class="px-4 py-1.5 text-sm text-zinc-400 hover:text-white rounded-md
                   border border-zinc-700 hover:bg-zinc-800 transition-colors"
            :class="FORM_ITEMS[formFocusedIndex] === 'cancel' ? 'ring-2 ring-zinc-500' : ''"
          >
            Cancel
          </button>
          <button
            ref="submitRef"
            type="submit"
            :disabled="form.submitting"
            class="px-4 py-1.5 text-sm font-medium bg-white text-zinc-950
                   rounded-md hover:bg-zinc-100 transition-colors
                   disabled:opacity-40 disabled:cursor-not-allowed"
            :class="FORM_ITEMS[formFocusedIndex] === 'submit' ? 'ring-2 ring-zinc-500 ring-offset-1 ring-offset-zinc-950' : ''"
          >
            {{ form.submitting ? (isEdit ? "Saving…" : "Adding…") : (isEdit ? "Save Changes" : "Add Game") }}
          </button>
        </div>
      </form>
    </div>
  </div>

  <FileExplorer
    v-if="explorerMode === 'executable'"
    title="Select Executable"
    @select="onExplorerSelect"
    @cancel="explorerMode = null"
  />
  <FileExplorer
    v-if="explorerMode === 'cover'"
    title="Select Cover Image"
    @select="onExplorerSelect"
    @cancel="explorerMode = null"
  />

  <VirtualKeyboard
    v-if="virtualKeyboardTarget !== null"
    :model-value="form[virtualKeyboardTarget!]"
    @update:model-value="form[virtualKeyboardTarget!] = $event"
    @confirm="onVirtualKeyboardConfirm"
  />
</template>
