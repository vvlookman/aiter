import { defineStore } from 'pinia';

export const useDocStore = defineStore('doc', {
  state: () => ({
    learnQueues: {},
  }),

  actions: {
    addToLearnQueue(ai, hash, file) {
      const queue = (this.learnQueues[ai] = this.learnQueues[ai] ?? []);
      queue.push({ hash, file });
    },

    deleteFromLearnQueue(ai, hash) {
      const queue = (this.learnQueues[ai] = this.learnQueues[ai] ?? []);
      const index = queue.findIndex((item) => item.hash === hash);
      queue.splice(index, 1);
    },

    existsInLearnQueue(ai, hash) {
      const queue = (this.learnQueues[ai] = this.learnQueues[ai] ?? []);
      if (queue.length > 0) {
        return queue.some((item) => item.hash === hash);
      }

      return false;
    },

    shiftFromLearnQueue(ai) {
      const queue = (this.learnQueues[ai] = this.learnQueues[ai] ?? []);
      if (queue.length > 0) {
        queue.shift();
      }
    },
  },

  getters: {
    getLearnQueue(state) {
      return (ai) => state.learnQueues[ai] ?? [];
    },
  },
});
