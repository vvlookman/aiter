<script setup>
import { useToolStore } from '@/stores/tool';
import { ElMessageBox } from 'element-plus';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const toolStore = useToolStore();

const emit = defineEmits(['delete', 'group', 'view']);
const props = defineProps({
  skill: Object,

  deleting: Boolean,
});

const title = computed(() => {
  const tool = toolStore.getById(props.skill.tool_id);
  return tool ? tool.toolset_title : '';
});

const onDelete = () => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: props.skill.trigger }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      emit('delete', props.skill);
    })
    .catch(() => {});
};
</script>

<template>
  <div>
    <div class="bg-primary flex cursor-pointer items-center rounded-t-md py-2 pl-4 pr-3 text-xs text-white">
      <el-tooltip :content="title" effect="light" placement="top" trigger="click">
        <span class="flex-1 overflow-hidden text-ellipsis text-nowrap">
          {{ title }}
        </span>
      </el-tooltip>

      <el-button @click="emit('group')" link size="small" color="#079d55">
        <el-icon><i class="ri-menu-fold-3-line"></i></el-icon>
      </el-button>
    </div>

    <div class="text-0.65rem cursor-pointer bg-green-50 px-4 py-2 text-gray-400" @click="emit('view', skill)">
      <div class="cursor-pointer break-all">{{ skill.trigger }}</div>
    </div>

    <div class="text-0.65rem flex select-none items-center rounded-b-md bg-gray-50 py-2 pl-4 pr-2 text-gray-400">
      <span class="flex-1">
        <el-icon class="mr-1"><i class="ri-time-line"></i></el-icon>
        {{ new Date(skill.updated_at).toLocaleString() }}
      </span>

      <template v-if="deleting">
        <el-icon class="rotating ml-2"><i class="ri-loader-4-line"></i></el-icon>
      </template>
      <template v-else>
        <el-button @click="onDelete" link size="small" type="info">
          <el-icon><i class="ri-delete-bin-6-line"></i></el-icon>
        </el-button>
      </template>
    </div>
  </div>
</template>
