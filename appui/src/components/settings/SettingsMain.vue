<script setup>
import SettingsMenuItem from '@/components/settings/SettingsMenuItem.vue';
import TabAbout from '@/components/settings/TabAbout.vue';
import TabAi from '@/components/settings/TabAi.vue';
import TabGeneral from '@/components/settings/TabGeneral.vue';
import TabLlmChat from '@/components/settings/TabLlmChat.vue';
import TabLlmReasoning from '@/components/settings/TabLlmReasoning.vue';
import TabRemote from '@/components/settings/TabRemote.vue';
import TabTool from '@/components/settings/TabTool.vue';
import { useAppStore } from '@/stores/app';

const visible = defineModel('visible');
const appStore = useAppStore();

const onSelected = (key) => {
  appStore.settingsMenu = key;
};
</script>

<template>
  <div class="flex h-full w-full" v-show="visible">
    <div class="flex flex-col gap-2 p-4">
      <SettingsMenuItem @select="onSelected('ai')" :active="appStore.settingsMenu === 'ai'">
        <el-icon><i class="ri-robot-3-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_ai') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('llm_chat')" :active="appStore.settingsMenu === 'llm_chat'">
        <el-icon><i class="ri-chat-ai-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_llm_chat') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('llm_reasoning')" :active="appStore.settingsMenu === 'llm_reasoning'">
        <el-icon><i class="ri-stairs-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_llm_reasoning') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('tool')" :active="appStore.settingsMenu === 'tool'">
        <el-icon><i class="ri-command-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_tool') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('general')" :active="appStore.settingsMenu === 'general'">
        <el-icon><i class="ri-settings-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_general') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('remote')" :active="appStore.settingsMenu === 'remote'">
        <el-icon><i class="ri-radar-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_remote') }}</span>
      </SettingsMenuItem>

      <SettingsMenuItem @select="onSelected('about')" :active="appStore.settingsMenu === 'about'">
        <el-icon><i class="ri-information-line"></i></el-icon>
        <span class="ml-2 mr-20">{{ $t('label.settings_about') }}</span>
      </SettingsMenuItem>
    </div>

    <div class="flex h-full flex-1 flex-col items-center gap-2 rounded-r-md bg-neutral-200">
      <TabAi v-if="appStore.settingsMenu === 'ai'" />
      <TabLlmChat v-else-if="appStore.settingsMenu === 'llm_chat'" />
      <TabLlmReasoning v-else-if="appStore.settingsMenu === 'llm_reasoning'" />
      <TabTool v-else-if="appStore.settingsMenu === 'tool'" />
      <TabGeneral v-else-if="appStore.settingsMenu === 'general'" />
      <TabRemote v-else-if="appStore.settingsMenu === 'remote'" />
      <TabAbout v-else-if="appStore.settingsMenu === 'about'" />
    </div>
  </div>
</template>
