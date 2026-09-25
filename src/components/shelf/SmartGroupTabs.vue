<script setup lang="ts">
import { computed } from "vue";
import type { SmartGroupType } from "@/stores/smartGroups";
import { useSmartGroupsStore } from "@/stores/smartGroups";

const emit = defineEmits<{
  (e: "select-group", groupId: SmartGroupType | null): void;
}>();

const props = defineProps<{
  activeGroup: SmartGroupType | null;
}>();

const smartGroupsStore = useSmartGroupsStore();
const { groups, groupCounts } = smartGroupsStore;

const tabs = computed(() => {
  const result: { id: SmartGroupType | null; label: string; count: number }[] = [
    { id: null, label: "全部", count: Object.values(groupCounts).reduce((a, b) => a + b, 0) },
  ];
  for (const group of groups) {
    result.push({ id: group.id, label: group.label, count: groupCounts[group.id] });
  }
  return result;
});

function selectTab(groupId: SmartGroupType | null) {
  if (props.activeGroup === groupId) return;
  emit("select-group", groupId);
}
</script>

<template>
  <div class="smart-group-tabs">
    <div class="smart-group-tabs__scroll">
      <button
        v-for="tab in tabs"
        :key="tab.id ?? 'all'"
        class="smart-group-tabs__tab"
        :class="{ 'smart-group-tabs__tab--active': activeGroup === tab.id }"
        @click="selectTab(tab.id)"
      >
        <span class="smart-group-tabs__label">{{ tab.label }}</span>
        <span class="smart-group-tabs__count">{{ tab.count }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.smart-group-tabs {
  flex-shrink: 0;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.smart-group-tabs__scroll {
  display: flex;
  gap: 0;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  padding: 0 var(--space-4);
}

.smart-group-tabs__scroll::-webkit-scrollbar {
  display: none;
}

.smart-group-tabs__tab {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  background: none;
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  color: var(--color-text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition: color var(--dur-fast) var(--ease-standard);
  font-family: var(--font-ui);
  flex-shrink: 0;
}

.smart-group-tabs__tab::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 16px;
  right: 16px;
  height: 2px;
  background: var(--brand-500);
  border-radius: 2px 2px 0 0;
  transform: scaleX(0);
  transition: transform var(--dur-fast) var(--ease-standard);
}

.smart-group-tabs__tab--active {
  color: var(--brand-500);
}

.smart-group-tabs__tab--active::after {
  transform: scaleX(1);
}

.smart-group-tabs__count {
  font-size: var(--fs-11);
  font-weight: var(--fw-semibold);
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  background: var(--color-active);
  color: var(--color-text-muted);
  min-width: 18px;
  text-align: center;
  line-height: 1.4;
}

.smart-group-tabs__tab--active .smart-group-tabs__count {
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

@media (hover: hover) and (pointer: fine) {
  .smart-group-tabs__tab:hover {
    color: var(--color-text);
  }

  .smart-group-tabs__tab--active:hover {
    color: var(--brand-500);
  }
}
</style>