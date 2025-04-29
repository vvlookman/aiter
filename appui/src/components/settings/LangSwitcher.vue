<script setup>
import { onMounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { locale } = useI18n();
const state = reactive({
  lang: 'zh-CN',
});

onMounted(() => {
  state.lang = locale.value;
});

watch(
  () => state.lang,
  (lang) => {
    locale.value = lang;
    localStorage.setItem('aiter-lang', lang);
  },
);
</script>

<template>
  <el-select class="!w-12em" v-model="state.lang">
    <el-option key="zh-CN" label="中文" value="zh-CN" />
    <el-option key="en-US" label="English" value="en-US" />
  </el-select>
</template>
