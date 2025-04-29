import { createApp } from 'vue';

import messages from '@intlify/unplugin-vue-i18n/messages';
import { createPinia } from 'pinia';
import { createI18n } from 'vue-i18n';

import App from './App.vue';
import api from './api';
import router from './router';
import './styles/main.scss';

import 'element-plus/theme-chalk/el-message-box.css';
import 'element-plus/theme-chalk/el-message.css';
import 'katex/dist/katex.min.css';
import 'remixicon/fonts/remixicon.css';
import 'virtual:uno.css';

api.init();

const app = createApp(App);

const browserLang = navigator.language ?? navigator.userLanguage;
const langCode = browserLang.split('-')[0];
const defaultLocale = langCode == 'zh' ? 'zh-CN' : 'en-US';

app.use(
  createI18n({
    locale: localStorage.getItem('aiter-lang') ?? defaultLocale,
    fallbackLocale: 'en-US',
    messages,
  }),
);
app.use(createPinia());
app.use(router);

app.directive('title', (el, binding) => {
  document.title = binding.value ? `${binding.value} - Aiter` : 'Aiter';
});

app.mount('#app');
