<script setup lang="ts">
import { ref } from "vue";
import type { WholeBookSwitchedPayload } from "@/components/reader/types";
import type { CachedChapter, ChapterItem, ShelfBook } from "@/stores";
import BookCoverGeneratorDialog from "@/components/bookshelf/BookCoverGeneratorDialog.vue";
import BookDetailEditorDialog from "@/components/bookshelf/BookDetailEditorDialog.vue";
import BookExportDialog from "@/components/bookshelf/BookExportDialog.vue";
import BookSourceSwitchDialog from "@/components/explore/BookSourceSwitchDialog.vue";
import TxtImportDialog from "@/features/local-txt/TxtImportDialog.vue";
import CbzImportDialog from "@/components/bookshelf/CbzImportDialog.vue";

interface TxtImportDialogHandle {
  ack: () => void;
  fail: (message: string) => void;
}

interface CbzImportDialogHandle {
  ack?: () => void;
  fail?: (message: string) => void;
}

defineProps<{
  showSourceSwitchDialog: boolean;
  switchTargetBook: ShelfBook | null;
  switchTargetChapters: ChapterItem[];
  showCoverGeneratorDialog: boolean;
  coverGeneratorBook: ShelfBook | null;
  showExportDialog: boolean;
  exportBook: ShelfBook | null;
  exportCachedChapters: CachedChapter[];
  showBookDetailDialog: boolean;
  bookDetailBook: ShelfBook | null;
  bookDetailMode: "view" | "edit";
  showTxtImportDialog: boolean;
  showCbzImportDialog: boolean;
}>();

const emit = defineEmits<{
  (e: "update:showSourceSwitchDialog", value: boolean): void;
  (e: "update:showCoverGeneratorDialog", value: boolean): void;
  (e: "update:showExportDialog", value: boolean): void;
  (e: "update:showBookDetailDialog", value: boolean): void;
  (e: "update:showTxtImportDialog", value: boolean): void;
  (e: "update:showCbzImportDialog", value: boolean): void;
  (e: "whole-book-switched", payload: WholeBookSwitchedPayload): void;
  (e: "cover-applied", bookId: string): void;
  (e: "book-detail-saved", bookId: string): void;
  (
    e: "txt-imported",
    payload: {
      title: string;
      author: string;
      chapters: Array<{ title: string; content: string }>;
      preface: string;
    },
  ): void;
  (
    e: "cbz-imported",
    payload: {
      title: string;
      pages: string[];
      coverUrl: string;
    },
  ): void;
}>();

const txtImportRef = ref<TxtImportDialogHandle | null>(null);
const cbzImportRef = ref<CbzImportDialogHandle | null>(null);

function ackTxtImport() {
  txtImportRef.value?.ack();
}

function failTxtImport(msg: string) {
  txtImportRef.value?.fail(msg);
}

function ackCbzImport() {
  cbzImportRef.value?.ack?.();
}

function failCbzImport(msg: string) {
  cbzImportRef.value?.fail?.(msg);
}

defineExpose({
  txtImport: { ack: ackTxtImport, fail: failTxtImport },
  cbzImport: { ack: ackCbzImport, fail: failCbzImport },
});
</script>

<template>
  <BookSourceSwitchDialog
    :show="showSourceSwitchDialog"
    mode="whole-book"
    :current-book="{
      name: switchTargetBook?.name ?? '',
      author: switchTargetBook?.author ?? '',
      coverUrl: switchTargetBook?.coverUrl,
      intro: switchTargetBook?.intro,
      kind: switchTargetBook?.kind,
      lastChapter: switchTargetBook?.lastChapter,
      bookUrl: switchTargetBook?.bookUrl,
    }"
    :current-file-name="switchTargetBook?.fileName ?? ''"
    :current-source-name="switchTargetBook?.sourceName ?? ''"
    :current-source-type="switchTargetBook?.sourceType ?? 'novel'"
    :current-chapters="switchTargetChapters"
    :current-read-chapter-index="switchTargetBook?.readChapterIndex ?? -1"
    :current-read-chapter-url="switchTargetBook?.readChapterUrl"
    :shelf-book-id="switchTargetBook?.id"
    @update:show="emit('update:showSourceSwitchDialog', $event)"
    @whole-book-switched="emit('whole-book-switched', $event)"
  />

  <BookCoverGeneratorDialog
    :show="showCoverGeneratorDialog"
    :book="coverGeneratorBook"
    @update:show="emit('update:showCoverGeneratorDialog', $event)"
    @applied="emit('cover-applied', $event)"
  />

  <BookExportDialog
    v-if="exportBook"
    :show="showExportDialog"
    :book="exportBook"
    :chapters="exportCachedChapters"
    @update:show="emit('update:showExportDialog', $event)"
  />

  <BookDetailEditorDialog
    :show="showBookDetailDialog"
    :book="bookDetailBook"
    :initial-mode="bookDetailMode"
    @update:show="emit('update:showBookDetailDialog', $event)"
    @saved="emit('book-detail-saved', $event)"
  />

  <TxtImportDialog
    ref="txtImportRef"
    :show="showTxtImportDialog"
    @update:show="emit('update:showTxtImportDialog', $event)"
    @imported="emit('txt-imported', $event)"
  />

  <CbzImportDialog
    ref="cbzImportRef"
    :show="showCbzImportDialog"
    @update:show="emit('update:showCbzImportDialog', $event)"
    @imported="emit('cbz-imported', $event)"
  />
</template>
