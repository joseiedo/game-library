<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useGamepad } from "../composables/useGamepad";

const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{
  "update:modelValue": [string];
  confirm: [];
}>();

// ── Keyboard layout ─────────────────────────────────────────────────────────

const ROWS: string[][] = [
  ["1","2","3","4","5","6","7","8","9","0"],
  ["q","w","e","r","t","y","u","i","o","p"],
  ["a","s","d","f","g","h","j","k","l","⌫"],
  ["z","x","c","v","b","n","m",".","_","-"],
  ["SHIFT","SPACE","DONE"],
];

const cursorRow = ref(1);
const cursorCol = ref(0);
const shiftActive = ref(false);

// ── Key label display ────────────────────────────────────────────────────────

function keyLabel(key: string): string {
  if (key === "⌫" || key === "SHIFT" || key === "SPACE" || key === "DONE") return key;
  return shiftActive.value ? key.toUpperCase() : key;
}

// ── Column mapping between rows of unequal length ───────────────────────────

function mappedCol(fromLen: number, toLen: number, col: number): number {
  if (fromLen === toLen) return col;
  return Math.round((col / (fromLen - 1)) * (toLen - 1));
}

// ── Cursor movement ──────────────────────────────────────────────────────────

function moveUp() {
  if (cursorRow.value === 0) return;
  const fromLen = ROWS[cursorRow.value].length;
  cursorRow.value -= 1;
  const toLen = ROWS[cursorRow.value].length;
  cursorCol.value = mappedCol(fromLen, toLen, cursorCol.value);
}

function moveDown() {
  if (cursorRow.value === ROWS.length - 1) return;
  const fromLen = ROWS[cursorRow.value].length;
  cursorRow.value += 1;
  const toLen = ROWS[cursorRow.value].length;
  cursorCol.value = mappedCol(fromLen, toLen, cursorCol.value);
}

function moveLeft() {
  cursorCol.value = Math.max(cursorCol.value - 1, 0);
}

function moveRight() {
  cursorCol.value = Math.min(cursorCol.value + 1, ROWS[cursorRow.value].length - 1);
}

// ── Key press logic ──────────────────────────────────────────────────────────

function pressKey(key: string) {
  switch (key) {
    case "SHIFT":
      shiftActive.value = !shiftActive.value;
      break;
    case "SPACE":
      emit("update:modelValue", props.modelValue + " ");
      break;
    case "⌫":
      emit("update:modelValue", props.modelValue.slice(0, -1));
      break;
    case "DONE":
      emit("confirm");
      break;
    default: {
      const char = shiftActive.value ? key.toUpperCase() : key;
      emit("update:modelValue", props.modelValue + char);
      shiftActive.value = false; // single-shot shift
      break;
    }
  }
}

function pressCurrentKey() {
  pressKey(ROWS[cursorRow.value][cursorCol.value]);
}

// ── Gamepad input ────────────────────────────────────────────────────────────

useGamepad((action) => {
  switch (action) {
    case "up":    moveUp();          break;
    case "down":  moveDown();        break;
    case "left":  moveLeft();        break;
    case "right": moveRight();       break;
    case "a":     pressCurrentKey(); break;
    case "b":     emit("confirm");   break;
  }
});

// ── Key styling helpers ──────────────────────────────────────────────────────

function keyClass(rowIdx: number, colIdx: number, key: string): string {
  const isFocused = cursorRow.value === rowIdx && cursorCol.value === colIdx;
  const isShiftActive = key === "SHIFT" && shiftActive.value;

  const base = "flex items-center justify-center rounded-md border text-xs font-medium transition-all duration-150 select-none cursor-pointer h-9";

  const sizeClass =
    key === "SPACE"
      ? "flex-1"
      : key === "SHIFT" || key === "DONE"
        ? "w-16"
        : "w-9";

  const colorClass = isFocused
    ? "ring-2 ring-zinc-400 dark:ring-white bg-zinc-100 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-white"
    : isShiftActive
      ? "bg-zinc-200 dark:bg-zinc-600 text-zinc-900 dark:text-white border-zinc-400 dark:border-zinc-500"
      : "bg-white dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 text-zinc-500 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-700 hover:text-zinc-900 dark:hover:text-white shadow-sm";

  return `${base} ${sizeClass} ${colorClass}`;
}

// ── Preview bar cursor blink ─────────────────────────────────────────────────

const showCursor = ref(true);
let blinkInterval: ReturnType<typeof setInterval>;

onMounted(() => {
  blinkInterval = setInterval(() => {
    showCursor.value = !showCursor.value;
  }, 530);
});

onUnmounted(() => {
  clearInterval(blinkInterval);
});
</script>

<template>
  <div class="fixed bottom-0 inset-x-0 z-[70] flex justify-center pb-4 px-4">
    <div class="w-full max-w-xl bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-xl shadow-2xl p-3 flex flex-col gap-2 transition-colors duration-200">

      <!-- Text preview bar -->
      <div class="bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-md px-3 py-1.5 font-mono text-sm text-zinc-900 dark:text-white min-h-[2rem] flex items-center">
        <span>{{ modelValue }}</span>
        <span :class="showCursor ? 'opacity-100' : 'opacity-0'" class="ml-px transition-opacity">|</span>
      </div>

      <!-- Key rows -->
      <div v-for="(row, rowIdx) in ROWS" :key="rowIdx" class="flex gap-1 justify-center">
        <button
          v-for="(key, colIdx) in row"
          :key="key"
          type="button"
          :class="keyClass(rowIdx, colIdx, key)"
          @click="pressKey(key)"
        >
          {{ keyLabel(key) }}
        </button>
      </div>

      <!-- Hint bar -->
      <div class="flex items-center justify-center gap-4 text-zinc-400 dark:text-zinc-600 text-[10px] pt-0.5">
        <span><span class="text-zinc-500 dark:text-zinc-400">A</span> = Type</span>
        <span><span class="text-zinc-500 dark:text-zinc-400">B</span> = Confirm</span>
        <span><span class="text-zinc-500 dark:text-zinc-400">↕↔</span> = Navigate</span>
      </div>

    </div>
  </div>
</template>
