<script setup>
import { useI18n } from 'vue-i18n';

const { locale } = useI18n();

const onSwitch = (lang) => {
  locale.value = lang;
  localStorage.setItem('aiter-lang', lang);
};
</script>

<template>
  <el-dropdown @command="onSwitch" trigger="click">
    <span class="el-dropdown-link">
      <el-icon class="cursor-pointer"><i class="ri-translate-2"></i> </el-icon>
    </span>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="zh-CN">中文</el-dropdown-item>
        <el-dropdown-item command="en-US">English</el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>
