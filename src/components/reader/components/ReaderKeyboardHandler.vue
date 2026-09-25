<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useReaderActionsStore } from "@/features/reader/stores/readerActions";

export interface KeyboardHandlerOptions {
  enabled?: boolean;
  onOpenSearch?: () => void;
  onToggleBookmark?: () => void;
  onAddHighlight?: () => void;
}

const props = withDefaults(defineProps<KeyboardHandlerOptions>(), {
  enabled: true,
});

const emit = defineEmits<{
  (e: "escape"): void;
}>();

const readerActions = useReaderActionsStore();

function handleKeyDown(event: KeyboardEvent) {
  if (!props.enabled) return;

  const target = event.target as HTMLElement;
  const isInputLike =
    target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable;

  if (isInputLike) return;

  const isCtrlOrCmd = event.ctrlKey || event.metaKey;

  switch (event.key) {
    case "ArrowLeft":
    case "ArrowUp":
      // 阻止全局焦点导航（useFocusNavigation）也消费方向键
      event.preventDefault();
      event.stopPropagation();
      // 按"页"前进：paged 模式翻页，scroll 模式滚一页，comic / video no-op
      readerActions.volumePagePrev();
      break;

    case "ArrowRight":
    case "ArrowDown":
      event.preventDefault();
      event.stopPropagation();
      readerActions.volumePageNext();
      break;

    case "PageUp":
      event.preventDefault();
      readerActions.gotoPrevBoundary();
      break;

    case "PageDown":
      event.preventDefault();
      readerActions.gotoNextBoundary();
      break;

    case "Home":
      event.preventDefault();
      readerActions.gotoPrevBoundary();
      break;

    case "End":
      event.preventDefault();
      readerActions.gotoNextBoundary();
      break;

    case " ":
      event.preventDefault();
      readerActions.gotoNextBoundary();
      break;

    case "Enter":
      event.preventDefault();
      readerActions.gotoNextBoundary();
      break;

    case "Escape":
      event.preventDefault();
      emit("escape");
      readerActions.close();
      break;

    case "f":
    case "F":
      if (isCtrlOrCmd) {
        event.preventDefault();
        props.onOpenSearch?.();
      }
      break;

    case "b":
    case "B":
      if (isCtrlOrCmd) {
        event.preventDefault();
        props.onAddHighlight?.();
      } else {
        event.preventDefault();
        props.onToggleBookmark?.();
      }
      break;

    default:
      break;
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown, { capture: true });
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown, { capture: true });
});
</script>

<template>
  <slot />
</template>
