<script setup>
import LangSwitcher from '@/components/settings/LangSwitcher.vue';
import SettingsGroup from '@/components/settings/SettingsGroup.vue';
import { useConfigStore } from '@/stores/config';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { computed, onMounted, reactive, watch } from 'vue';

const configStore = useConfigStore();

const state = reactive({
  appConfig: null,
});

const requireRestart = computed(() => {
  if (state.appConfig) {
    return (
      state.appConfig.digest_batch !== configStore.appDigestBatch ||
      state.appConfig.digest_concurrent !== configStore.appDigestConcurrent ||
      state.appConfig.digest_deep !== configStore.appDigestDeep ||
      state.appConfig.skip_digest !== configStore.appSkipDigest
    );
  }

  return false;
});

onMounted(async () => {
  state.appConfig = await invoke('app_config');
  await configStore.fetch();
});

watch(
  () => configStore.appDigestBatch,
  () => {
    configStore.saveAppDigestBatch();
  },
);

watch(
  () => configStore.appDigestConcurrent,
  () => {
    configStore.saveAppDigestConcurrent();
  },
);

watch(
  () => configStore.appDigestDeep,
  () => {
    configStore.saveAppDigestDeep();
  },
);

watch(
  () => configStore.appSkipDigest,
  () => {
    configStore.saveAppSkipDigest();
  },
);
</script>

<template>
  <div class="flex h-full w-full px-2 py-4">
    <div class="flex w-full flex-col gap-2 overflow-auto px-2">
      <SettingsGroup :title="$t('label.settings_titles.ui')">
        <hr class="my-4" />

        <div class="flex items-center">
          <span>{{ $t('label.settings_titles.language') }}</span>
          <span class="flex-1"></span>
          <LangSwitcher />
        </div>
      </SettingsGroup>

      <SettingsGroup :title="$t('label.settings_titles.digest')">
        <hr class="my-4" />

        <div class="flex items-center">
          <span>{{ $t('label.settings_titles.digest_batch') }}</span>
          <span class="flex-1"></span>
          <span class="w-48">
            <el-slider v-model="configStore.appDigestBatch" :step="1" :min="1" :max="10" />
          </span>
        </div>

        <hr class="my-4" />

        <div class="flex items-center">
          <span>{{ $t('label.settings_titles.digest_concurrent') }}</span>
          <span class="flex-1"></span>
          <span class="w-48">
            <el-slider v-model="configStore.appDigestConcurrent" :step="1" :min="1" :max="20" />
          </span>
        </div>

        <hr class="my-4" />

        <div class="flex items-center">
          <span>{{ $t('label.settings_titles.digest_deep') }}</span>
          <el-tooltip :content="$t('tip.digest_deep')" placement="top" trigger="click" popper-class="max-w-20em">
            <el-button link>
              <el-icon><i class="ri-question-line"></i></el-icon>
            </el-button>
          </el-tooltip>
          <span class="flex-1"></span>
          <el-switch v-model="configStore.appDigestDeep" />
        </div>

        <hr class="my-4" />

        <div class="flex items-center">
          <span>{{ $t('label.settings_titles.skip_digest') }}</span>
          <el-tooltip :content="$t('tip.skip_digest')" placement="top" trigger="click" popper-class="max-w-20em">
            <el-button link>
              <el-icon><i class="ri-question-line"></i></el-icon>
            </el-button>
          </el-tooltip>
          <span class="flex-1"></span>
          <el-switch v-model="configStore.appSkipDigest" />
        </div>
      </SettingsGroup>

      <SettingsGroup v-if="requireRestart">
        <el-button @click="relaunch()" type="danger">
          <el-icon class="el-icon--left"><i class="ri-restart-line"></i></el-icon>
          {{ $t('label.require_restart') }}
        </el-button>
      </SettingsGroup>
    </div>
  </div>
</template>
