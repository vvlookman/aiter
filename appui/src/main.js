import messages from '@intlify/unplugin-vue-i18n/messages';
import { createPinia } from 'pinia';
import { createApp } from 'vue';
import { createI18n } from 'vue-i18n';

import 'element-plus/theme-chalk/el-message-box.css';
import 'element-plus/theme-chalk/el-message.css';
import 'katex/dist/katex.min.css';
import 'remixicon/fonts/remixicon.css';
import 'virtual:uno.css';

import App from '@/App.vue';
import '@/styles/main.scss';

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

app.mount('#app');
