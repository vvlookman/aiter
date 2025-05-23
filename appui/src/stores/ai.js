import { callAiList, callMemStats } from '@/call';
import { defineStore } from 'pinia';

export const useAiStore = defineStore('ai', {
  state: () => ({
    items: [],
    memStats: {},

    activeId: undefined,
  }),

  actions: {
    active(id) {
      if (!id) {
        this.activeId = undefined;
      } else {
        const index = this.items.findIndex((item) => item.id === id);
        if (index > -1) {
          this.activeId = id;
        }
      }
    },

    delete(ai) {
      const index = this.items.findIndex((item) => item.id === ai.id);
      if (index > -1) {
        this.items.splice(index, 1);

        if (this.activeId === ai.id) {
          this.activeId = undefined;
        }
      }
    },

    async fetch() {
      this.items = await callAiList();
    },

    async fetchMemStats() {
      const ai = this.items.find((item) => item.id === this.activeId);
      this.memStats[ai?.id] = await callMemStats(ai?.name);
    },

    upsert(ai) {
      const index = this.items.findIndex((item) => item.id === ai.id);
      if (index > -1) {
        this.items[index] = ai;
      } else {
        this.items.unshift(ai);
      }
    },
  },

  getters: {
    getActiveName(state) {
      return () => state.items.find((item) => item.id === state.activeId)?.name;
    },

    getActiveMemStats(state) {
      return () => state.memStats[state.activeId] ?? {};
    },
  },
});
