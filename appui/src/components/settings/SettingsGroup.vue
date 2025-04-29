<script setup>
defineProps({
  title: String,
});
</script>

<template>
  <div class="border-1 rounded-lg border-neutral-300 bg-white p-4">
    <div class="font-bold" v-if="title">{{ title }}</div>
    <slot></slot>
  </div>
</template>
