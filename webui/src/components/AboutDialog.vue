<script setup>
import { onMounted, reactive } from 'vue';

import api from '@/api';

const visible = defineModel('visible');

const state = reactive({
  version: '',
});

onMounted(async () => {
  state.version = await api.get('/version');
});
</script>

<template>
  <el-dialog v-model="visible" :show-close="false" destroy-on-close>
    <template #header>
      <div>{{ $t('label.about') }}</div>
    </template>

    <div class="text-l text-center">
      <img class="h-4em" src="@/assets/logo.png" />
      <div>
        <el-text size="large" type="primary">Aiter {{ !state.version ? '' : `v${state.version}` }}</el-text>
      </div>
      <div>
        <el-text size="small" type="info">{{ $t('label.slogan') }}</el-text>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="visible = false" type="primary"><i class="ri-check-line"></i></el-button>
      </div>
    </template>
  </el-dialog>
</template>
