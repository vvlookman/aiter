<script setup>
import api from '@/api';
import { callAppGetRemoteUrl, callAppSetRemote } from '@/call';
import SettingsGroup from '@/components/settings/SettingsGroup.vue';
import { relaunch } from '@tauri-apps/plugin-process';
import { ElMessageBox } from 'element-plus';
import { sha256 } from 'js-sha256';
import { onMounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const state = reactive({
  remoteOn: false,
  remoteUrl: '',
  remotePassword: '',

  requestingTestRemoteAndSave: false,
});

const onTestRemoteAndSave = async () => {
  state.requestingTestRemoteAndSave = true;

  try {
    await callAppSetRemote(state.remoteUrl, sha256(state.remotePassword));

    const version = await api.get('/version');
    if (!version) {
      await callAppSetRemote();
    } else {
      ElMessageBox.confirm(t('message.success_access_to_remote', { version }), t('label.require_restart'), {
        confirmButtonText: t('label.confirm'),
        cancelButtonText: t('label.cancel'),
        confirmButtonClass: 'el-button--danger',
        type: 'info',
      })
        .then(relaunch)
        .catch(() => {
          callAppSetRemote();
        });
    }
  } finally {
    state.requestingTestRemoteAndSave = false;
  }
};

onMounted(async () => {
  const remoteUrl = await callAppGetRemoteUrl();

  state.remoteOn = !!remoteUrl;
  state.remoteUrl = remoteUrl || 'http://localhost:6868';
});

watch(
  () => state.remoteOn,
  async (val) => {
    const remoteUrl = await callAppGetRemoteUrl();
    if (!val && remoteUrl) {
      await callAppSetRemote();

      ElMessageBox.confirm(t('message.turn_off_remote'), t('label.require_restart'), {
        confirmButtonText: t('label.confirm'),
        cancelButtonText: t('label.cancel'),
        confirmButtonClass: 'el-button--danger',
        type: 'info',
      })
        .then(relaunch)
        .catch(() => {});
    }
  },
);
</script>

<template>
  <div class="flex h-full w-full px-2 py-4">
    <div class="flex w-full flex-col gap-2 overflow-auto px-2">
      <SettingsGroup>
        <div class="flex items-center">
          <span :class="{ 'text-gray-400': !state.remoteOn }">{{ $t('label.settings_titles.access_to_remote') }}</span>
          <span class="flex-1"></span>
          <el-switch v-model="state.remoteOn" />
        </div>

        <div class="mt-8 flex flex-col gap-4" v-if="state.remoteOn">
          <div class="flex items-center">
            <el-form class="w-full" label-position="top">
              <el-form-item :label="$t('label.settings_titles.remote_base_url')">
                <el-input v-model="state.remoteUrl" placeholder="e.g. http://localhost:6868" />
              </el-form-item>

              <el-form-item :label="$t('label.settings_titles.remote_password')">
                <el-input v-model="state.remotePassword" type="password" show-password />
              </el-form-item>
            </el-form>
          </div>

          <div class="flex items-center">
            <el-button
              type="primary"
              :disabled="!state.remoteUrl || state.requestingTestRemoteAndSave"
              @click="onTestRemoteAndSave"
            >
              <template v-if="state.requestingTestRemoteAndSave">
                <el-icon class="rotating el-icon--left"><i class="ri-loader-4-line"></i></el-icon>
              </template>
              <template v-else>
                <el-icon class="el-icon--left"><i class="ri-links-line"></i></el-icon>
              </template>
              {{ $t('label.test_access_and_save') }}
            </el-button>
          </div>
        </div>
      </SettingsGroup>
    </div>
  </div>
</template>
