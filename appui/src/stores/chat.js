import { defineStore } from 'pinia';

export const useChatStore = defineStore('chat', {
  state: () => ({
    chatRetrace: 6,
    chatTemperature: 0.6,

    uiExpandWidth: false,
  }),
});
