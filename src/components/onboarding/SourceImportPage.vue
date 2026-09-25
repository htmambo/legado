<script setup lang="ts">
import { ref } from 'vue'
import { installFromRepository } from '@/composables/useBookSource'

export interface SourceOption {
  name: string
  url: string
  description: string
}

const props = defineProps<{
  sources: SourceOption[]
}>()

const emit = defineEmits<{
  next: []
  skip: []
}>()

const selected = ref<Set<string>>(new Set())
const importing = ref(false)
const importProgress = ref(0)
const importTotal = ref(0)
const importError = ref('')

function toggleSource(name: string) {
  const next = new Set(selected.value)
  if (next.has(name)) {
    next.delete(name)
  } else {
    next.add(name)
  }
  selected.value = next
}

function selectAll() {
  selected.value = new Set((props.sources ?? []).map((s) => s.name))
}

async function handleImport() {
  const targets = (props.sources ?? []).filter((s) => selected.value.has(s.name))
  if (targets.length === 0) {
    emit('next')
    return
  }

  importing.value = true
  importError.value = ''
  importTotal.value = targets.length
  importProgress.value = 0

  for (const source of targets) {
    try {
      await installFromRepository(source.url, `${source.name}.js`)
      importProgress.value++
    } catch (e: unknown) {
      importError.value = e instanceof Error ? e.message : String(e)
    }
  }

  importing.value = false
  emit('next')
}
</script>

<template>
  <div class="source-import">
    <div class="source-import__header">
      <h2 class="source-import__title">导入书源</h2>
      <p class="source-import__desc">书源是获取内容的渠道，推荐以下精选书源</p>
    </div>

    <div class="source-import__toolbar">
      <button class="source-import__select-all" @click="selectAll">全选</button>
    </div>

    <div class="source-import__list">
      <label
        v-for="source in sources"
        :key="source.name"
        class="source-import__item"
        :class="{ 'source-import__item--selected': selected.has(source.name) }"
      >
        <input
          type="checkbox"
          class="source-import__checkbox"
          :checked="selected.has(source.name)"
          @change="toggleSource(source.name)"
        />
        <div class="source-import__item-content">
          <span class="source-import__item-name">{{ source.name }}</span>
          <span class="source-import__item-desc">{{ source.description }}</span>
        </div>
      </label>
    </div>

    <div v-if="importing" class="source-import__progress">
      <div class="source-import__progress-bar">
        <div
          class="source-import__progress-fill"
          :style="{ width: importTotal ? `${(importProgress / importTotal) * 100}%` : '0%' }"
        />
      </div>
      <span class="source-import__progress-text">{{ importProgress }} / {{ importTotal }}</span>
    </div>

    <div v-if="importError" class="source-import__error">{{ importError }}</div>

    <div class="source-import__actions">
      <button
        class="source-import__btn source-import__btn--primary"
        :disabled="importing"
        @click="handleImport"
      >
        {{ selected.size > 0 ? `一键导入 (${selected.size})` : '下一步' }}
      </button>
      <button class="source-import__skip" :disabled="importing" @click="$emit('skip')">跳过</button>
    </div>
  </div>
</template>

<style scoped>
.source-import {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  padding: var(--space-8) var(--space-6);
  background: var(--color-bg-page);
}

.source-import__header {
  text-align: center;
  margin-bottom: var(--space-6);
}

.source-import__title {
  font-size: var(--fs-24);
  font-weight: var(--fw-bold);
  color: var(--color-text);
  margin-bottom: var(--space-2);
}

.source-import__desc {
  font-size: var(--fs-14);
  color: var(--color-text-soft);
  line-height: var(--lh-relaxed);
}

.source-import__toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: var(--space-3);
}

.source-import__select-all {
  background: none;
  border: none;
  color: var(--color-accent);
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  cursor: pointer;
  padding: var(--space-1) var(--space-2);
}

.source-import__select-all:hover {
  text-decoration: underline;
}

.source-import__list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  flex: 1;
  overflow-y: auto;
  margin-bottom: var(--space-6);
}

.source-import__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  background: var(--color-surface);
  border: var(--border-thin) solid var(--color-border);
  border-radius: var(--radius-2);
  cursor: pointer;
  transition:
    border-color var(--dur-fast) var(--ease-standard),
    background var(--dur-fast) var(--ease-standard);
}

.source-import__item:hover {
  background: var(--color-hover);
}

.source-import__item--selected {
  border-color: var(--color-accent);
  background: var(--color-accent-soft);
}

.source-import__checkbox {
  width: 1.25rem;
  height: 1.25rem;
  accent-color: var(--color-accent);
  flex-shrink: 0;
  cursor: pointer;
}

.source-import__item-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.source-import__item-name {
  font-size: var(--fs-15);
  font-weight: var(--fw-medium);
  color: var(--color-text);
}

.source-import__item-desc {
  font-size: var(--fs-13);
  color: var(--color-text-muted);
  line-height: var(--lh-tight);
}

.source-import__progress {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.source-import__progress-bar {
  flex: 1;
  height: 6px;
  background: var(--color-border);
  border-radius: var(--radius-pill);
  overflow: hidden;
}

.source-import__progress-fill {
  height: 100%;
  background: var(--color-accent);
  border-radius: var(--radius-pill);
  transition: width var(--dur-base) var(--ease-standard);
}

.source-import__progress-text {
  font-size: var(--fs-13);
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.source-import__error {
  padding: var(--space-3);
  background: var(--color-danger-bg);
  color: var(--color-danger);
  border: var(--border-thin) solid var(--color-danger-border);
  border-radius: var(--radius-1);
  font-size: var(--fs-13);
  margin-bottom: var(--space-4);
}

.source-import__actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

.source-import__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  max-width: 320px;
  min-height: var(--control-lg);
  padding-inline: var(--space-5);
  border: none;
  border-radius: var(--radius-2);
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  cursor: pointer;
  transition:
    background var(--dur-fast) var(--ease-standard),
    opacity var(--dur-fast) var(--ease-standard);
}

.source-import__btn--primary {
  background: var(--color-accent);
  color: var(--color-accent-contrast);
}

.source-import__btn--primary:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.source-import__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.source-import__skip {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  cursor: pointer;
  padding: var(--space-2);
}

.source-import__skip:hover {
  color: var(--color-text-soft);
}

.source-import__skip:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>