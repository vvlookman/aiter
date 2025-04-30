<script setup>
import { useAiStore } from '@/stores/ai';
import { useDocStore } from '@/stores/doc';
import { computed } from 'vue';

const visible = defineModel('visible');
const aiStore = useAiStore();
const docStore = useDocStore();

const queue = computed(() => docStore.getLearnQueue(aiStore.getActiveName()));

const onDelete = (task) => {
  docStore.deleteFromLearnQueue(aiStore.getActiveName(), task.hash);
};
</script>

<template>
  <el-dialog class="!md:w-2/3 !xl:w-1/2 !w-5/6" v-model="visible" destroy-on-close>
    <div class="max-h-50vh overflow-auto">
      <div class="mt-4 flex w-full items-center gap-4" v-for="(task, i) in queue" :key="task.hash">
        <span class="break-all">
          {{ task.file.name }}
        </span>
        <span class="flex-1"></span>
        <el-button v-if="task.processing" link disabled>
          <el-icon class="rotating"><i class="ri-loader-4-line"></i></el-icon>
        </el-button>
        <el-button v-else @click="onDelete(task)" link type="danger">
          <el-icon><i class="ri-delete-bin-6-line"></i></el-icon>
        </el-button>
      </div>
    </div>
  </el-dialog>
</template>
