<script setup>
import { reactive } from 'vue';

import AboutDialog from '@/components/AboutDialog.vue';
import ChatMain from '@/components/chat/ChatMain.vue';
import AiSwitcher from '@/components/utils/AiSwitcher.vue';
import LangSwitcher from '@/components/utils/LangSwitcher.vue';

const state = reactive({
  aboutVisible: false,
});
</script>

<template>
  <main class="flex h-screen flex-col">
    <el-page-header class="px-4 py-4" @back="state.aboutVisible = true">
      <template #icon>
        <div class="flex items-center">
          <img class="h-1.8em" src="@/assets/logo.png" />
        </div>
      </template>

      <template #title>
        <div class="flex items-center">
          <el-text class="select-none font-bold" size="large" type="primary">Aiter</el-text>
        </div>
      </template>

      <template #content>
        <div class="flex items-center">
          <LangSwitcher />
        </div>
      </template>

      <template #extra>
        <div class="flex items-center">
          <AiSwitcher />
        </div>
      </template>
    </el-page-header>

    <hr class="m-0" />

    <div style="height: calc(100vh - 2rem - 32px - 4px)">
      <ChatMain />
    </div>
  </main>

  <AboutDialog v-model:visible="state.aboutVisible" />
</template>
