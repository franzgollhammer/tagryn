<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { X } from '@lucide/vue';
defineProps<{ title: string; wide?: boolean }>();
const emit = defineEmits<{ close: [] }>();
const element = ref<HTMLDialogElement>();
const previous = document.activeElement;
onMounted(() => element.value?.showModal());
onBeforeUnmount(() => {
  element.value?.close();
  if (previous instanceof HTMLElement) previous.focus();
});
</script>
<template>
  <dialog
    ref="element"
    class="modal"
    :class="{ wide }"
    aria-labelledby="modal-heading"
    @cancel.prevent="emit('close')"
    @click="
      (event) => {
        if (event.target === element) emit('close');
      }
    "
  >
    <header class="modal-header">
      <h2 id="modal-heading">{{ title }}</h2>
      <button
        class="icon-button"
        aria-label="Schließen / Close"
        @click="emit('close')"
      >
        <X :size="18" />
      </button>
    </header>
    <div class="modal-body"><slot /></div>
    <footer v-if="$slots.footer" class="modal-footer">
      <slot name="footer" />
    </footer>
  </dialog>
</template>
