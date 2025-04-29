<script setup>
const emit = defineEmits(['select']);
defineProps({
  active: Boolean,
});
</script>

<template>
  <div
    class="border-1 flex cursor-pointer items-center rounded-lg px-4 py-2 hover:bg-neutral-100"
    @click="emit('select')"
    :class="active ? 'border-neutral-200 bg-neutral-100' : 'border-transparent'"
  >
    <slot></slot>
  </div>
</template>
