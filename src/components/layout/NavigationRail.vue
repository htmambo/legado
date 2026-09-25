<script setup lang="ts">
import type { Component } from "vue";
import { BookOpen, Compass, Search, LayoutGrid, Package, SlidersHorizontal, Bell } from "lucide-vue-next";
import type { NavItem } from "./types";

const props = withDefaults(
  defineProps<{
    items?: NavItem[];
    activeId?: string;
    version?: string;
    unreadCount?: number;
  }>(),
  {
    items: () => [],
    activeId: "",
    version: "",
    unreadCount: 0,
  },
);

const emit = defineEmits<{
  select: [id: string];
  notification: [];
}>();

const ICON_COMPONENTS: Record<string, Component> = {
  bookshelf: BookOpen,
  explore: Compass,
  search: Search,
  booksource: LayoutGrid,
  extensions: Package,
  settings: SlidersHorizontal,
};
</script>

<template>
  <nav class="nav-rail" role="navigation" :aria-label="'主导航'">
    <div class="nav-rail__header">
      <div class="nav-rail__logo">
        <BookOpen :size="24" :stroke-width="1.75" />
        <span class="nav-rail__app-name">Legado</span>
      </div>
    </div>

    <ul class="nav-rail__list">
      <li
        v-for="item in items"
        :key="item.id"
        class="nav-rail__item"
        :class="{ 'nav-rail__item--active': activeId === item.id }"
      >
        <button
          class="nav-rail__btn"
          :aria-label="item.label"
          :aria-selected="activeId === item.id"
          role="tab"
          @click="emit('select', item.id)"
        >
          <span class="nav-rail__indicator" aria-hidden="true" />
          <span class="nav-rail__icon">
            <component :is="ICON_COMPONENTS[item.icon]" :size="24" :stroke-width="1.75" />
            <span v-if="item.badge" class="nav-rail__badge">{{ item.badge }}</span>
          </span>
          <span class="nav-rail__label">{{ item.label }}</span>
        </button>
      </li>
      <li
        class="nav-rail__item"
        :class="{ 'nav-rail__item--active': activeId === 'updateFeed' }"
      >
        <button
          class="nav-rail__btn"
          aria-label="更新通知"
          :aria-selected="activeId === 'updateFeed'"
          role="tab"
          @click="emit('notification')"
        >
          <span class="nav-rail__indicator" aria-hidden="true" />
          <span class="nav-rail__icon">
            <Bell :size="24" :stroke-width="1.75" />
            <span v-if="unreadCount > 0" class="nav-rail__badge">{{ unreadCount > 99 ? '99+' : unreadCount }}</span>
          </span>
          <span class="nav-rail__label">更新</span>
        </button>
      </li>
    </ul>

    <div v-if="version" class="nav-rail__footer">
      <span class="nav-rail__version">v{{ version }}</span>
    </div>
  </nav>
</template>

<style scoped>
.nav-rail {
  grid-area: navrail;
  display: flex;
  flex-direction: column;
  width: 72px;
  height: 100%;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  overflow: hidden;
  white-space: nowrap;
  transition: width 0.3s ease;
  user-select: none;
  flex-shrink: 0;
}

.nav-rail:hover {
  z-index: 999999;
  width: 150px;
}

.nav-rail__header {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 16px;
  flex-shrink: 0;
}

.nav-rail__logo {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--color-accent);
}

.nav-rail__app-name {
  font-size: 1.125rem;
  font-weight: 700;
  opacity: 0;
  transition: opacity 0.3s ease;
  flex-shrink: 0;
}

.nav-rail:hover .nav-rail__app-name {
  opacity: 1;
}

.nav-rail__list {
  flex: 1;
  list-style: none;
  margin: 0;
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
  overflow-x: hidden;
}

.nav-rail__item {
  position: relative;
}

.nav-rail__btn {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 200px;
  height: 56px;
  padding: 0 16px;
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: background 0.15s;
  font-family: inherit;
  text-align: left;
}

.nav-rail__btn:hover {
  background: var(--color-hover);
}

.nav-rail__indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  border-radius: 0 3px 3px 0;
  background: var(--color-accent);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.nav-rail__item--active .nav-rail__indicator {
  opacity: 1;
}

.nav-rail__item--active .nav-rail__btn {
  color: var(--color-accent);
  background: rgba(var(--color-accent-rgb, 99 102 241), 0.1);
}

.nav-rail__icon {
  position: relative;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-rail__badge {
  position: absolute;
  top: -4px;
  right: -6px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 8px;
  background: var(--color-error);
  color: #fff;
  font-size: 0.625rem;
  font-weight: 600;
  line-height: 16px;
  text-align: center;
}

.nav-rail__label {
  font-size: 0.875rem;
  font-weight: 500;
  opacity: 0;
  transition: opacity 0.3s ease;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.nav-rail:hover .nav-rail__label {
  opacity: 1;
}

.nav-rail__footer {
  flex-shrink: 0;
  padding: 12px 16px;
}

.nav-rail__version {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.nav-rail:hover .nav-rail__version {
  opacity: 1;
}

@media (max-width: 750px) {
  .nav-rail {
    display: none;
  }
}
</style>
