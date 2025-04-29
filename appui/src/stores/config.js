import { invoke } from '@tauri-apps/api/core';
import { defineStore } from 'pinia';

export const useConfigStore = defineStore('config', {
  state: () => ({
    appDigestBatch: 2,
    appDigestConcurrent: 8,
    appDigestDeep: false,
    appSkipDigest: false,
  }),

  actions: {
    async fetch() {
      const strAppDigestBatch = await invoke('config_get', { key: 'AppDigestBatch' });
      this.appDigestBatch = Math.max(parseInt(strAppDigestBatch) || 2, 1);

      const strAppDigestConcurrent = await invoke('config_get', { key: 'AppDigestConcurrent' });
      this.appDigestConcurrent = Math.max(parseInt(strAppDigestConcurrent) || 8, 1);

      const strAppDigestDeep = await invoke('config_get', { key: 'AppDigestDeep' });
      this.appSkipDigest = strAppDigestDeep?.toLowerCase() == 'true';

      const strAppSkipDigest = await invoke('config_get', { key: 'AppSkipDigest' });
      this.appSkipDigest = strAppSkipDigest?.toLowerCase() == 'true';
    },

    async saveAppDigestBatch() {
      await invoke('config_set', { key: 'AppDigestBatch', value: String(this.appDigestBatch) });
    },

    async saveAppDigestConcurrent() {
      await invoke('config_set', { key: 'AppDigestConcurrent', value: String(this.appDigestConcurrent) });
    },

    async saveAppDigestDeep() {
      await invoke('config_set', { key: 'AppDigestDeep', value: String(this.appDigestConcurrent) });
    },

    async saveAppSkipDigest() {
      await invoke('config_set', { key: 'AppSkipDigest', value: String(this.appSkipDigest) });
    },
  },
});
