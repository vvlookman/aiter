<script setup>
import { callDocCountPart, callDocGetPart } from '@/call';
import CsvViewer from '@/components/utils/CsvViewer.vue';
import { useAiStore } from '@/stores/ai';
import { renderMarkdown } from '@/utils/markdown';
import { computed, reactive, ref, watch } from 'vue';

const visible = defineModel('visible');
const props = defineProps({
  doc: Object,
});
const aiStore = useAiStore();

const refScroll = ref();
const state = reactive({
  pages: 0,

  currentPage: 1,
  currentPageData: '',

  fetching: false,
});

const markdownPage = computed(() => renderMarkdown(state.currentPageData));

const countPart = async () => {
  state.fetching = true;

  try {
    state.pages = await callDocCountPart(aiStore.getActiveName(), props.doc.id);
  } finally {
    state.fetching = false;
  }
};

const getPart = async () => {
  state.fetching = true;

  try {
    state.currentPageData = await callDocGetPart(aiStore.getActiveName(), props.doc.id, state.currentPage - 1);
  } finally {
    state.fetching = false;
  }
};

const onCurrentChange = async (page) => {
  state.currentPage = page;
  await getPart();

  const wrap = refScroll.value;
  if (wrap) {
    wrap.scrollTop = 0;
  }
};

watch(
  () => props.doc,
  async () => {
    state.currentPage = 1;
    state.currentPageData = '';

    await countPart();
    await getPart();
  },
);
</script>

<template>
  <el-dialog class="!md:w-2/3 !xl:w-1/2 !w-5/6" v-model="visible" destroy-on-close :title="doc?.title">
    <div class="max-h-50vh overflow-auto" ref="refScroll">
      <template v-if="doc.content_type == 'Geo'">
        <CsvViewer :data="state.currentPageData" />
      </template>
      <template v-else-if="doc.content_type == 'Markdown'">
        <div v-html="markdownPage"></div>
      </template>
      <template v-else-if="doc.content_type == 'Sheet'">
        <CsvViewer :data="state.currentPageData" />
      </template>
      <template v-else>
        <div class="whitespace-pre-wrap">{{ state.currentPageData }}</div>
      </template>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-pagination
          class="justify-end"
          :page-size="1"
          :total="state.pages"
          @current-change="onCurrentChange"
          background
          layout="prev, pager, next"
        />
      </div>
    </template>
  </el-dialog>
</template>
