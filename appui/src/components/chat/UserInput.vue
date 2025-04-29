<script setup>
import { onMounted, reactive, ref, watch } from 'vue';

const input = defineModel('input');
const emit = defineEmits(['send', 'stop']);
defineProps({
  waiting: Boolean,
});

const refInput = ref();
const state = reactive({
  deep: false,
  freely: false,

  shiftKey: false,
  isComposing: false,
});

const onSend = () => {
  emit('send', { input: input.value.trim(), deep: state.deep, strict: !state.freely });
  input.value = '';
};

watch(
  () => input.value,
  () => {
    refInput.value.focus();
  },
);

watch(
  () => state.deep,
  (newVal) => {
    localStorage.setItem('aiter-deep', newVal);
  },
);

watch(
  () => state.freely,
  (newVal) => {
    localStorage.setItem('aiter-freely', newVal);
  },
);

onMounted(() => {
  state.deep = localStorage.getItem('aiter-deep') === 'true';
  state.freely = localStorage.getItem('aiter-freely') === 'true';
});
</script>

<template>
  <div class="border-1 rounded-2xl border-solid border-neutral-200 bg-neutral-100 px-4 pb-2 pt-4">
    <el-input
      v-model="input"
      :autosize="{ minRows: 3, maxRows: 8 }"
      :placeholder="$t('label.chat_user_input_placeholder')"
      :rows="3"
      @keydown.enter.prevent="
        (e) => {
          state.shiftKey = e.shiftKey;
          state.isComposing = e.isComposing || e.keyCode !== 13;
        }
      "
      @keyup.enter.prevent="
        (e) => {
          if (!state.shiftKey && !state.isComposing && !waiting) {
            onSend();
          }
        }
      "
      autofocus
      input-style="background: transparent; border: none; box-shadow: none; padding: 0 4px;"
      maxlength="2000"
      ref="refInput"
      resize="none"
      type="textarea"
    />

    <div class="flex">
      <el-switch
        class="mr-2"
        v-model="state.deep"
        :active-text="$t('label.chat_deep_on')"
        :inactive-text="$t('label.chat_deep_off')"
        inline-prompt
      />

      <el-switch
        class="mr-2"
        v-model="state.freely"
        :active-text="$t('label.chat_freely_on')"
        :inactive-text="$t('label.chat_freely_off')"
        inline-prompt
      />

      <div class="flex-1"></div>

      <template v-if="waiting">
        <el-button @click="emit('stop')" circle type="primary">
          <el-icon><i class="ri-stop-fill"></i></el-icon>
        </el-button>
      </template>
      <template v-else>
        <el-button :disabled="!input.trim()" @click="onSend" circle type="primary">
          <el-icon><i class="ri-arrow-up-line"></i></el-icon>
        </el-button>
      </template>
    </div>
  </div>
</template>
