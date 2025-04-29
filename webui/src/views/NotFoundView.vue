<script setup>
import { useRouter } from 'vue-router';

const router = useRouter();

function backToHome() {
  router.replace({ name: 'home' });
}
</script>

<template>
  <main class="flex h-screen flex-col" v-title="404">
    <div class="flex flex-1 flex-col items-center justify-center">
      <div>
        <div class="text-6xl font-bold text-gray-500">404</div>

        <div class="my-2 text-gray-400">{{ $t('label.404') }}</div>

        <el-button @click="backToHome" type="primary">
          <el-icon class="el-icon--left"><i class="ri-arrow-left-s-line"></i></el-icon>
          {{ $t('label.back_to_home') }}
        </el-button>
      </div>
    </div>
  </main>
</template>
