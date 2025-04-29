import { defineStore } from 'pinia';

export const useAppStore = defineStore('app', {
  state: () => ({
    mainMenu: 'chat',
    settingsMenu: 'ai',
  }),
});
