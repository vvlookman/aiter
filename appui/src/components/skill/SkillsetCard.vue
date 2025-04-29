<script setup>
import { ElMessageBox } from 'element-plus';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const emit = defineEmits(['delete', 'ungroup']);
const props = defineProps({
  skillset: Object,

  deleting: Boolean,
});

const onDelete = () => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: props.skillset.toolset_title }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      emit('delete', props.skillset);
    })
    .catch(() => {});
};
</script>

<template>
  <div>
    <div class="bg-primary flex cursor-pointer items-center rounded-t-md py-2 pl-4 pr-3 text-xs text-white">
      <el-tooltip :content="skillset.toolset_title" effect="light" placement="top" trigger="click">
        <span class="flex-1 overflow-hidden text-ellipsis text-nowrap">
          {{ skillset.toolset_title }}
        </span>
      </el-tooltip>

      <el-button @click="emit('ungroup')" link size="small" color="#079d55">
        <el-icon><i class="ri-menu-unfold-3-line"></i></el-icon>
      </el-button>
    </div>

    <div class="text-0.65rem bg-green-50 px-4 py-2 text-gray-400">
      <div class="break-all" v-for="skill in skillset.skills" :key="skill.id">{{ skill.trigger }}</div>
    </div>

    <div class="text-0.65rem flex select-none items-center rounded-b-md bg-gray-50 py-2 pl-4 pr-2 text-gray-400">
      <span class="flex-1">
        <el-icon class="mr-1"><i class="ri-list-check"></i></el-icon>
        {{ skillset.skills.length }}
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
