<script setup>
import { callAiAdd, callAiDelete, callAiRename } from '@/call';
import SettingsMenuItem from '@/components/settings/SettingsMenuItem.vue';
import { useAiStore } from '@/stores/ai';
import { ElMessage, ElMessageBox } from 'element-plus';
import { reactive } from 'vue';
import { useI18n } from 'vue-i18n';

const aiStore = useAiStore();
const { t } = useI18n();

const state = reactive({
  requestingAdd: false,
  requestingDeleteIds: [],
  requestingRenameIds: [],
});

const requestAdd = async (name) => {
  state.requestingAdd = true;

  try {
    const added = await callAiAdd(name);

    aiStore.upsert(added);
    aiStore.activeId = added.id;

    localStorage.setItem('aiter-ai', added.id);
  } finally {
    state.requestingAdd = false;
  }
};

const requestDelete = async (ai) => {
  state.requestingDeleteIds.push(ai.id);

  try {
    const deleted = await callAiDelete(ai.name);

    aiStore.delete(deleted);
  } finally {
    state.requestingDeleteIds.splice(state.requestingDeleteIds.indexOf(ai.id), 1);
  }
};

const requestRename = async (ai, newName) => {
  state.requestingRenameIds.push(ai.id);

  try {
    const renamed = await callAiRename(ai.name, newName);

    aiStore.upsert(renamed);
  } finally {
    state.requestingRenameIds.splice(state.requestingRenameIds.indexOf(ai.id), 1);
  }
};

const onConfirmDelete = (ai) => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: ai.name }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      requestDelete(ai);
    })
    .catch(() => {});
};

const onPromptAdd = () => {
  ElMessageBox.prompt('', t('label.add'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
  })
    .then(({ value }) => {
      let name = value.trim();

      if (!name.length || name.startsWith('~') || name.startsWith('@')) {
        ElMessage.error(t('message.invalid_name'));
      } else {
        requestAdd(name);
      }
    })
    .catch(() => {});
};

const onPromptRename = (ai) => {
  ElMessageBox.prompt('', t('label.rename'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    inputValue: ai.name,
  })
    .then(({ value }) => {
      let name = value.trim();

      if (!name.length || name.startsWith('~') || name.startsWith('@')) {
        ElMessage.error(t('message.invalid_name'));
      } else {
        requestRename(ai, name);
      }
    })
    .catch(() => {});
};
</script>

<template>
  <div class="flex h-full w-full">
    <div class="border-l-1 flex w-full flex-col gap-2 overflow-auto rounded-r-md border-neutral-100 bg-white p-4">
      <el-button class="mb-2" round :disabled="state.requestingAdd" @click="onPromptAdd">
        <template v-if="state.requestingAdd">
          <el-icon class="rotating">
            <i class="ri-loader-4-line"></i>
          </el-icon>
        </template>
        <template v-else>
          <el-icon><i class="ri-add-line"></i></el-icon>
        </template>
        {{ $t('label.add') }}
      </el-button>

      <SettingsMenuItem v-for="ai in aiStore.items" :key="ai.id">
        <div class="flex w-full items-center">
          <span class="font-semibold">{{ ai.name }}</span>
          <el-icon class="ml-2" v-if="ai.id === aiStore.activeId" type="info"
            ><i class="ri-checkbox-circle-line"></i
          ></el-icon>
          <span class="flex-1"></span>
          <el-button @click="onPromptRename(ai)" link>
            <el-icon class="el-icon--left"><i class="ri-edit-box-line"></i></el-icon>
          </el-button>
          <el-button @click="onConfirmDelete(ai)" link type="danger">
            <template v-if="state.requestingDeleteIds.includes(ai.id)">
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
  </div>
</template>
