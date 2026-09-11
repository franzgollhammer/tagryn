<script setup lang="ts">
const props = defineProps<{
  value: number;
  min: number;
  max: number;
  reverse?: boolean;
  label: string;
}>();
const emit = defineEmits<{ resize: [value: number] }>();
function resize(value: number) {
  emit('resize', Math.max(props.min, Math.min(props.max, value)));
}
function start(event: PointerEvent) {
  const origin = event.clientX;
  const value = props.value;
  const move = (e: PointerEvent) =>
    resize(value + (e.clientX - origin) * (props.reverse ? -1 : 1));
  const stop = () => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', stop);
    document.body.classList.remove('resizing');
  };
  document.body.classList.add('resizing');
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', stop, { once: true });
}
</script>
<template>
  <div
    class="pane-divider"
    tabindex="0"
    role="separator"
    aria-orientation="vertical"
    :aria-label="label"
    :aria-valuemin="min"
    :aria-valuemax="max"
    :aria-valuenow="value"
    @pointerdown.prevent="start"
    @keydown.left.prevent="resize(value - (reverse ? -12 : 12))"
    @keydown.right.prevent="resize(value + (reverse ? -12 : 12))"
  />
</template>
