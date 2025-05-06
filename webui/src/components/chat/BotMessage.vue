<script setup>
import { renderMarkdown } from '@/utils/markdown';
import { extractThinkBlock, removeThinkBlock } from '@/utils/text';
import { ElMessage } from 'element-plus';
import { computed, reactive } from 'vue';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['deleteExchange']);
const props = defineProps({
  message: Object,
  deleting: Boolean,

  waiting: Boolean,
  receiving: Boolean,
});
const { t } = useI18n();

const state = reactive({
  showTools: false,
});

const markdownContent = computed(() => renderMarkdown(removeThinkBlock(props.message.content)));
const markdownReasoning = computed(() =>
  renderMarkdown(props.message.reasoning + extractThinkBlock(props.message.content)),
);

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

const tooltipCallTool = (call_tool) => {
  let tooltip = call_tool.task?.description;

  if (Object.keys(call_tool.task?.parameters).length > 0) {
    tooltip += ` ( ${Object.entries(call_tool.task.parameters)
      .map(([k, v]) => `${k}=${v}`)
      .join(', ')} )`;
  }

  return tooltip;
};
</script>

<template>
  <div class="flex">
    <el-avatar class="bg-primary" size="small"><i class="ri-robot-3-line"></i></el-avatar>
    <div class="ml-4 flex-1" @mouseenter="state.showTools = true" @mouseleave="state.showTools = false">
      <template v-if="waiting">
        <el-icon class="rotating"><i class="ri-loader-3-line"></i></el-icon>
      </template>
      <template v-else>
        <template v-if="!receiving && !message.content && !message.reasoning">
          <el-text type="info">{{ $t('message.empty_bot_message') }}</el-text>
        </template>
        <template v-else>
          <div class="mb-2" v-if="message.call_tools?.length > 0">
            <el-tooltip
              v-for="call_tool in message.call_tools"
              :content="tooltipCallTool(call_tool)"
              placement="top"
              trigger="click"
              popper-class="max-w-20em md:max-w-40em xl:max-w-60em break-all"
              :key="call_tool.task.id"
            >
              <div
                class="border-1 mb-1 w-fit max-w-full cursor-pointer rounded border-gray-200 bg-gray-100 px-2 py-1 text-xs text-gray-400"
              >
                <el-icon class="mr-2"><i class="ri-hammer-line"></i></el-icon>
                <span class="flex-1">{{ call_tool.task.description }}</span>
                <span class="ml-2">
                  <template v-if="!call_tool.time">
                    <el-icon class="rotating"><i class="ri-loader-3-line"></i></el-icon>
                  </template>
                  <template v-else>
                    <el-icon><i class="ri-check-line"></i></el-icon>
                  </template>
                </span>
              </div>
            </el-tooltip>
          </div>

          <div
            class="message-content mb-2 break-all border-0 border-l-2 border-solid border-gray-200 pl-2 text-gray-400"
            v-if="markdownReasoning"
            v-html="markdownReasoning"
          ></div>

          <div class="message-content break-all" v-if="markdownContent" v-html="markdownContent"></div>
        </template>

        <div class="mt-1 flex" :style="{ opacity: state.showTools && !receiving && message.content ? 1 : 0 }">
          <el-button @click="onCopy" size="small">
            <el-icon class="el-icon--left"><i class="ri-file-copy-line"></i></el-icon>
            {{ $t('label.copy') }}
          </el-button>
          <el-button v-if="message.exchange" @click="onDeleteExchange" size="small">
            <template v-if="deleting">
              <el-icon class="rotating"><i class="ri-loader-3-line"></i></el-icon>
            </template>
            <template v-else>
              <el-icon><i class="ri-delete-bin-2-line"></i></el-icon>
            </template>
          </el-button>
        </div>
      </template>
    </div>
  </div>
</template>
