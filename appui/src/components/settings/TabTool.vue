<script setup>
import { callToolQueryByToolset, callToolDeleteByToolset } from '@/call';
import AddToolsetDialog from '@/components/settings/AddToolsetDialog.vue';
import SettingsMenuItem from '@/components/settings/SettingsMenuItem.vue';
import { useToolStore } from '@/stores/tool';
import { formatToolParameters } from '@/utils/tool';
import { ElMessageBox } from 'element-plus';
import { onMounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const toolStore = useToolStore();

const state = reactive({
  showAddToolsetDialog: false,

  currentToolset: null,
  currentTools: [],

  requestingDeleteToolsetIds: [],
});

const fetchTools = async () => {
  state.currentTools = [];

  const toolsetId = state.currentToolset?.id;
  if (toolsetId) {
    state.currentTools = await callToolQueryByToolset(toolsetId);
  }
};

const onToolsetAdded = (toolsets) => {
  onToolsetSelected(toolsets[0]);
};

const onConfirmDeleteToolset = (toolset) => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: toolset.title }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      requestDeleteToolset(toolset);
    })
    .catch(() => {});
};

const onToolsetSelected = (v) => {
  state.currentToolset = v;
};

const requestDeleteToolset = async (toolset) => {
  state.requestingDeleteToolsetIds.push(toolset.id);

  try {
    await callToolDeleteByToolset(toolset.id);

    await toolStore.fetchToolsets();

    if (state.currentToolset?.id === toolset.id) {
      selectFirstToolset();
    }
  } finally {
    state.requestingDeleteToolsetIds.splice(state.requestingDeleteToolsetIds.indexOf(toolset.id), 1);
  }
};

const selectFirstToolset = () => {
  if (toolStore.toolsets.length > 0) {
    onToolsetSelected(toolStore.toolsets[0]);
  } else {
    state.currentTools = [];
  }
};

onMounted(async () => {
  await toolStore.fetchToolsets();

  selectFirstToolset();
});

watch(
  () => state.currentToolset,
  async () => {
    await fetchTools();
  },
);
</script>

<template>
  <div class="flex h-full w-full">
    <div class="border-l-1 flex flex-col gap-2 border-neutral-100 bg-white p-4">
      <el-button class="mb-2 min-w-48" round @click="state.showAddToolsetDialog = true">
        <el-icon><i class="ri-add-line"></i></el-icon>
        {{ $t('label.add') }}
      </el-button>

      <SettingsMenuItem
        v-for="toolset in toolStore.toolsets"
        :key="toolset.id"
        @select="onToolsetSelected(toolset)"
        :active="state.currentToolset?.id === toolset.id"
      >
        <div class="flex w-full items-center">
          <span>{{ toolset.title }}</span>

          <span class="min-w-20 flex-1"></span>

          <el-button @click="onConfirmDeleteToolset(toolset)" link type="danger">
            <template v-if="state.requestingDeleteToolsetIds.includes(toolset.id)">
              <el-icon class="rotating">
                <i class="ri-loader-4-line"></i>
              </el-icon>
            </template>
            <template v-else>
              <el-icon><i class="ri-delete-bin-6-line"></i></el-icon>
            </template>
          </el-button>
        </div>
      </SettingsMenuItem>
    </div>

    <div class="flex flex-1 flex-col">
      <div class="border-l-1 flex flex-1 flex-col gap-2 overflow-auto border-neutral-100">
        <el-collapse accordion>
          <el-collapse-item v-for="tool in state.currentTools" :key="tool.id">
            <template #title>
              <div class="ml-4 flex items-center">
                <el-tag class="mr-2" v-if="tool.type.toLowerCase() == 'ahp'" size="small" disable-transitions>
                  AHP
                </el-tag>
                <el-tag class="mr-2" v-else-if="tool.type.toLowerCase() == 'mcp'" size="small" disable-transitions>
                  MCP
                </el-tag>
                <div class="font-semibold">{{ tool.name }}</div>
              </div>
            </template>

            <div class="whitespace-pre-wrap pl-4">
              <div>{{ tool.description }}</div>

              <div class="mt-4">
                <div class="mb-2" v-for="(item, name) in formatToolParameters(tool)" :key="name">
                  <div class="font-semibold">{{ name }}</div>
                  <div>{{ item.description }}</div>
                </div>
              </div>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
    </div>
  </div>

  <AddToolsetDialog v-model:visible="state.showAddToolsetDialog" @added="onToolsetAdded" />
</template>
