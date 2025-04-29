<script setup>
import { getCurrentInstance, onErrorCaptured } from 'vue';

import { ElMessage } from 'element-plus';
import { RouterView } from 'vue-router';

const { appContext } = getCurrentInstance();

onErrorCaptured((err) => {
  console.error(err);

  ElMessage(
    {
      type: err.code ? 'error' : 'warning',
      duration: 3000,
      message: err.code ? `${err.code}: ${err.message}` : err.message,
    },
    appContext,
  );

  switch (err.code) {
    case 'UNAUTHORIZED':
      appContext.config.globalProperties.$router.push({ name: 'signin' });
      break;
  }

  return false;
});
</script>

<template>
  <RouterView />
</template>
