<script setup lang="ts">
import { useMessage } from "naive-ui";
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import { FolderOpen } from "lucide-vue-next";
import ChapterReaderModal from "@/components/explore/ChapterReaderModal.vue";
import GlobalSearchModal from "@/components/explore/GlobalSearchModal.vue";
import type { SearchResult } from "@/composables/useGlobalSearch";
import ShelfGroupMenu from "@/components/shelf/ShelfGroupMenu.vue";
import SmartGroupTabs from "@/components/shelf/SmartGroupTabs.vue";
import BatchActionBar from "@/components/shelf/BatchActionBar.vue";
import { useBatchOperations } from "@/composables/useBatchOperations";
import { useOverlayBackstack } from "@/composables/useOverlayBackstack";
import { useTocAutoUpdate } from "@/composables/useTocAutoUpdate";
import { isTransportAvailable } from "@/composables/useTransport";
import { useViewCardDensity, type CardSizeKey } from "@/composables/useViewCardDensity";
import BookshelfContextMenu from "@/features/bookshelf/components/BookshelfContextMenu.vue";
import BookshelfDialogs from "@/features/bookshelf/components/BookshelfDialogs.vue";
import BookshelfGrid from "@/features/bookshelf/components/BookshelfGrid.vue";
import BookshelfHeader from "@/features/bookshelf/components/BookshelfHeader.vue";
import { useBookshelfActions } from "@/features/bookshelf/services/bookshelfActions";
import { useBookshelfReaderLauncher } from "@/features/bookshelf/services/bookshelfReaderLauncher";
import {
  useBookshelfReaderStore,
  useBookshelfStore,
  useBookshelfUiStore,
  useFrontendPluginsStore,
  useNavigationStore,
  usePrivacyModeStore,
  useShelfGroupsStore,
} from "@/stores";
import type { SmartGroupType } from "@/stores/smartGroups";
import { useSmartGroupsStore } from "@/stores/smartGroups";

const message = useMessage();
const bookshelfStore = useBookshelfStore();
const uiStore = useBookshelfUiStore();
const readerStore = useBookshelfReaderStore();
const frontendPluginsStore = useFrontendPluginsStore();
const privacyModeStore = usePrivacyModeStore();

const dialogsRef = ref<{
  txtImport: { ack: () => void; fail: (msg: string) => void };
  cbzImport: { ack: () => void; fail: (msg: string) => void };
} | null>(null);

const { books, loading } = storeToRefs(bookshelfStore);
const {
  searchKw,
  openingBookId,
  showDropdown,
  dropdownX,
  dropdownY,
  showCoverGeneratorDialog,
  coverGeneratorBook,
  showSourceSwitchDialog,
  switchTargetBook,
  showExportDialog,
  exportBook,
  exportCachedChapters,
  showBookDetailDialog,
  bookDetailBook,
  bookDetailMode,
  showTxtImportDialog,
  showCbzImportDialog,
} = storeToRefs(uiStore);
const { menuOptions } = storeToRefs(uiStore);
const {
  showReader,
  readerFileName,
  readerChapterUrl,
  readerChapterName,
  readerChapters,
  readerCurrentIndex,
  readerShelfId,
  readerBookInfo,
  readerSourceType,
  readerChapterGroups,
  episodeProgressMap,
  refreshingToc,
} = storeToRefs(readerStore);
const { privacyModeEnabled, privacyExitTick } = storeToRefs(privacyModeStore);
const { togglePrivacyMode } = privacyModeStore;

const shelfGroupsStore = useShelfGroupsStore();
const {
  state: shelfGroupsState,
  groupsWithAll,
  filteredBooks,
  lastReadBook,
  selectedTagIds,
} = storeToRefs(shelfGroupsStore);

const batchOps = useBatchOperations();

const smartGroupsStore = useSmartGroupsStore();
const smartGroupFilter = ref<SmartGroupType | null>(null);

function handleSelectSmartGroup(groupId: SmartGroupType | null) {
  smartGroupFilter.value = groupId;
}

const showGroupMenu = computed({
  get: () => uiStore.showGroupMenu ?? false,
  set: (v: boolean) => {
    uiStore.showGroupMenu = v;
  },
});

const showSearch = ref(false);
const searchPopupKw = ref("");

const showGlobalSearch = ref(false);

const editMode = ref(false);
const selectedBookIds = ref<Set<string>>(new Set());

const showTagSheet = ref(false);
const showGroupSheet = ref(false);

function toggleEditMode() {
  editMode.value = !editMode.value;
  if (!editMode.value) {
    selectedBookIds.value = new Set();
  }
}

function enterMultiSelect(bookId?: string) {
  editMode.value = true;
  if (bookId) {
    selectedBookIds.value = new Set([bookId]);
  }
}

function handleEnterMultiSelect(book: { id: string }) {
  enterMultiSelect(book.id);
}

function toggleBookSelect(bookId: string) {
  const next = new Set(selectedBookIds.value);
  if (next.has(bookId)) {
    next.delete(bookId);
  } else {
    next.add(bookId);
  }
  selectedBookIds.value = next;
}

function handleGridToggleSelect(book: { id: string }) {
  toggleBookSelect(book.id);
}

const allSelected = computed(
  () =>
    searchedBooks.value.length > 0 &&
    searchedBooks.value.every((b) => selectedBookIds.value.has(b.id)),
);

function toggleSelectAll() {
  if (allSelected.value) {
    selectedBookIds.value = new Set();
  } else {
    selectedBookIds.value = batchOps.selectAll(searchedBooks.value, selectedBookIds.value);
  }
}

function exitMultiSelect() {
  selectedBookIds.value = new Set();
  editMode.value = false;
}

async function handleMarkRead() {
  const ids = [...selectedBookIds.value];
  if (!ids.length) return;
  await batchOps.batchMarkRead(ids);
  message.success(`已标记 ${ids.length} 本书为已读`);
}

async function handleMarkUnread() {
  const ids = [...selectedBookIds.value];
  if (!ids.length) return;
  await batchOps.batchMarkUnread(ids);
  message.success(`已标记 ${ids.length} 本书为未读`);
}

async function handleBatchDelete() {
  const ids = [...selectedBookIds.value];
  if (!ids.length) return;
  await batchOps.batchDelete(ids);
  selectedBookIds.value = new Set();
  editMode.value = false;
  message.success(`已删除 ${ids.length} 本书`);
}

async function handleBatchAddTags(tagIds: string[]) {
  const ids = [...selectedBookIds.value];
  if (!ids.length) return;
  await batchOps.batchAddTags(ids, tagIds);
  message.success(`已为 ${ids.length} 本书添加标签`);
  showTagSheet.value = false;
}

async function handleBatchMoveGroup(groupId: string) {
  const ids = [...selectedBookIds.value];
  if (!ids.length) return;
  await batchOps.batchMoveGroup(ids, groupId);
  message.success(`已将 ${ids.length} 本书移动到目标分组`);
  showGroupSheet.value = false;
}

const availableTags = computed(() => shelfGroupsState.value.tags);

const availableGroups = computed(() =>
  shelfGroupsState.value.groups.filter((g) => g.id !== shelfGroupsState.value.activeGroupId),
);

const contextBookGroupId = computed(() => {
  const bookId = uiStore.contextBook?.id;
  if (!bookId) {
    return undefined;
  }
  return shelfGroupsState.value.bookGroupMap[bookId] ?? "all";
});

const {
  cardSizes: CARD_SIZES,
  activeSize,
  activeSizeKey,
  style: bookshelfDensityStyle,
  setSize,
} = useViewCardDensity("bookshelf");

const navigationStore = useNavigationStore();
const { activeView } = storeToRefs(navigationStore);

const readerLauncher = useBookshelfReaderLauncher(message);
const bookshelfActions = useBookshelfActions(message);
const tocAutoUpdate = useTocAutoUpdate();

function onTxtImported(payload: {
  title: string;
  author: string;
  chapters: Array<{ title: string; content: string }>;
  preface: string;
}) {
  return bookshelfActions.handleTxtImported(payload, dialogsRef.value?.txtImport);
}

function onCbzImported(payload: {
  title: string;
  pages: string[];
  coverUrl: string;
}) {
  return bookshelfActions.handleCbzImported(payload, dialogsRef.value?.cbzImport);
}

async function handleGlobalSearchNavigate(result: SearchResult) {
  showGlobalSearch.value = false;
  const book = bookshelfStore.books.find((b) => b.id === result.bookId);
  if (!book) {
    message.warning("书籍已不在书架中");
    return;
  }
  await readerLauncher.openBook(book);
  readerStore.openAt(result.chapterIndex);
}

useOverlayBackstack(
  () => showDropdown.value,
  () => {
    showDropdown.value = false;
  },
);

const visibleBookCount = computed(() => {
  if (privacyModeStore.privacyModeEnabled) {
    return filteredBooks.value.filter((book) => book.isPrivate).length;
  }
  return filteredBooks.value.filter((book) => !book.isPrivate).length;
});

const switchTargetChapters = computed(() =>
  bookshelfActions.currentChaptersForSwitch(switchTargetBook.value),
);

const sortedBooks = computed(() => {
  const privacyFiltered = privacyModeEnabled.value
    ? filteredBooks.value.filter((b) => b.isPrivate)
    : filteredBooks.value.filter((b) => !b.isPrivate);
  const booksToSort = [...privacyFiltered];
  const lastRead = lastReadBook.value;

  return booksToSort.toSorted((a, b) => {
    if (lastRead) {
      if (a.id === lastRead.id) {
        return -1;
      }
      if (b.id === lastRead.id) {
        return 1;
      }
    }

    const aHasRead = a.lastReadAt > 0;
    const bHasRead = b.lastReadAt > 0;
    if (aHasRead && !bHasRead) {
      return -1;
    }
    if (!aHasRead && bHasRead) {
      return 1;
    }
    if (aHasRead && bHasRead) {
      return b.lastReadAt - a.lastReadAt;
    }

    return b.addedAt - a.addedAt;
  });
});

const searchedBooks = computed(() => {
  const kw = searchKw.value.trim().toLowerCase();
  if (!kw) {
    return sortedBooks.value;
  }
  return sortedBooks.value.filter(
    (book) => book.name.toLowerCase().includes(kw) || book.author.toLowerCase().includes(kw),
  );
});

let _shelfViewWatchInitialized = false;
watch(activeView, (newView) => {
  if (!_shelfViewWatchInitialized) {
    _shelfViewWatchInitialized = true;
    return;
  }
  if (newView === "bookshelf") {
    tocAutoUpdate.refreshAllOnShelfView();
  }
});

watch(privacyExitTick, () => {
  if (!showReader.value || !readerShelfId.value) {
    return;
  }
  if (bookshelfStore.isPrivateShelfBook(readerShelfId.value)) {
    showReader.value = false;
  }
});

async function handleAddGroup(name: string) {
  await shelfGroupsStore.addGroup(name);
  message.success("分组已创建");
}

function handleRemoveGroup(groupId: string) {
  shelfGroupsStore.removeGroup(groupId);
  message.success("分组已删除");
}

async function handleRenameGroup(groupId: string, name: string) {
  await shelfGroupsStore.renameGroup(groupId, name);
  message.success("分组已重命名");
}

function handleToggleGroup(groupId: string, enabled: boolean) {
  shelfGroupsStore.setGroupEnabled(groupId, enabled);
}

function handleToggleAllGroup() {
  shelfGroupsStore.toggleAllGroupEnabled();
}

function handleCreateTag(name: string) {
  shelfGroupsStore.createTag(name);
  message.success("标签已创建");
}

function handleToggleTag(tagId: string) {
  shelfGroupsStore.toggleTagFilter(tagId);
}

function handleClearTagFilter() {
  shelfGroupsStore.clearTagFilter();
}

async function handleRefresh() {
  const result = await tocAutoUpdate.refreshAllOnShelfView();

  if (result.updated > 0) {
    message.success(`发现 ${result.updated} 个新章节`);
  } else if (result.success > 0) {
    message.info("已是最新，没有新章节");
  } else if (result.failed > 0) {
    message.warning(`刷新完成，${result.failed} 本书刷新失败`);
  } else {
    message.info("没有需要刷新的书籍");
  }
}

onMounted(async () => {
  privacyModeStore.setupAutoExit();
  if (!(await isTransportAvailable())) {
    return;
  }
  loading.value = true;
  try {
    await Promise.all([bookshelfStore.loadBooks(), frontendPluginsStore.ensureInitialized()]);
    smartGroupsStore.autoClassifyAll(bookshelfStore.books);
  } catch (error: unknown) {
    message.error(`加载书架失败: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    loading.value = false;
  }
  tocAutoUpdate.refreshAllOnAppStart();
});
</script>

<template>
  <div class="bookshelf-view app-page" :style="bookshelfDensityStyle">
    <BookshelfHeader
      :book-count="visibleBookCount"
      :privacy-mode-enabled="privacyModeEnabled"
      :card-sizes="CARD_SIZES"
      :active-size-key="activeSizeKey"
      :active-size-label="activeSize.label"
      :groups="groupsWithAll"
      :active-group-id="shelfGroupsState.activeGroupId"
      :show-group-menu="showGroupMenu"
      :loading="loading"
      :tags="shelfGroupsState.tags"
      :selected-tag-ids="selectedTagIds"
      @set-size="(key: CardSizeKey) => setSize(key)"
      @toggle-privacy="togglePrivacyMode"
      @toggle-group-menu="showGroupMenu = !showGroupMenu"
      @select-group="(id: string) => shelfGroupsStore.selectGroup(id)"
      @import-txt="uiStore.showTxtImportDialog = true"
      @import-cbz="uiStore.showCbzImportDialog = true"
      @refresh="handleRefresh"
      @toggle-search="showSearch = !showSearch"
      @toggle-global-search="showGlobalSearch = !showGlobalSearch"
      @toggle-edit="toggleEditMode"
      @create-tag="handleCreateTag"
      @toggle-tag="handleToggleTag"
      @clear-tag-filter="handleClearTagFilter"
    />

    <SmartGroupTabs
      :active-group="smartGroupFilter"
      @select-group="handleSelectSmartGroup"
    />

    <BookshelfGrid
      :loading="loading"
      :books="books"
      :filtered-books="searchedBooks"
      :privacy-mode-enabled="privacyModeEnabled"
      :opening-book-id="editMode ? null : openingBookId"
      :edit-mode="editMode"
      :selected-book-ids="selectedBookIds"
      :smart-group-filter="smartGroupFilter"
      @select="editMode ? toggleBookSelect($event.id) : readerLauncher.openBook($event)"
      @contextmenu="(book, e) => !editMode && uiStore.openContextMenu(book, e)"
      @refresh="handleRefresh"
      @enter-multiselect="handleEnterMultiSelect"
      @toggle-select="handleGridToggleSelect"
    />

    <BatchActionBar
      :selected-count="selectedBookIds.size"
      :all-selected="allSelected"
      @select-all="toggleSelectAll"
      @add-tags="showTagSheet = true"
      @move-group="showGroupSheet = true"
      @mark-read="handleMarkRead"
      @mark-unread="handleMarkUnread"
      @delete="handleBatchDelete"
      @cancel="exitMultiSelect"
    />

    <!-- 标签选择弹窗 -->
    <n-modal
      v-model:show="showTagSheet"
      preset="card"
      title="选择标签"
      :style="{ width: '380px', maxWidth: '92vw' }"
      :segmented="{ content: true }"
    >
      <div class="bs-batch-sheet">
        <template v-if="availableTags.length">
          <div class="bs-batch-sheet__list">
            <div
              v-for="tag in availableTags"
              :key="tag.id"
              class="bs-batch-sheet__item"
              role="button"
              tabindex="0"
              @click="handleBatchAddTags([tag.id])"
              @keydown.enter.prevent="handleBatchAddTags([tag.id])"
            >
              <span
                class="bs-batch-sheet__tag-dot"
                :style="{ background: tag.color }"
              />
              <span class="bs-batch-sheet__item-name">{{ tag.name }}</span>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="bs-batch-sheet__empty">暂无标签，请先在书架头部创建标签</div>
        </template>
      </div>
    </n-modal>

    <!-- 分组选择弹窗 -->
    <n-modal
      v-model:show="showGroupSheet"
      preset="card"
      title="选择目标分组"
      :style="{ width: '380px', maxWidth: '92vw' }"
      :segmented="{ content: true }"
    >
      <div class="bs-batch-sheet">
        <template v-if="availableGroups.length">
          <div class="bs-batch-sheet__list">
            <div
              v-for="group in availableGroups"
              :key="group.id"
              class="bs-batch-sheet__item"
              role="button"
              tabindex="0"
              @click="handleBatchMoveGroup(group.id)"
              @keydown.enter.prevent="handleBatchMoveGroup(group.id)"
            >
              <FolderOpen :size="16" class="bs-batch-sheet__item-icon" />
              <span class="bs-batch-sheet__item-name">{{ group.name }}</span>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="bs-batch-sheet__empty">暂无可用的其他分组</div>
        </template>
      </div>
    </n-modal>

    <!-- 搜索弹出层 -->
    <n-modal
      v-model:show="showSearch"
      preset="card"
      title="搜索书架"
      :style="{ width: '380px', maxWidth: '92vw' }"
      :segmented="{ content: true }"
      @after-leave="searchPopupKw = ''"
    >
      <n-input v-model:value="searchPopupKw" placeholder="搜索书名或作者..." clearable autofocus />
      <div class="bs-search-results">
        <template v-if="searchPopupKw.trim()">
          <div
            v-for="book in sortedBooks
              .filter(
                (b) =>
                  b.name.toLowerCase().includes(searchPopupKw.trim().toLowerCase()) ||
                  b.author.toLowerCase().includes(searchPopupKw.trim().toLowerCase()),
              )
              .slice(0, 30)"
            :key="book.id"
            class="bs-search-item"
            role="button"
            tabindex="0"
            @click="
              readerLauncher.openBook(book);
              showSearch = false;
            "
            @keydown.enter.prevent="
              readerLauncher.openBook(book);
              showSearch = false;
            "
          >
            <span class="bs-search-item__name">{{ book.name || "未知书名" }}</span>
            <span class="bs-search-item__author">{{ book.author || "佚名" }}</span>
          </div>
          <div
            v-if="
              !sortedBooks.filter(
                (b) =>
                  b.name.toLowerCase().includes(searchPopupKw.trim().toLowerCase()) ||
                  b.author.toLowerCase().includes(searchPopupKw.trim().toLowerCase()),
              ).length
            "
            class="bs-search-empty"
          >
            没有找到匹配的书籍
          </div>
        </template>
        <div v-else class="bs-search-empty">输入关键词以搜索</div>
      </div>
    </n-modal>

    <!-- 全文搜索弹窗 -->
    <GlobalSearchModal
      v-model:show="showGlobalSearch"
      @navigate="handleGlobalSearchNavigate"
    />

    <ShelfGroupMenu
      v-model:show="showGroupMenu"
      :groups="groupsWithAll"
      :active-group-id="shelfGroupsState.activeGroupId"
      :all-group-enabled="shelfGroupsState.allGroupEnabled"
      @select="(id: string) => shelfGroupsStore.selectGroup(id)"
      @add="handleAddGroup"
      @remove="handleRemoveGroup"
      @rename="handleRenameGroup"
      @toggle="handleToggleGroup"
      @toggle-all="handleToggleAllGroup"
    />

    <BookshelfContextMenu
      v-model:show="showDropdown"
      :x="dropdownX"
      :y="dropdownY"
      :options="menuOptions"
      :groups="groupsWithAll"
      :context-book-id="uiStore.contextBook?.id"
      :context-book-group-id="contextBookGroupId"
      @select="bookshelfActions.handleMenuSelect"
    />

    <ChapterReaderModal
      v-model:show="showReader"
      v-model:current-index="readerCurrentIndex"
      :chapter-url="readerChapterUrl"
      :chapter-name="readerChapterName"
      :file-name="readerFileName"
      :chapters="readerChapters"
      :chapter-groups="readerChapterGroups"
      :inline-group-tabs="true"
      :episode-progress="episodeProgressMap"
      :save-episode-progress="(_, url, t, d) => readerStore.setEpisodeProgress(url, t, d)"
      :shelf-book-id="readerShelfId"
      :book-info="readerBookInfo"
      :source-type="readerSourceType"
      :refreshing-toc="refreshingToc"
      @refresh-toc="readerLauncher.refreshToc"
      @source-switched="bookshelfActions.handleReaderSourceSwitched"
    />

    <BookshelfDialogs
      ref="dialogsRef"
      v-model:show-source-switch-dialog="showSourceSwitchDialog"
      v-model:show-cover-generator-dialog="showCoverGeneratorDialog"
      v-model:show-export-dialog="showExportDialog"
      v-model:show-book-detail-dialog="showBookDetailDialog"
      v-model:show-txt-import-dialog="showTxtImportDialog"
      v-model:show-cbz-import-dialog="showCbzImportDialog"
      :switch-target-book="switchTargetBook"
      :switch-target-chapters="switchTargetChapters"
      :cover-generator-book="coverGeneratorBook"
      :export-book="exportBook"
      :export-cached-chapters="exportCachedChapters"
      :book-detail-book="bookDetailBook"
      :book-detail-mode="bookDetailMode"
      @whole-book-switched="bookshelfActions.handleWholeBookSwitched"
      @cover-applied="readerLauncher.syncOpenReaderBookInfo"
      @book-detail-saved="readerLauncher.syncOpenReaderBookInfo"
      @txt-imported="onTxtImported"
      @cbz-imported="onCbzImported"
    />
  </div>
</template>

<style scoped>
.bookshelf-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

/* 标签/分组选择弹窗内部 */
.bs-batch-sheet {
  max-height: 55vh;
  overflow-y: auto;
}

.bs-batch-sheet__list {
  display: flex;
  flex-direction: column;
}

.bs-batch-sheet__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 4px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  border-radius: var(--radius-1);
  transition: background var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-batch-sheet__item:hover {
    background: var(--color-fill-secondary);
  }
}

.bs-batch-sheet__tag-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.bs-batch-sheet__item-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.bs-batch-sheet__item-name {
  font-size: var(--fs-14);
  color: var(--color-text);
}

.bs-batch-sheet__empty {
  text-align: center;
  padding: 24px 0;
  font-size: var(--fs-14);
  color: var(--color-text-muted);
}

/* 搜索弹出层内容 */
.bs-search-results {
  margin-top: 12px;
  max-height: 55vh;
  overflow-y: auto;
}

.bs-search-item {
  display: flex;
  flex-direction: column;
  padding: 10px 4px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  border-radius: var(--radius-1);
  transition: background var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-search-item:hover {
    background: var(--color-fill-secondary);
  }
}

.bs-search-item__name {
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  color: var(--color-text);
}

.bs-search-item__author {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  margin-top: 2px;
}

.bs-search-empty {
  text-align: center;
  padding: 24px 0;
  font-size: var(--fs-14);
  color: var(--color-text-muted);
}
</style>