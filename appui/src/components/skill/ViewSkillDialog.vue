<script setup>
import { callToolGet } from '@/call';
import { formatToolParameters } from '@/utils/tool';
import { reactive, watch } from 'vue';

const emit = defineEmits(['delete']);
const visible = defineModel('visible');
const props = defineProps({
  skill: Object,
});

const state = reactive({
  tool: null,
  fetchingTool: false,
});

const getTool = async () => {
  state.fetchingTool = true;

  try {
    state.tool = await callToolGet(props.skill.tool_id);
  } finally {
    state.fetchingTool = false;
  }
};

const onDelete = () => {
  emit('delete');

  visible.value = false;
};

watch(
  () => props.skill,
  async () => {
    await getTool();
  },
);
</script>

<template>
  <el-dialog class="!md:w-2/3 !xl:w-1/2 !w-5/6" v-model="visible" destroy-on-close>
    <template v-if="state.tool" #title>
      <el-tag class="mr-2" v-if="state.tool.type.toLowerCase() == 'mcp'" size="small" disable-transitions>MCP</el-tag>
      <span class="text-lg font-semibold">{{ state.tool?.toolset_title }}</span>
    </template>

    <div class="max-h-50vh overflow-auto">
      <div class="whitespace-pre-wrap" v-if="state.tool">
        <div>{{ state.tool.description }}</div>

        <div class="mt-4">
          <div class="mb-2" v-for="(item, name) in formatToolParameters(state.tool)" :key="name">
            <div class="font-semibold">{{ name }}</div>
            <div>{{ item.description }}</div>
          </div>
        </div>
      </div>
      <div v-else>
        <div class="flex items-center">
          <span class="mr-4">{{ $t('label.skill_tool_not_exists') }}</span>
          <el-button size="small" type="danger" @click="onDelete">{{ $t('label.delete') }}</el-button>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button type="primary" @click="visible = false">{{ $t('label.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
