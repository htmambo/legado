<!--
  BookshelfHeader — 书架页顶部标题、操作入口、分组标签栏与标签过滤栏。
-->
<script setup lang="ts">
import {
  LayoutGrid,
  EyeOff,
  Eye,
  FolderPlus,
  FilePlus,
  ImagePlus,
  RefreshCw,
  Search,
  FileSearch,
  Pencil,
  Settings2,
  Plus,
  X,
  Tag,
} from "lucide-vue-next";
import type { DropdownOption } from "naive-ui";
import { computed, ref, nextTick } from "vue";
import MobileToolbarMenu from "@/components/layout/MobileToolbarMenu.vue";
import type { CardSizeKey } from "@/composables/useViewCardDensity";
import type { BookTag, ShelfGroup } from "@/types/shelfGroup";

const props = defineProps<{
  bookCount: number;
  privacyModeEnabled: boolean;
  cardSizes: { key: CardSizeKey; label: string }[];
  activeSizeKey: CardSizeKey;
  activeSizeLabel: string;
  groups: ShelfGroup[];
  activeGroupId: string;
  showGroupMenu: boolean;
  loading?: boolean;
  tags: BookTag[];
  selectedTagIds: string[];
}>();

const emit = defineEmits<{
  (e: "set-size", key: CardSizeKey): void;
  (e: "toggle-privacy"): void;
  (e: "toggle-group-menu"): void;
  (e: "select-group", groupId: string): void;
  (e: "import-txt"): void;
  (e: "import-cbz"): void;
  (e: "refresh"): void;
  (e: "toggle-search"): void;
  (e: "toggle-global-search"): void;
  (e: "toggle-edit"): void;
  (e: "create-tag", name: string): void;
  (e: "toggle-tag", tagId: string): void;
  (e: "clear-tag-filter"): void;
}>();

const showTagInput = ref(false);
const newTagName = ref("");

const enabledGroups = computed(() => {
  return props.groups.filter((g) => g.enabled);
});

const showGroupBar = computed(() => {
  return enabledGroups.value.length > 0;
});

const hasActiveTagFilter = computed(() => {
  return props.selectedTagIds.length > 0;
});

function openTagInput() {
  showTagInput.value = true;
  newTagName.value = "";
  nextTick(() => {
    const input = document.getElementById("new-tag-name-input");
    if (input instanceof HTMLInputElement) {
      input.focus();
    }
  });
}

function confirmTag() {
  const name = newTagName.value.trim();
  if (name) {
    emit("create-tag", name);
  }
  showTagInput.value = false;
  newTagName.value = "";
}

function cancelTag() {
  showTagInput.value = false;
  newTagName.value = "";
}

function handleTagKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    confirmTag();
  } else if (e.key === "Escape") {
    cancelTag();
  }
}

const mobileMenuOptions = computed<DropdownOption[]>(() => [
  {
    label: "搜索书架",
    key: "search",
  },
  {
    label: props.showGroupMenu ? "关闭分组管理" : "分组管理",
    key: "group-menu",
  },
  {
    label: "刷新书架",
    key: "refresh",
    disabled: props.loading,
  },
  {
    label: "导入本地 TXT",
    key: "import-txt",
  },
  {
    label: "导入本地 CBZ",
    key: "import-cbz",
  },
  ...props.cardSizes.map((size) => ({
    label: `卡片大小：${size.label}`,
    key: `size-${size.key}`,
    disabled: props.activeSizeKey === size.key,
  })),
  {
    label: props.privacyModeEnabled ? "退出隐私模式" : "进入隐私模式",
    key: "privacy",
  },
  {
    label: "编辑书架",
    key: "edit",
  },
]);

function handleMobileMenuSelect(key: string) {
  if (key.startsWith("size-")) {
    emit("set-size", key.slice(5) as CardSizeKey);
    return;
  }
  switch (key) {
    case "search":
      emit("toggle-search");
      break;
    case "group-menu":
      emit("toggle-group-menu");
      break;
    case "refresh":
      emit("refresh");
      break;
    case "import-txt":
      emit("import-txt");
      break;
    case "import-cbz":
      emit("import-cbz");
      break;
    case "privacy":
      emit("toggle-privacy");
      break;
    case "edit":
      emit("toggle-edit");
      break;
  }
}
</script>

<template>
  <div class="bs-header">
    <div class="bs-header__row">
      <div>
        <h1 class="bs-header__title" style="float: left">书架</h1>
        <p class="bs-header__sub" style="float: left; padding-left: 20px; padding-top: 10px;">
          {{ privacyModeEnabled ? "隐私模式" : `${bookCount} 本书籍` }}
        </p>
      </div>
      <div class="bs-header__actions">
        <MobileToolbarMenu :options="mobileMenuOptions" @select="handleMobileMenuSelect">
          <button
            class="bs-icon-btn"
            type="button"
            title="搜索书架"
            aria-label="搜索书架"
            @click="emit('toggle-search')"
          >
            <Search :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            type="button"
            title="全文搜索"
            aria-label="全文搜索"
            @click="emit('toggle-global-search')"
          >
            <FileSearch :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            :class="{ 'bs-icon-btn--active': showGroupMenu }"
            type="button"
            title="分组管理"
            aria-label="分组管理"
            @click="emit('toggle-group-menu')"
          >
            <FolderPlus :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            :class="{ 'bs-icon-btn--spinning': loading }"
            type="button"
            title="刷新书架"
            aria-label="刷新书架"
            :disabled="loading"
            @click="emit('refresh')"
          >
            <RefreshCw :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            type="button"
            title="导入本地 TXT"
            aria-label="导入本地 TXT"
            @click="emit('import-txt')"
          >
            <FilePlus :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            type="button"
            title="导入本地 CBZ"
            aria-label="导入本地 CBZ"
            @click="emit('import-cbz')"
          >
            <ImagePlus :size="16" />
          </button>
          <n-dropdown
            trigger="click"
            :options="cardSizes.map((size) => ({ label: size.label, key: size.key }))"
            :value="activeSizeKey"
            @select="(key: string) => emit('set-size', key as CardSizeKey)"
          >
            <button
              class="bs-icon-btn"
              type="button"
              :title="`卡片大小（${activeSizeLabel}）`"
              aria-label="卡片大小"
            >
              <LayoutGrid :size="16" />
            </button>
          </n-dropdown>
          <button
            class="bs-icon-btn"
            :class="{ 'bs-icon-btn--active': privacyModeEnabled }"
            type="button"
            :title="privacyModeEnabled ? '退出隐私模式' : '进入隐私模式'"
            aria-label="隐私模式"
            @click="emit('toggle-privacy')"
          >
            <EyeOff v-if="privacyModeEnabled" :size="16" />
            <Eye v-else :size="16" />
          </button>
          <button
            class="bs-icon-btn"
            type="button"
            title="编辑书架"
            aria-label="编辑书架"
            @click="emit('toggle-edit')"
          >
            <Pencil :size="16" />
          </button>
        </MobileToolbarMenu>
      </div>
    </div>

    <div v-if="showGroupBar" class="bs-header__groups">
      <button
        v-for="group in enabledGroups"
        :key="group.id"
        class="bs-group-tag"
        :class="{ 'bs-group-tag--active': activeGroupId === group.id }"
        @click="emit('select-group', group.id)"
      >
        {{ group.name }}
      </button>
      <button
        class="bs-group-edit-btn"
        :class="{ 'bs-group-edit-btn--active': showGroupMenu }"
        type="button"
        title="编辑分组"
        aria-label="编辑分组"
        @click="emit('toggle-group-menu')"
      >
        <Settings2 :size="13" />
      </button>
    </div>

    <div class="bs-header__tags">
      <div class="bs-header__tags-list">
        <button
          v-for="tag in tags"
          :key="tag.id"
          class="bs-tag-chip"
          :class="{ 'bs-tag-chip--active': selectedTagIds.includes(tag.id) }"
          :style="{ '--tag-color': tag.color }"
          @click="emit('toggle-tag', tag.id)"
        >
          <Tag :size="11" />
          {{ tag.name }}
        </button>

        <div v-if="showTagInput" class="bs-tag-input-row">
          <input
            id="new-tag-name-input"
            v-model="newTagName"
            class="bs-tag-input"
            placeholder="标签名..."
            @keydown="handleTagKeydown"
            @blur="cancelTag"
          />
        </div>

        <button
          v-if="!showTagInput"
          class="bs-tag-chip bs-tag-chip--add"
          @click="openTagInput"
        >
          <Plus :size="12" />
        </button>
      </div>

      <button
        v-if="hasActiveTagFilter"
        class="bs-tag-clear-btn"
        @click="emit('clear-tag-filter')"
      >
        <X :size="12" />
        清除
      </button>
    </div>
  </div>
</template>

<style scoped>
.bs-header {
  flex-shrink: 0;
  padding: 24px 24px 8px;
}
.bs-header__row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}
.bs-header__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bs-header__title {
  font-size: var(--fs-20);
  font-weight: var(--fw-bold);
  color: var(--color-text);
  margin: 0 0 2px;
}
.bs-header__sub {
  font-size: var(--fs-13);
  color: var(--color-text-muted);
  margin: 0;
}
.bs-icon-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-1);
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    color var(--dur-fast) var(--ease-standard),
    border-color var(--dur-fast) var(--ease-standard),
    background var(--dur-fast) var(--ease-standard);
}
@media (hover: hover) and (pointer: fine) {
  .bs-icon-btn:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
}
.bs-icon-btn--active {
  color: var(--color-accent);
  border-color: var(--color-accent);
  background: color-mix(in srgb, var(--color-accent) 12%, transparent);
}
.bs-icon-btn--spinning svg {
  animation: bs-spin 0.8s linear infinite;
}
@keyframes bs-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
@media (pointer: coarse), (max-width: 640px) {
  .bs-header {
    padding: 16px 16px 6px;
  }
}

.bs-header__groups {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.bs-group-tag {
  display: inline-flex;
  align-items: center;
  padding: 6px 12px;
  font-size: var(--fs-13);
  color: var(--color-text-muted);
  background: var(--color-fill-secondary);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  cursor: pointer;
  transition:
    color var(--dur-fast) var(--ease-standard),
    border-color var(--dur-fast) var(--ease-standard),
    background var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-group-tag:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
}

.bs-group-tag--active {
  color: var(--color-accent);
  border-color: var(--color-accent);
  background: color-mix(in srgb, var(--color-accent) 12%, transparent);
}

@media (pointer: coarse), (max-width: 640px) {
  .bs-header__groups {
    gap: 6px;
    margin-top: 10px;
    padding-top: 10px;
    overflow-x: auto;
    flex-wrap: nowrap;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .bs-header__groups::-webkit-scrollbar {
    display: none;
  }

  .bs-group-tag {
    flex-shrink: 0;
    padding: 5px 10px;
    font-size: 12px;
  }
}

.bs-group-edit-btn {
  flex-shrink: 0;
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-1);
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    color var(--dur-fast) var(--ease-standard),
    border-color var(--dur-fast) var(--ease-standard),
    background var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-group-edit-btn:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
}

.bs-group-edit-btn--active {
  color: var(--color-accent);
  border-color: var(--color-accent);
  background: color-mix(in srgb, var(--color-accent) 12%, transparent);
}

.bs-header__tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border);
}

.bs-header__tags-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.bs-tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  background: var(--color-fill-secondary);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  cursor: pointer;
  transition:
    color var(--dur-fast) var(--ease-standard),
    border-color var(--dur-fast) var(--ease-standard),
    background var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-tag-chip:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
}

.bs-tag-chip--active {
  color: var(--color-on-accent, #fff);
  border-color: var(--tag-color);
  background: var(--tag-color);
}

.bs-tag-chip--add {
  width: 26px;
  height: 26px;
  padding: 0;
  justify-content: center;
  border-radius: 12px;
  border-style: dashed;
}

.bs-tag-input-row {
  display: flex;
  align-items: center;
}

.bs-tag-input {
  width: 80px;
  height: 26px;
  padding: 0 8px;
  font-size: var(--fs-12);
  border: 1px solid var(--color-accent);
  border-radius: 12px;
  background: var(--color-surface);
  color: var(--color-text);
  outline: none;
}

.bs-tag-clear-btn {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
  padding: 3px 8px;
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: transparent;
  cursor: pointer;
  transition:
    color var(--dur-fast) var(--ease-standard),
    border-color var(--dur-fast) var(--ease-standard);
}

@media (hover: hover) and (pointer: fine) {
  .bs-tag-clear-btn:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
}

@media (pointer: coarse), (max-width: 640px) {
  .bs-header__tags {
    overflow-x: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .bs-header__tags::-webkit-scrollbar {
    display: none;
  }

  .bs-header__tags-list {
    flex-wrap: nowrap;
  }

  .bs-tag-chip {
    flex-shrink: 0;
    padding: 3px 8px;
    font-size: 11px;
  }
}
</style>
