<script setup>
import { callDocList, callDocLearn, callDocDelete, callDocListByIds, callDocListDigestingIds } from '@/call';
import DocCard from '@/components/doc/DocCard.vue';
import LearnQueueDialog from '@/components/doc/LearnQueueDialog.vue';
import ViewDocDialog from '@/components/doc/ViewDocDialog.vue';
import { useAiStore } from '@/stores/ai';
import { useDocStore } from '@/stores/doc';
import { fileMd5 } from '@/utils/fs';
import { ElMessage } from 'element-plus';
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const visible = defineModel('visible');
const aiStore = useAiStore();
const docStore = useDocStore();
const { t } = useI18n();

const refInputFile = ref();
const refScroll = ref();
const state = reactive({
  search: '',
  docs: [],
  digestingIds: [],
  deletingIds: [],

  requestingFetchDocs: false,
  hasMoreDocs: false,

  addingToLearnQueue: false,
  showLearnQueueDialog: false,

  docSelected: null,
  showViewDocDialog: false,
});

const learnQueue = computed(() => docStore.getLearnQueue(aiStore.getActiveName()));

const pageSize = 20;

const fetchDigestingIds = async () => {
  const currentDigestingIds = await callDocListDigestingIds(aiStore.getActiveName(), 100);

  const deltaIds = state.digestingIds.filter((id) => !currentDigestingIds.includes(id));
  if (deltaIds.length > 0) {
    const deltaDocs = await callDocListByIds(aiStore.getActiveName(), deltaIds);

    for (const deltaDoc of deltaDocs) {
      let doc = state.docs.find((d) => d.id === deltaDoc.id);
      if (doc) {
        doc.digest_end = deltaDoc.digest_end;
      }
    }
  }

  state.digestingIds = currentDigestingIds;

  if (state.digestingIds.length > 0) {
    clearTimeout(fetchDigestingIdsTimer);
    fetchDigestingIdsTimer = setTimeout(fetchDigestingIds, 5000);
  }
};

const fetchDocs = async () => {
  state.requestingFetchDocs = true;

  try {
    const aiName = aiStore.getActiveName();
    let docs = await callDocList(aiName, state.search, pageSize + 1, state.docs.length);
    if (aiName !== aiStore.getActiveName()) {
      return;
    }

    if (docs.length > pageSize) {
      state.docs = state.docs.concat(docs.splice(0, pageSize));
      state.hasMoreDocs = true;
    } else {
      state.docs = state.docs.concat(docs);
      state.hasMoreDocs = false;
    }
  } finally {
    state.requestingFetchDocs = false;
  }
};

const refreshDocs = async () => {
  state.docs = [];
  state.hasMoreDocs = false;

  await fetchDocs();
  await fetchDigestingIds();
};

const refreshDocsReset = async () => {
  state.search = '';
  await refreshDocs();
};

const onDeleteDoc = async (doc) => {
  state.deletingIds.push(doc.id);

  try {
    await callDocDelete(aiStore.getActiveName(), doc.id);
    state.docs = state.docs.filter((d) => d.id !== doc.id);
  } finally {
    state.deletingIds = state.deletingIds.filter((id) => id !== doc.id);
  }
};

const onScroll = async () => {
  const wrap = refScroll.value;
  if (wrap) {
    if (
      wrap.scrollTop + wrap.clientHeight >= wrap.scrollHeight - 2 &&
      !state.requestingFetchDocs &&
      state.hasMoreDocs
    ) {
      await fetchDocs();
    }
  }
};

const onSelectFile = async (e) => {
  state.addingToLearnQueue = true;

  for (const file of e.target.files) {
    const hash = await fileMd5(file);
    if (docStore.existsInLearnQueue(aiStore.getActiveName(), hash)) {
      ElMessage({
        type: 'info',
        message: t('message.doc_in_queue', { name: file.name }),
      });
    } else {
      docStore.addToLearnQueue(aiStore.getActiveName(), hash, file);
    }
  }

  refInputFile.value.value = '';
  state.addingToLearnQueue = false;

  await processLearnQueue();
};

const onViewDoc = (doc) => {
  state.docSelected = doc;
  state.showViewDocDialog = true;
};

const processLearnQueue = async () => {
  const aiName = aiStore.getActiveName();

  if (docStore.isProcessingLearnQueue(aiName)) {
    return;
  }

  docStore.setProcessingLearnQueue(aiName, true);

  const queue = docStore.getLearnQueue(aiName);
  while (true) {
    const task = queue.find((task) => !task.processing);
    if (!task) {
      break;
    }

    task.processing = true;

    const { hash, file } = task;

    try {
      const result = await callDocLearn(aiName, file);
      if (result.doc_exists) {
        ElMessage({
          type: 'info',
          message: t('message.doc_exists', { name: file.name }),
        });
      } else {
        state.docs.unshift(result.doc);

        clearTimeout(fetchDigestingIdsTimer);
        fetchDigestingIdsTimer = setTimeout(fetchDigestingIds, 1000);
      }
    } finally {
      docStore.deleteFromLearnQueue(aiName, hash);
      if (!queue.length) {
        state.showLearnQueueDialog = false;
      }
    }
  }

  docStore.setProcessingLearnQueue(aiName, false);
};

let fetchDigestingIdsTimer;

watch(
  () => aiStore.getActiveName(),
  async () => {
    state.search = '';

    await refreshDocs();
    await processLearnQueue();
  },
);

onMounted(async () => {
  await refreshDocs();
  await processLearnQueue();
});

onUnmounted(() => {
  clearTimeout(fetchDigestingIdsTimer);
});
</script>

<template>
  <div class="flex h-full w-full" v-show="visible">
    <div class="flex h-full flex-1 flex-col items-center gap-2 px-2">
      <div class="my-4 flex w-full gap-2 px-2">
        <div class="flex gap-2">
          <el-input class="lg:w-100 w-40 sm:w-60 md:w-80" v-model="state.search" @keyup.enter.prevent="refreshDocs">
            <template #prefix>
              <template v-if="state.requestingFetchDocs">
                <el-icon class="rotating"><i class="ri-loader-2-line"></i></el-icon>
              </template>
              <template v-else>
                <el-icon><i class="ri-search-line"></i></el-icon>
              </template>
            </template>

            <template v-if="!!state.search.trim()" #append>
              <el-button @click="refreshDocsReset">
                <el-icon><i class="ri-close-line"></i></el-icon>
              </el-button>
            </template>
          </el-input>

          <el-button :disabled="state.requestingFetchDocs" @click="refreshDocs">
            <template v-if="state.requestingFetchDocs">
              <el-icon class="rotating"><i class="ri-loader-2-line"></i></el-icon>
            </template>
            <template v-else>
              <el-icon><i class="ri-refresh-line"></i></el-icon>
            </template>
          </el-button>
        </div>

        <span class="flex-1"></span>

        <el-button v-if="learnQueue.length > 0" @click="state.showLearnQueueDialog = true">
          <el-icon class="rotating el-icon--left"><i class="ri-loader-4-line"></i></el-icon>
          <span class="max-w-12em md:max-w-16em xl:max-w-20em overflow-hidden text-ellipsis text-nowrap">
            {{ learnQueue[0].file.name }}
          </span>
          <span class="ml-2">({{ learnQueue.length }})</span>
        </el-button>

        <input
          class="hidden"
          @change="onSelectFile"
          accept=".csv,.docx,.epub,.md,.pdf,.txt,.xlsx,.xls,.xlsm,.xlsb,.xla,.xlam,.ods"
          ref="refInputFile"
          type="file"
          multiple
        />
        <el-button :disabled="state.addingToLearnQueue" @click="refInputFile.click()" type="primary">
          <template v-if="state.addingToLearnQueue">
            <el-icon class="rotating"><i class="ri-loader-4-line"></i></el-icon>
          </template>
          <template v-else>
            <el-icon><i class="ri-upload-2-line"></i></el-icon>
          </template>
        </el-button>
      </div>

      <div class="flex w-full flex-1 flex-col items-center overflow-auto px-2" ref="refScroll" @scroll="onScroll">
        <template v-if="state.docs.length > 0">
          <div
            class="grid w-full grid-flow-row grid-cols-1 gap-4 pb-8 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
          >
            <DocCard
              v-for="doc in state.docs"
              :deleting="state.deletingIds.includes(doc.id)"
              :digesting="state.digestingIds.includes(doc.id)"
              :doc="doc"
              :key="doc.id"
              @delete="onDeleteDoc(doc)"
              @view="onViewDoc(doc)"
            />
          </div>
        </template>
        <template v-else>
          <div v-if="!state.requestingFetchDocs">
            <el-empty :description="$t('label.empty')" />
          </div>
        </template>
      </div>
    </div>
  </div>

  <LearnQueueDialog v-model:visible="state.showLearnQueueDialog" />
  <ViewDocDialog v-model:visible="state.showViewDocDialog" :doc="state.docSelected" />
</template>
