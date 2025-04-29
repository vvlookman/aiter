import api from '@/api';
import { defineStore } from 'pinia';

export const useAiStore = defineStore('ai', {
  state: () => ({
    items: [],

    activeId: null,
  }),

  actions: {
    delete(ai) {
      const index = this.items.findIndex((item) => item.id === ai.id);
      if (index > -1) {
        this.items.splice(index, 1);

        if (this.activeId === ai.id) {
          this.activeId = null;
        }
      }
    },

    async fetch() {
      this.items = await api.post(`/ai/list`);
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
      return () => state.items.find((item) => item.id === this.activeId)?.name;
    },
  },
});
