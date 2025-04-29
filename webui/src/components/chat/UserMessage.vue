<script setup>
import { renderMarkdown } from '@/utils/markdown';
import { ElMessage } from 'element-plus';
import { computed, reactive } from 'vue';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['deleteExchange', 'reuse']);
const props = defineProps({
  message: Object,
  deleting: Boolean,
});
const { t } = useI18n();

const state = reactive({
  showTools: false,
});

const markdownContent = computed(() => renderMarkdown(props.message.content));

const onCopy = async () => {
  await navigator.clipboard.writeText(props.message.content);

  ElMessage({
    type: 'success',
    message: t('message.success_copy'),
  });
};

const onDeleteExchange = () => {
  emit('deleteExchange', { exchange: props.message.exchange });
};

const onReuse = () => {
  emit('reuse', { content: props.message.content });
};
</script>

<template>
  <div class="flex flex-col items-end" @mouseenter="state.showTools = true" @mouseleave="state.showTools = false">
    <div class="rounded-md bg-neutral-700 px-3 py-2 text-sm text-white">
      <div class="break-all" v-html="markdownContent"></div>
    </div>

    <div class="mt-1 flex" :style="{ opacity: state.showTools && message.content ? 1 : 0 }">
      <el-button v-if="message.exchange" @click="onDeleteExchange" size="small">
        <template v-if="deleting">
          <el-icon class="rotating"><i class="ri-loader-3-line"></i></el-icon>
        </template>
        <template v-else>
          <el-icon><i class="ri-delete-bin-2-line"></i></el-icon>
        </template>
      </el-button>
      <el-button @click="onCopy" size="small">
        <el-icon class="el-icon--left"><i class="ri-file-copy-line"></i></el-icon>
        {{ $t('label.copy') }}
      </el-button>
      <el-button @click="onReuse" size="small">
        <el-icon class="el-icon--left"><i class="ri-repeat-line"></i></el-icon>
        {{ $t('label.reuse') }}
      </el-button>
    </div>
  </div>
</template>
