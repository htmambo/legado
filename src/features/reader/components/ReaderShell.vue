<script setup lang="ts">
import type { StyleValue } from "vue";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import { useAndroidLifecycle } from "@/composables/useAndroidLifecycle";
import { useReaderViewStore } from "@/features/reader/stores/readerView";
import { useReaderSessionStore } from "@/features/reader/stores/readerSession";
import { useReadingStats } from "@/composables/useReadingStats";
import { usePrivacyModeStore } from "@/stores/privacyMode";
import { useFinishAnimation } from "@/composables/useFinishAnimation";
import { useReadingGoalStore } from "@/stores/readingGoal";
import { useReadingRecordsStore } from "@/stores/readingRecords";
import { safeRandomUUID } from "@/utils/uuid";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
import { useGamepadControl } from "@/composables/useGamepadControl";
import ReadingGoalRing from "@/components/reading/ReadingGoalRing.vue";
import { storeToRefs } from "pinia";

defineProps<{
  styleValue: StyleValue;
  skinPresetId?: string;
}>();

const message = useMessage();
const privacyStore = usePrivacyModeStore();
const { isIncognito } = storeToRefs(privacyStore);

const { onPause, onResume, onDestroy } = useAndroidLifecycle();

const readerViewStore = useReaderViewStore();
const readerSessionStore = useReaderSessionStore();
const { currentShelfId, bookName, readingChapterIndex, bookInfo } = storeToRefs(readerViewStore);
const { currentPageIndex, currentScrollRatio, content } = storeToRefs(readerSessionStore);

const { recordSession } = useReadingStats();
const readingRecordsStore = useReadingRecordsStore();
const sessionStartTime = ref(0);
const sessionRecStart = ref({ bookId: '', bookName: '', startChapter: 0, timestamp: 0, startPage: 0 });

let incognitoToastShown = false;
let finishCelebratedBookId: string | null = null;

const { overlayVisible, showFinishCelebration } = useFinishAnimation();

const keyboardShortcuts = useKeyboardShortcuts();
const gamepadControl = useGamepadControl();

watch(readingChapterIndex, (newChapterIdx) => {
  const total = bookInfo.value?.totalChapters ?? 0;
  const bookId = currentShelfId.value;
  if (total <= 0 || !bookId) return;
  if (newChapterIdx >= total - 1 && bookId !== finishCelebratedBookId) {
    finishCelebratedBookId = bookId;
    showFinishCelebration();
  }
});

function showIncognitoExitToast() {
  if (incognitoToastShown) return;
  incognitoToastShown = true;
  if (!isIncognito.value) return;
  const bookId = currentShelfId.value;
  if (bookId) {
    privacyStore.addSkippedBook(bookId);
  }
  message.info("本次阅读未保存进度");
}

function getPageOffset(): number {
  if (currentPageIndex.value >= 0) return currentPageIndex.value;
  if (currentScrollRatio.value >= 0) return currentScrollRatio.value;
  return 0;
}

function saveReadingProgress() {
  if (isIncognito.value) return;
  const bookId = currentShelfId.value;
  if (!bookId) return;

  const progressData = {
    chapterIndex: readingChapterIndex.value,
    pageOffset: getPageOffset(),
    timestamp: Date.now(),
  };

  try {
    localStorage.setItem(`legado-reading-progress-${bookId}`, JSON.stringify(progressData));

    localStorage.setItem(
      "legado-last-reading",
      JSON.stringify({
        bookId,
        bookName: bookName.value,
        chapterIndex: readingChapterIndex.value,
        pageOffset: getPageOffset(),
        currentTab: "reader",
        timestamp: Date.now(),
      }),
    );
  } catch {
    // storage full or unavailable
  }
}

function finishSession() {
  if (!sessionStartTime.value) return
  if (isIncognito.value) {
    sessionStartTime.value = 0
    return
  }
  const bookId = currentShelfId.value
  if (!bookId) return
  const durationMs = Date.now() - sessionStartTime.value
  const wordCount = content.value ? content.value.length : 0
  recordSession(bookId, durationMs, wordCount)
  void useReadingGoalStore().addReadingMinutes(Math.round(durationMs / 60000))
  const endPage = getPageOffset()
  const pagesRead = Math.max(1, Math.abs(endPage - sessionRecStart.value.startPage))
  const coverUrl = typeof bookInfo.value?.coverUrl === 'string' ? bookInfo.value.coverUrl : ''
  readingRecordsStore.addSession({
    id: safeRandomUUID(),
    bookId,
    bookName: bookName.value || '',
    coverUrl,
    startTime: sessionRecStart.value.timestamp,
    endTime: Date.now(),
    duration: Math.round(durationMs / 1000),
    startChapter: sessionRecStart.value.startChapter,
    endChapter: readingChapterIndex.value,
    pagesRead,
  })
  sessionStartTime.value = 0
}

function resetRecStart() {
  sessionRecStart.value = {
    bookId: currentShelfId.value || '',
    bookName: bookName.value || '',
    startChapter: readingChapterIndex.value,
    timestamp: Date.now(),
    startPage: getPageOffset(),
  }
}

function onVisibilityChange() {
  if (document.visibilityState === "hidden") {
    finishSession()
  } else if (document.visibilityState === "visible") {
    sessionStartTime.value = Date.now()
    resetRecStart()
  }
}

onMounted(() => {
  incognitoToastShown = false
  sessionStartTime.value = Date.now()
  resetRecStart()
  document.addEventListener("visibilitychange", onVisibilityChange)
  keyboardShortcuts.startListening()
  gamepadControl.startListening()
})

onBeforeUnmount(() => {
  finishSession()
  showIncognitoExitToast()
  document.removeEventListener("visibilitychange", onVisibilityChange)
  keyboardShortcuts.stopListening()
  gamepadControl.stopListening()
})

onPause(async () => {
  finishSession()
  sessionStartTime.value = Date.now()
  saveReadingProgress()
});

onResume(() => {
  sessionStartTime.value = Date.now()
  resetRecStart()
});

onDestroy(() => {
  finishSession()
  saveReadingProgress()
  showIncognitoExitToast()
});
</script>

<template>
  <div class="reader-modal" :style="styleValue" :data-reader-skin="skinPresetId || ''">
    <div class="reader-modal__top-bar">
      <ReadingGoalRing />
    </div>
    <slot />
    <Teleport to="body">
      <Transition name="finish-fade">
        <div v-if="overlayVisible" class="finish-celebration-overlay">
          <div class="finish-celebration-card">
            <div class="finish-celebration-emoji">🎉</div>
            <h2 class="finish-celebration-title">恭喜读完！</h2>
            <p class="finish-celebration-sub">你太棒了，又完成了一本书！</p>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.finish-celebration-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
}

.finish-celebration-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-8) var(--space-8);
  border-radius: var(--radius-3);
  background: var(--gray-0);
  box-shadow: var(--shadow-3);
  animation: finish-pop 0.4s var(--ease-standard);
}

.finish-celebration-emoji {
  font-size: 4rem;
  line-height: 1;
  animation: finish-bounce 0.6s var(--ease-standard) infinite alternate;
}

.finish-celebration-title {
  font-size: var(--fs-24);
  font-weight: var(--fw-bold);
  color: var(--color-text);
  margin: 0;
  font-family: var(--font-ui);
}

.finish-celebration-sub {
  font-size: var(--fs-14);
  color: var(--color-text-muted);
  margin: 0;
  font-family: var(--font-ui);
}

.finish-fade-enter-active {
  transition: opacity 0.3s ease;
}

.finish-fade-leave-active {
  transition: opacity 0.5s ease;
}

.finish-fade-enter-from,
.finish-fade-leave-to {
  opacity: 0;
}

@keyframes finish-pop {
  0% {
    transform: scale(0.8);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

@keyframes finish-bounce {
  0% {
    transform: translateY(0);
  }
  100% {
    transform: translateY(-8px);
  }
}
</style>

<style scoped>
.reader-modal {
  position: relative;
}

.reader-modal__top-bar {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  pointer-events: none;
}

.reader-modal__top-bar > * {
  pointer-events: auto;
}
</style>
