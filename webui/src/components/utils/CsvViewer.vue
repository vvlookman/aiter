<script setup>
import Papa from 'papaparse';
import { onMounted, reactive, watch } from 'vue';

const props = defineProps({
  data: String,
});

const state = reactive({
  columns: [],
  rows: [],
  parsing: false,
});

const parseData = () => {
  if (props.data) {
    state.parsing = true;

    Papa.parse(props.data, {
      header: true,
      worker: Papa.WORKERS_SUPPORTED,
      complete: function (result) {
        state.columns = result.meta.fields.map((field) => ({ dataKey: field, key: field, title: field, width: 160 }));
        state.rows = result.data;
        state.parsing = false;
      },
    });
  }
};

onMounted(() => {
  parseData();
});

watch(
  () => props.data,
  () => {
    parseData();
  },
);
</script>

<template>
  <div class="h-45vh w-full" v-loading="state.parsing">
    <el-auto-resizer>
      <template #default="{ height, width }">
        <el-table-v2
          v-if="!state.parsing"
          :columns="state.columns"
          :data="state.rows"
          :fixed="state.columns.length > 4"
          :height="height"
          :width="width"
        />
      </template>
    </el-auto-resizer>
  </div>
</template>
