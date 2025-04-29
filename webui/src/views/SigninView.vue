<script setup>
import { reactive } from 'vue';

import api from '@/api';
import AboutDialog from '@/components/AboutDialog.vue';
import LangSwitcher from '@/components/utils/LangSwitcher.vue';
import { sha256 } from 'js-sha256';
import { useRouter } from 'vue-router';

const router = useRouter();

const state = reactive({
  aboutVisible: false,

  password: '',
  passwordError: false,
  passwordVerifing: false,
});

async function onSignin() {
  localStorage.setItem('aiter-token', sha256(state.password));

  state.passwordError = false;
  state.passwordVerifing = true;

  try {
    await api.get('/version');

    router.push({ name: 'home' });
  } catch (err) {
    state.passwordError = true;
    throw err;
  } finally {
    state.passwordVerifing = false;
  }
}
</script>

<template>
  <main class="flex h-screen flex-col" v-title="$t('title.signin')">
    <el-page-header class="px-4 py-4" @back="state.aboutVisible = true">
      <template #icon>
        <div class="flex items-center">
          <img class="h-1.8em" src="@/assets/logo.png" />
        </div>
      </template>

      <template #title>
        <div class="flex items-center">
          <el-text class="select-none font-bold" size="large" type="primary">Aiter</el-text>
        </div>
      </template>

      <template #content>
        <div class="flex items-center">
          <LangSwitcher />
        </div>
      </template>
    </el-page-header>

    <hr class="m-0" />

    <div class="flex flex-1 flex-col items-center justify-center">
      <div class="w-24em">
        <el-input
          v-model="state.password"
          :disabled="state.passwordVerifing"
          :placeholder="$t('label.signin_password_placeholder')"
          @keyup.enter.prevent="onSignin"
          clearable
          show-password
          size="large"
          type="password"
        >
          <template #prefix>
            <div class="pl-2 pr-4">
              <el-icon><i class="ri-lock-password-line"></i></el-icon>
            </div>
          </template>

          <template #append>
            <template v-if="!state.passwordVerifing">
              <el-button @click="onSignin">
                <el-icon><i class="ri-arrow-right-s-line"></i></el-icon>
              </el-button>
            </template>
            <template v-else>
              <el-icon class="rotating"><i class="ri-loader-4-line"></i></el-icon>
            </template>
          </template>
        </el-input>

        <Transition name="fade">
          <div class="mt-4" v-show="state.passwordError">
            <el-alert :closable="false" :title="$t('message.password_invalid')" show-icon type="error" />
          </div>
        </Transition>
      </div>
    </div>
  </main>

  <AboutDialog v-model:visible="state.aboutVisible" />
</template>
