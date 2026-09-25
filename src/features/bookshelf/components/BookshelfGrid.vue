<script setup lang="ts">
import { computed, ref, type Ref } from "vue";
import { BookOpen } from "lucide-vue-next";
import type { ShelfBook } from "@/stores";
import ShelfBookCard from "@/components/bookshelf/ShelfBookCard.vue";
import { useShelfPullRefresh } from "@/composables/useShelfPullRefresh";
import { useResponsiveControl } from "@/composables/useResponsiveControl";
import { useSmartGroupsStore } from "@/stores/smartGroups";

const LONG_PRESS_DURATION = 500;

const containerRef: Ref<HTMLElement | null> = ref(null);

const props = defineProps<{
  loading: boolean;
  books: ShelfBook[];
  filteredBooks: ShelfBook[];
  privacyModeEnabled: boolean;
  openingBookId: string | null;
  editMode?: boolean;
  selectedBookIds?: Set<string>;
  smartGroupFilter?: string | null;
}>();

const emit = defineEmits<{
  (e: "select", book: ShelfBook): void;
  (e: "contextmenu", book: ShelfBook, event: MouseEvent): void;
  (e: "refresh"): Promise<void>;
  (e: "enter-multiselect", book: ShelfBook): void;
  (e: "toggle-select", book: ShelfBook): void;
}>();

const { pullDistance, isRefreshing, isReady, onTouchStart, onTouchMove, onTouchEnd, onMouseDown } =
  useShelfPullRefresh({
    onRefresh: async () => {
      await emit("refresh");
    },
  });

const { columns, breakpoint } = useResponsiveControl();

const smartGroupsStore = useSmartGroupsStore();

const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${columns.value}, 1fr)`,
}));

const displayBooks = computed(() => {
  if (!props.smartGroupFilter) return props.filteredBooks;
  const group = smartGroupsStore.getGroupByType(props.smartGroupFilter as "unread" | "reading" | "finished");
  if (!group) return props.filteredBooks;
  const idSet = new Set(group.bookIds);
  return props.filteredBooks.filter((b) => idSet.has(b.id));
});

const longPressTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const longPressBook = ref<ShelfBook | null>(null);
const longPressStartX = ref(0);
const longPressStartY = ref(0);

function onCardPointerDown(e: PointerEvent, book: ShelfBook) {
  if (props.editMode) return;
  // 仅左键启动长按多选计时器。右键会触发 contextmenu，
  // 若同时启动计时器，pointerup 时会先 emit('select') 打开阅读页，
  // 导致右键菜单被阅读页遮挡。
  if (e.button !== 0) return;
  longPressBook.value = book;
  longPressStartX.value = e.clientX;
  longPressStartY.value = e.clientY;
  longPressTimer.value = setTimeout(() => {
    longPressTimer.value = null;
    if (longPressBook.value) {
      emit("enter-multiselect", longPressBook.value);
      longPressBook.value = null;
    }
  }, LONG_PRESS_DURATION);
}

function onCardPointerMove(e: PointerEvent) {
  if (!longPressTimer.value) return;
  const dx = Math.abs(e.clientX - longPressStartX.value);
  const dy = Math.abs(e.clientY - longPressStartY.value);
  if (dx > 8 || dy > 8) {
    clearLongPress();
  }
}

function clearLongPress() {
  if (longPressTimer.value) {
    clearTimeout(longPressTimer.value);
    longPressTimer.value = null;
  }
  longPressBook.value = null;
}

function onCardPointerUp(e: PointerEvent, book: ShelfBook) {
  if (longPressTimer.value) {
    clearLongPress();
    if (props.editMode) {
      emit("toggle-select", book);
    } else {
      emit("select", book);
    }
  }
}

function onCardPointerCancel() {
  clearLongPress();
}

function onCardClick(book: ShelfBook) {
  if (props.editMode) {
    emit("toggle-select", book);
  } else {
    emit("select", book);
  }
}

function onCardContextMenu(book: ShelfBook, e: MouseEvent) {
  if (props.editMode) return;
  emit("contextmenu", book, e);
}
</script>

<template>
  <div class="bs-wrapper">
    <div
      class="bs-pull-indicator"
      :class="{
        'bs-pull-indicator--ready': isReady,
        'bs-pull-indicator--refreshing': isRefreshing,
      }"
      :style="{ height: `${pullDistance}px` }"
    >
      <div class="bs-pull-indicator__content">
        <div v-if="isRefreshing" class="bs-pull-indicator__spinner">
          <svg viewBox="0 0 24 24" fill="none">
            <circle
              class="bs-pull-indicator__spinner-track"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="2"
            />
            <circle
              class="bs-pull-indicator__spinner-head"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-dasharray="31.4"
              stroke-dashoffset="10"
            />
          </svg>
        </div>
        <div
          v-else
          class="bs-pull-indicator__arrow"
          :class="{ 'bs-pull-indicator__arrow--up': isReady }"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          >
            <path d="M12 19V5M5 12l7-7 7 7" />
          </svg>
        </div>
        <span class="bs-pull-indicator__text">
          <template v-if="isRefreshing">刷新中...</template>
          <template v-else-if="isReady">释放刷新</template>
          <template v-else>下拉刷新</template>
        </span>
      </div>
    </div>

    <div
      ref="containerRef"
      class="bs-content app-scrollbar"
      @touchstart="onTouchStart"
      @touchmove="onTouchMove"
      @touchend="onTouchEnd"
      @mousedown="onMouseDown"
    >
      <n-spin :show="loading" style="flex: 1">
        <div v-if="!loading && !books.length" class="bs-empty">
          <div class="bs-empty__icon">
            <BookOpen :size="56" :stroke-width="1" />
          </div>
          <h3 class="bs-empty__title">书架空空如也</h3>
          <p class="bs-empty__desc">去发现页搜索书籍，加入书架开始阅读吧</p>
        </div>

        <div v-else class="bs-grid" :style="gridStyle" :data-breakpoint="breakpoint">
          <div
            v-for="book in displayBooks"
            :key="book.id"
            class="bs-grid-item"
            :class="{ 'bs-grid-item--selected': editMode && selectedBookIds?.has(book.id) }"
            @pointerdown.prevent="onCardPointerDown($event, book)"
            @pointermove.prevent="onCardPointerMove($event)"
            @pointerup.prevent="onCardPointerUp($event, book)"
            @pointercancel="onCardPointerCancel"
            @click.left="onCardClick(book)"
            @contextmenu.prevent="onCardContextMenu(book, $event)"
          >
            <ShelfBookCard
              :book="book"
              :privacy-mode-enabled="privacyModeEnabled"
              :loading="openingBookId === book.id"
              :edit-mode="editMode"
              :selected="selectedBookIds?.has(book.id)"
              @select="editMode ? emit('toggle-select', book) : emit('select', book)"
              @contextmenu="(_, e: MouseEvent) => !editMode && emit('contextmenu', book, e)"
            />
          </div>
        </div>
      </n-spin>
    </div>
  </div>
</template>

<style scoped>
.bs-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.bs-pull-indicator {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 0;
  overflow: hidden;
  transition: height 0.1s ease-out;
}

.bs-pull-indicator__content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.bs-pull-indicator--refreshing .bs-pull-indicator__content,
.bs-pull-indicator:not([style*="height: 0"]) .bs-pull-indicator__content {
  opacity: 1;
}

.bs-pull-indicator__spinner {
  width: 20px;
  height: 20px;
  color: var(--color-accent);
}

.bs-pull-indicator__spinner svg {
  width: 100%;
  height: 100%;
}

.bs-pull-indicator__spinner-track {
  opacity: 0.25;
}

.bs-pull-indicator__spinner-head {
  transform-origin: center;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.bs-pull-indicator__arrow {
  width: 20px;
  height: 20px;
  color: var(--color-text-muted);
  transition:
    transform 0.2s ease,
    color 0.2s ease;
}

.bs-pull-indicator__arrow svg {
  width: 100%;
  height: 100%;
}

.bs-pull-indicator--ready .bs-pull-indicator__arrow {
  color: var(--color-accent);
}

.bs-pull-indicator__arrow--up {
  transform: rotate(180deg);
}

.bs-pull-indicator__text {
  font-size: 12px;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.bs-pull-indicator--ready .bs-pull-indicator__text {
  color: var(--color-accent);
}

.bs-pull-indicator--refreshing .bs-pull-indicator__text {
  color: var(--color-accent);
}

.bs-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px 16px;
}

.bs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--book-card-col-min, 120px), 1fr));
  gap: 12px;
  padding-top: 4px;
}

.bs-grid[data-breakpoint="compact"] {
  --bs-card-width: calc((100% - 2 * 12px) / 3);
}

.bs-grid[data-breakpoint="medium"] {
  --bs-card-width: calc((100% - 3 * 12px) / 4);
}

.bs-grid[data-breakpoint="expanded"] {
  --bs-card-width: calc((100% - 4 * 12px) / 5);
}

.bs-grid[data-breakpoint="wide"] {
  --bs-card-width: calc((100% - 5 * 12px) / 6);
}

.bs-grid-item {
  position: relative;
  border-radius: var(--radius-md);
  transition: box-shadow var(--dur-fast) var(--ease-standard);
  touch-action: manipulation;
}

.bs-grid-item--selected {
  border-radius: var(--radius-md);
}

.bs-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  gap: 8px;
}

.bs-empty__icon {
  opacity: 0.25;
  color: var(--color-text-muted);
}

.bs-empty__title {
  font-size: 1rem;
  font-weight: var(--fw-semibold);
  color: var(--color-text-soft);
  margin: 0;
}

.bs-empty__desc {
  font-size: var(--fs-13);
  color: var(--color-text-muted);
  margin: 0;
}

@media (pointer: coarse), (max-width: 640px) {
  .bs-content {
    padding: 0 16px 16px;
  }

  .bs-grid {
    grid-template-columns: repeat(
      auto-fill,
      minmax(var(--book-card-col-min-mobile, var(--book-card-col-min, 100px)), 1fr)
    );
    gap: 8px;
  }
}
</style>