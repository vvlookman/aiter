<script setup>
import { useAiStore } from '@/stores/ai';

const visible = defineModel('visible');
const aiStore = useAiStore();

const onSwitch = (id) => {
  aiStore.active(id);
  visible.value = false;
};
</script>

<template>
  <el-dialog v-model="visible" :title="$t('label.switch_ai')" destroy-on-close :show-close="false">
    <div class="max-h-50vh my-4 overflow-auto">
      <el-button class="!ml-0 mr-4 mt-4" @click="onSwitch()" round :type="!aiStore.activeId ? 'primary' : ''">
        {{ $t('label.default_ai') }}
      </el-button>

      <el-button
        class="!ml-0 mr-4 mt-4"
        v-for="ai in aiStore.items"
        :key="ai.id"
        @click="onSwitch(ai.id)"
        round
        :type="ai.id == aiStore.activeId ? 'primary' : ''"
      >
        {{ ai.name }}
      </el-button>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button type="primary" @click="visible = false">{{ $t('label.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
