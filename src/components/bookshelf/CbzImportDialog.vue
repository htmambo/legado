<script setup lang="ts">
import { Upload, BookOpen, X, Check, Image } from "lucide-vue-next"
import { NModal, NCard, NButton, NInput, NSpin, NProgress, NAlert } from "naive-ui"
import { ref, computed, watch } from "vue"
import { useOverlayBackstack } from "@/composables/useOverlayBackstack"
import { useCbzImporter } from "@/composables/useCbzImporter"

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", value: boolean): void
  (
    e: "imported",
    payload: {
      title: string
      pages: string[]
      coverUrl: string
    },
  ): void
}>()

type Phase = "upload" | "extracting" | "preview" | "importing" | "done"

const { progress, errorMsg, importCbz } = useCbzImporter()

const phase = ref<Phase>("upload")
const fileName = ref("")
const bookTitle = ref("")
const pages = ref<string[]>([])
const coverUrl = ref("")
const importProgress = ref(0)
const isDragOver = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

function extractTitle(name: string): string {
  return name.replace(/\.cbz$/i, "").trim()
}

async function handleFile(file: File) {
  if (!file.name.toLowerCase().endsWith(".cbz")) {
    errorMsg.value = "仅支持 .cbz 格式的文件"
    return
  }
  if (file.size > 500 * 1024 * 1024) {
    errorMsg.value = "文件过大（最大支持 500 MB）"
    return
  }
  errorMsg.value = ""

  fileName.value = file.name
  bookTitle.value = extractTitle(file.name)
  phase.value = "extracting"
  progress.value = 0

  const result = await importCbz(file, (pct) => {
    progress.value = pct
  })

  if (result.pages.length === 0) {
    errorMsg.value = errorMsg.value || "未找到可用的图片"
    phase.value = "upload"
    return
  }

  pages.value = result.pages
  coverUrl.value = result.coverUrl
  phase.value = "preview"
}

// ── 拖拽 / 文件选择 ───────────────────────────────────────────────────────

function onDragOver(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = true
}
function onDragLeave() {
  isDragOver.value = false
}
async function onDrop(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = false
  const file = e.dataTransfer?.files[0]
  if (file) {
    await handleFile(file)
  }
}
function onClickUpload() {
  fileInputRef.value?.click()
}
async function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) {
    await handleFile(file)
  }
  input.value = ""
}

// ── 导入 ─────────────────────────────────────────────────────────────────

async function doImport() {
  if (!pages.value.length || !bookTitle.value.trim()) {
    return
  }

  phase.value = "importing"
  importProgress.value = 0

  await new Promise<void>((resolve) => setTimeout(resolve, 16))
  importProgress.value = 30

  emit("imported", {
    title: bookTitle.value.trim(),
    pages: pages.value,
    coverUrl: coverUrl.value,
  })

  importProgress.value = 100
  // 不再直接切到 "done"——父组件持久化完成后会调 ack()/fail(msg)
}

// ── 关闭 & 重置 ───────────────────────────────────────────────────────────

function close() {
  if (phase.value === "extracting" || phase.value === "importing") {
    return
  }
  emit("update:show", false)
}

function reset() {
  phase.value = "upload"
  fileName.value = ""
  bookTitle.value = ""
  pages.value = []
  coverUrl.value = ""
  errorMsg.value = ""
  importProgress.value = 0
}

watch(
  () => props.show,
  (v) => {
    if (v) {
      reset()
    }
  },
)

useOverlayBackstack(() => props.show && phase.value !== "extracting" && phase.value !== "importing", close)

const canClose = computed(() => phase.value !== "extracting" && phase.value !== "importing")

const pageCountLabel = computed(() => `${pages.value.length} 页`)

// ── 父组件调用：告知持久化结果 ────────────────────────────────────────────

defineExpose({
  ack() {
    phase.value = "done";
  },
  fail(message: string) {
    errorMsg.value = message;
    phase.value = "preview";
    importProgress.value = 0;
  },
});
</script>

<template>
  <NModal
    :show="props.show"
    :mask-closable="canClose"
    @update:show="
      (v) => {
        if (!v && canClose) close()
      }
    "
  >
    <NCard
      class="cbz-import-dialog"
      :title="phase === 'done' ? '导入完成' : '导入本地 CBZ'"
      :bordered="false"
      role="dialog"
    >
      <template #header-extra>
        <button
          v-if="canClose"
          class="cbz-import-dialog__close"
          type="button"
          aria-label="关闭"
          @click="close"
        >
          <X :size="16" />
        </button>
      </template>

      <!-- 上传区域 -->
      <div v-if="phase === 'upload'" class="cbz-import-dialog__body">
        <div
          class="cbz-upload-zone"
          :class="{ 'cbz-upload-zone--over': isDragOver }"
          role="button"
          tabindex="0"
          @click="onClickUpload"
          @keydown.enter="onClickUpload"
          @keydown.space.prevent="onClickUpload"
          @dragover="onDragOver"
          @dragleave="onDragLeave"
          @drop="onDrop"
        >
          <Upload :size="40" class="cbz-upload-zone__icon" />
          <p class="cbz-upload-zone__hint">点击或拖拽 CBZ 文件到此处</p>
          <p class="cbz-upload-zone__sub">支持 CBZ（ZIP 压缩包），最大 500 MB</p>
        </div>
        <input
          ref="fileInputRef"
          type="file"
          accept=".cbz,application/zip,application/vnd.comicbook+zip"
          class="cbz-import-dialog__file-input"
          @change="onFileChange"
        />
        <NAlert v-if="errorMsg" type="error" :bordered="false" class="cbz-import-dialog__error">
          {{ errorMsg }}
        </NAlert>
      </div>

      <!-- 提取中 -->
      <div v-else-if="phase === 'extracting'" class="cbz-import-dialog__loading">
        <NSpin size="large" />
        <p class="cbz-import-dialog__loading-text">正在提取 CBZ 图片，请稍候…</p>
        <NProgress
          type="line"
          :percentage="progress"
          :indicator-placement="'inside'"
          class="cbz-import-dialog__progress"
        />
      </div>

      <!-- 预览 -->
      <div v-else-if="phase === 'preview'" class="cbz-import-dialog__body">
        <div class="cbz-import-dialog__meta">
          <div class="cbz-import-dialog__meta-row">
            <label class="cbz-import-dialog__label">书名</label>
            <NInput v-model:value="bookTitle" placeholder="书名" />
          </div>
        </div>

        <div v-if="coverUrl" class="cbz-import-dialog__cover">
          <p class="cbz-import-dialog__section-title">封面预览</p>
          <div class="cbz-cover-preview">
            <img :src="coverUrl" alt="封面预览" class="cbz-cover-preview__img" />
            <div class="cbz-cover-preview__badge">
              <Image :size="12" />
              {{ pageCountLabel }}
            </div>
          </div>
        </div>

        <div class="cbz-import-dialog__footer">
          <NButton @click="reset">重新选择</NButton>
          <NButton type="primary" :disabled="!bookTitle.trim() || pages.length === 0" @click="doImport">
            <BookOpen :size="14" style="margin-right: 4px" />
            导入书架（{{ pageCountLabel }}）
          </NButton>
        </div>
      </div>

      <!-- 导入中 -->
      <div v-else-if="phase === 'importing'" class="cbz-import-dialog__loading">
        <NSpin size="large" />
        <p class="cbz-import-dialog__loading-text">正在导入书架，请稍候…</p>
        <NProgress
          type="line"
          :percentage="importProgress"
          :indicator-placement="'inside'"
          class="cbz-import-dialog__progress"
        />
      </div>

      <!-- 完成 -->
      <div v-else-if="phase === 'done'" class="cbz-import-dialog__done">
        <Check :size="48" class="cbz-import-dialog__done-icon" />
        <p class="cbz-import-dialog__done-text">《{{ bookTitle }}》已成功加入书架</p>
        <NButton type="primary" @click="close">完成</NButton>
      </div>
    </NCard>
  </NModal>
</template>

<style scoped>
.cbz-import-dialog {
  width: min(540px, 92vw);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
}

.cbz-import-dialog :deep(.n-card__content) {
  overflow-y: auto;
  flex: 1;
}

.cbz-import-dialog__close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted, #888);
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}
.cbz-import-dialog__close:hover {
  color: var(--color-text, #333);
  background: var(--color-hover, rgba(0, 0, 0, 0.06));
}

.cbz-import-dialog__body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.cbz-import-dialog__file-input {
  display: none;
}

/* ── 上传区 ── */
.cbz-upload-zone {
  border: 2px dashed var(--color-border, #e0e0e0);
  border-radius: 12px;
  padding: 48px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition:
    border-color 0.2s,
    background 0.2s;
  user-select: none;
  outline: none;
}
.cbz-upload-zone:hover,
.cbz-upload-zone:focus {
  border-color: var(--primary-color, #18a058);
  background: var(--primary-color-suppl, rgba(24, 160, 88, 0.04));
}
.cbz-upload-zone--over {
  border-color: var(--primary-color, #18a058);
  background: var(--primary-color-suppl, rgba(24, 160, 88, 0.08));
}
.cbz-upload-zone__icon {
  color: var(--color-text-muted, #aaa);
}
.cbz-upload-zone--over .cbz-upload-zone__icon,
.cbz-upload-zone:hover .cbz-upload-zone__icon {
  color: var(--primary-color, #18a058);
}
.cbz-upload-zone__hint {
  font-size: var(--fs-15, 15px);
  font-weight: var(--fw-medium, 500);
  color: var(--color-text, #333);
  margin: 0;
}
.cbz-upload-zone__sub {
  font-size: var(--fs-13, 13px);
  color: var(--color-text-muted, #888);
  margin: 0;
}

/* ── 元信息 ── */
.cbz-import-dialog__meta {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.cbz-import-dialog__meta-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.cbz-import-dialog__label {
  font-size: var(--fs-13, 13px);
  color: var(--color-text-muted, #888);
  width: 32px;
  flex-shrink: 0;
}

/* ── 封面预览 ── */
.cbz-import-dialog__cover {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.cbz-import-dialog__section-title {
  font-size: var(--fs-13, 13px);
  font-weight: var(--fw-medium, 500);
  color: var(--color-text-muted, #888);
  margin: 0;
}
.cbz-cover-preview {
  position: relative;
  width: 100%;
  max-height: 320px;
  overflow: hidden;
  border-radius: 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-fill-secondary, #f5f5f5);
}
.cbz-cover-preview__img {
  width: 100%;
  height: auto;
  max-height: 320px;
  object-fit: contain;
}
.cbz-cover-preview__badge {
  position: absolute;
  bottom: 8px;
  right: 8px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: var(--fs-12, 12px);
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  border-radius: 6px;
}

/* ── 底部按钮 ── */
.cbz-import-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 4px;
}

.cbz-import-dialog__error {
  margin-top: 4px;
}

/* ── 加载中 ── */
.cbz-import-dialog__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px 0;
}
.cbz-import-dialog__loading-text {
  color: var(--color-text-muted, #888);
  margin: 0;
}
.cbz-import-dialog__progress {
  width: 100%;
}

/* ── 完成 ── */
.cbz-import-dialog__done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px 0;
}
.cbz-import-dialog__done-icon {
  color: var(--success-color, #18a058);
}
.cbz-import-dialog__done-text {
  font-size: var(--fs-15, 15px);
  font-weight: var(--fw-medium, 500);
  color: var(--color-text, #333);
  margin: 0;
  text-align: center;
}
</style>