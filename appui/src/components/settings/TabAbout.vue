<script setup>
import { callCoreVersion } from '@/call';
import SettingsGroup from '@/components/settings/SettingsGroup.vue';
import { getVersion } from '@tauri-apps/api/app';
import { onMounted, reactive } from 'vue';

const state = reactive({
  version: '',
  coreVersion: '',
});

onMounted(async () => {
  state.version = await getVersion();
  state.coreVersion = await callCoreVersion();
});
</script>

<template>
  <div class="flex h-full w-full px-2 py-4">
    <div class="flex w-full flex-col gap-2 overflow-auto px-2">
      <SettingsGroup :title="$t('label.settings_titles.version')">
        <hr class="my-4" />

        <div class="flex items-center">
          <el-image class="h-18 w-18" src="/icon.png" fit="cover" />

          <div class="ml-4 flex flex-col gap-2">
            <div class="flex items-center">
              <span class="text-xl font-bold">Aiter</span>

              <el-tag class="ml-4" v-if="state.version" size="small" disable-transitions>v{{ state.version }}</el-tag>

              <el-tag class="ml-4" v-if="state.coreVersion" size="small" disable-transitions type="info">
                {{ $t('label.core_version') }} v{{ state.coreVersion }}
              </el-tag>
            </div>
            <div class="text-neutral-700">{{ $t('label.slogan') }}</div>
          </div>
        </div>
      </SettingsGroup>
    </div>
  </div>
</template>
