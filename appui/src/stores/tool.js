import { callToolListToolsets, callToolListByIds } from '@/call';
import { defineStore } from 'pinia';

export const useToolStore = defineStore('tool', {
  state: () => ({
    toolsets: [],
    tools: {},
  }),

  actions: {
    async fetchToolsets() {
      this.toolsets = await callToolListToolsets();
    },

    async fetchToolsByIds(ids) {
      const tools = await callToolListByIds(ids);

      for (const tool of tools) {
        this.tools[tool.id] = tool;
      }
    },
  },

  getters: {
    getById(state) {
      return (id) => state.tools[id];
    },
  },
});
