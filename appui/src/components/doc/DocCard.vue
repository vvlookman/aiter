<script setup>
import { renderMarkdown } from '@/utils/markdown';
import { ElMessageBox } from 'element-plus';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['delete', 'view']);
const props = defineProps({
  doc: Object,

  deleting: Boolean,
  digesting: Boolean,
});
const { t } = useI18n();

const markdownPreview = computed(() => renderMarkdown(props.doc.preview));

const onDelete = () => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: props.doc.source }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      emit('delete', props.doc);
    })
    .catch(() => {});
};
</script>

<template>
  <div>
    <div class="bg-primary flex cursor-pointer items-center rounded-t-md py-2 pl-4 pr-3 text-xs text-white">
      <el-tooltip :content="doc.source" effect="light" placement="top" trigger="click">
        <span class="flex-1 overflow-hidden text-ellipsis text-nowrap">
          <el-icon>
            <template v-if="doc.content_type == 'Geo'">
              <i class="ri-map-pin-5-line"></i>
            </template>
            <template v-else-if="doc.content_type == 'Markdown'">
              <i class="ri-markdown-line"></i>
            </template>
            <template v-else-if="doc.content_type == 'Sheet'">
              <i class="ri-table-view"></i>
            </template>
            <template v-else>
              <i class="ri-file-text-line"></i>
            </template>
          </el-icon>
          {{ doc.source }}
        </span>
      </el-tooltip>

      <template v-if="doc.digest_end">
        <el-tooltip
          :content="$t('label.digested', { time: new Date(doc.digest_end).toLocaleString() })"
          effect="light"
          placement="top"
          trigger="click"
        >
          <el-icon class="ml-2"><i class="ri-honour-line"></i></el-icon>
        </el-tooltip>
      </template>
      <template v-else>
        <el-icon class="rotating ml-2" v-if="digesting"><i class="ri-loader-4-line"></i></el-icon>
      </template>
    </div>

    <div class="text-0.65rem cursor-pointer bg-green-50 px-4 py-2 text-gray-400" @click="emit('view', doc)">
      <div class="font-bold" v-if="doc.title">{{ doc.title }}</div>
      <div class="cursor-pointer break-all" v-html="markdownPreview"></div>
    </div>

    <div class="text-0.65rem flex select-none items-center rounded-b-md bg-gray-50 py-2 pl-4 pr-2 text-gray-400">
      <span class="flex-1">
        <el-icon class="mr-1"><i class="ri-time-line"></i></el-icon>
        {{ new Date(doc.updated_at).toLocaleString() }}
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
