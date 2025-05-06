import { defineStore } from 'pinia';

export const useChatStore = defineStore('chat', {
  state: () => ({
    notices: {},

    chatRetrace: 6,
    chatTemperature: 0.6,

    uiExpandWidth: false,
  }),

  actions: {
    setNoticed(ai, noticed) {
      this.notices[ai] = noticed;
    },
  },

  getters: {
    isNoticed(state) {
      return (ai) => state.notices[ai];
    },
  },
});
