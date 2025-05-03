<script setup>
import SwitchAiDialog from '@/components/ai/SwitchAiDialog.vue';
import ChatMain from '@/components/chat/ChatMain.vue';
import DocMain from '@/components/doc/DocMain.vue';
import SettingsMain from '@/components/settings/SettingsMain.vue';
import SkillMain from '@/components/skill/SkillMain.vue';
import { useAiStore } from '@/stores/ai';
import { useAppStore } from '@/stores/app';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ElMessage } from 'element-plus';
import { getCurrentInstance, onErrorCaptured, onMounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { locale, t } = useI18n();
const aiStore = useAiStore();
const appStore = useAppStore();
const { appContext } = getCurrentInstance();

const state = reactive({
  showSwitchAiDialog: false,
});

const onMenuSelected = (key) => {
  appStore.mainMenu = key;
};

const updateTitle = async () => {
  await getCurrentWindow().setTitle(aiStore.getActiveName() ?? t('label.default_ai'));
};

onErrorCaptured((err) => {
  console.error(err);

  ElMessage(
    {
      type: err.code ? 'error' : 'warning',
      duration: 3000,
      message: err.code ? `${err.code}: ${err.message}` : err.message ? err.message : err,
    },
    appContext,
  );

  return false;
});

onMounted(async () => {
  await aiStore.fetch();

  aiStore.active(localStorage.getItem('aiter-ai'));
  await updateTitle();
});

watch(
  () => aiStore.activeId,
  async (val) => {
    localStorage.setItem('aiter-ai', val);
    await updateTitle();

    const name = aiStore.getActiveName() ?? t('label.default_ai');
    ElMessage(
      {
        type: 'success',
        message: t('message.success_switch_ai', { name }),
      },
      appContext,
    );
  },
);

watch(
  () => locale.value,
  async () => {
    await updateTitle();
  },
);

watch(
  () => state.showSwitchAiDialog,
  async (val) => {
    if (val) {
      await aiStore.fetch();
    }
  },
);
</script>

<template>
  <main class="flex h-screen gap-2 bg-neutral-100 p-2">
    <div class="flex flex-col items-center">
      <el-tooltip :content="$t('label.switch_ai')" placement="right">
        <el-button class="mb-6 mt-4" circle type="primary" @click="state.showSwitchAiDialog = true">
          <i class="ri-robot-3-fill"></i>
        </el-button>
      </el-tooltip>

      <el-menu
        class="!border-0 !bg-transparent"
        :collapse="true"
        :unique-opened="true"
        :default-active="appStore.mainMenu"
        @select="onMenuSelected"
      >
        <el-menu-item class="rounded-md" index="chat">
          <el-icon><i class="ri-chat-smile-ai-line"></i></el-icon>
          <template #title>{{ $t('label.chat') }}</template>
        </el-menu-item>
        <el-menu-item class="rounded-md" index="doc">
          <el-icon><i class="ri-database-2-line"></i></el-icon>
          <template #title>{{ $t('label.knowledge') }}</template>
        </el-menu-item>
        <el-menu-item class="rounded-md" index="skill">
          <el-icon><i class="ri-hammer-line"></i></el-icon>
          <template #title>{{ $t('label.skill') }}</template>
        </el-menu-item>
      </el-menu>

      <span class="flex-1"></span>

      <el-menu
        class="!border-0 !bg-transparent"
        :collapse="true"
        :unique-opened="true"
        :default-active="appStore.mainMenu"
        @select="onMenuSelected"
      >
        <el-menu-item class="rounded-md" index="settings">
          <el-icon><i class="ri-settings-3-line"></i></el-icon>
          <template #title>{{ $t('label.settings') }}</template>
        </el-menu-item>
      </el-menu>
    </div>

    <div class="border-1 flex-1 rounded-md border-gray-100 bg-white">
      <ChatMain :visible="appStore.mainMenu === 'chat'" />
      <DocMain :visible="appStore.mainMenu === 'doc'" />
      <SkillMain :visible="appStore.mainMenu === 'skill'" />
      <SettingsMain :visible="appStore.mainMenu === 'settings'" />
    </div>
  </main>

  <SwitchAiDialog v-model:visible="state.showSwitchAiDialog" />
</template>
