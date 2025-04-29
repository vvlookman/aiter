import api from '@/api';
import { defineStore } from 'pinia';

export const useLlmStore = defineStore('llm', {
  state: () => ({
    items: [],

    defaultChatLlmName: null,
    chatLlmName: null,

    defaultReasoningLlmName: null,
    reasoningLlmName: null,
  }),

  actions: {
    delete(name) {
      const index = this.items.findIndex((item) => item.name === name);
      if (index > -1) {
        this.items.splice(index, 1);
      }

      if (this.defaultChatLlmName === name) {
        this.defaultChatLlmName = null;
      }

      if (this.chatLlmName === name) {
        this.chatLlmName = null;
        localStorage.setItem('aiter-chat-llm', '');
      }

      if (this.defaultReasoningLlmName === name) {
        this.defaultReasoningLlmName = null;
      }

      if (this.reasoningLlmName === name) {
        this.reasoningLlmName = null;
        localStorage.setItem('aiter-reasoning-llm', '');
      }
    },

    async fetch() {
      this.items = await api.post(`/llm/list`);
    },

    async fetchActivedNames() {
      const names = await api.post(`/llm/list-actived-names`);

      if (names.chat) {
        this.defaultChatLlmName = names.chat;
      }

      if (names.reasoning) {
        this.defaultReasoningLlmName = names.reasoning;
      }
    },

    upsert(name, llmData) {
      const index = this.items.findIndex((item) => item.name === name);
      if (index > -1) {
        this.items[index] = llmData;
      } else {
        this.items.unshift(llmData);
      }

      if (this.defaultChatLlmName === name) {
        this.defaultChatLlmName = llmData.name;
      }

      if (this.chatLlmName === name) {
        this.chatLlmName = llmData.name;
        localStorage.setItem('aiter-chat-llm', llmData.name);
      }

      if (this.defaultReasoningLlmName === name) {
        this.defaultReasoningLlmName = llmData.name;
      }

      if (this.reasoningLlmName === name) {
        this.reasoningLlmName = llmData.name;
        localStorage.setItem('aiter-reasoning-llm', llmData.name);
      }
    },
  },

  getters: {
    listByType(state) {
      return (type) => state.items.filter((item) => item.type === type);
    },

    getByName(state) {
      return (name) => state.items.find((item) => item.name === name);
    },
  },
});
