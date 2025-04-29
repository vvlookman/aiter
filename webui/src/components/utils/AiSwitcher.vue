<script setup>
import { useAiStore } from '@/stores/ai';
import { onMounted } from 'vue';

const aiStore = useAiStore();

const onSwitch = (id) => {
  if (id === '~') {
    aiStore.activeId = null;
    localStorage.removeItem('aiter-ai');
  } else {
    aiStore.activeId = id;
    localStorage.setItem('aiter-ai', id);
  }
};

const onSwitchVisibleChange = async (visible) => {
  if (visible) {
    await aiStore.fetch();
  }
};

onMounted(async () => {
  await aiStore.fetch();
  aiStore.activeId = localStorage.getItem('aiter-ai');
});
</script>

<template>
  <el-dropdown @command="onSwitch" @visible-change="onSwitchVisibleChange" placement="bottom-end" trigger="click">
    <el-button type="primary">
      <el-icon class="el-icon--left"><i class="ri-robot-3-line"></i></el-icon>
      <span class="pl-2">{{ aiStore.getActiveName() ?? $t('label.default_ai') }}</span>
    </el-button>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="~">
          {{ $t('label.default_ai') }}
          <span class="flex-1"></span>
          <el-icon class="el-icon--right" v-if="aiStore.activeId == null"><i class="ri-check-line"></i></el-icon>
        </el-dropdown-item>
        <el-dropdown-item v-for="ai in aiStore.items" :command="ai.id" :key="ai.id">
          {{ ai.name }}
          <el-icon class="el-icon--right" v-if="ai.id == aiStore.activeId"><i class="ri-check-line"></i></el-icon>
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>
