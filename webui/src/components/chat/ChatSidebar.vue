<script setup>
import { useChatStore } from '@/stores/chat';
import { useLlmStore } from '@/stores/llm';
import { computed, onMounted, reactive, watch } from 'vue';

const chatStore = useChatStore();
const llmStore = useLlmStore();

const state = reactive({
  showConfig: false,
});

const chatLlmSelected = computed(() => {
  return llmStore.chatLlmName || '~';
});

const onChatLlmChanged = (val) => {
  if (val === '~') {
    llmStore.chatLlmName = null;
  } else {
    llmStore.chatLlmName = val;
  }
};

const onReasoningLlmChanged = (val) => {
  if (val === '~') {
    llmStore.reasoningLlmName = null;
  } else {
    llmStore.reasoningLlmName = val;
  }
};

const reasoningLlmSelected = computed(() => {
  return llmStore.reasoningLlmName || '~';
});

onMounted(async () => {
  await llmStore.fetch();
  await llmStore.fetchActivedNames();

  const chatLlmName = localStorage.getItem('aiter-chatLlmName');
  if (chatLlmName) {
    llmStore.chatLlmName = chatLlmName;
  }

  const chatRetrace = parseInt(localStorage.getItem('aiter-chatRetrace'));
  if (chatRetrace >= 0) {
    chatStore.chatRetrace = chatRetrace;
  }

  const chatTemperature = parseFloat(localStorage.getItem('aiter-chatTemperature'));
  if (chatTemperature >= 0) {
    chatStore.chatTemperature = chatTemperature;
  }

  const uiExpandWidth = localStorage.getItem('aiter-uiExpandWidth');
  chatStore.uiExpandWidth = uiExpandWidth === 'true';
});

watch(
  () => llmStore.chatLlmName,
  (val) => {
    if (val) {
      localStorage.setItem('aiter-chatLlmName', val);
    } else {
      localStorage.removeItem('aiter-chatLlmName');
    }
  },
);

watch(
  () => chatStore.chatRetrace,
  (val) => {
    localStorage.setItem('aiter-chatRetrace', val);
  },
);

watch(
  () => chatStore.chatTemperature,
  (val) => {
    localStorage.setItem('aiter-chatTemperature', val);
  },
);

watch(
  () => chatStore.uiExpandWidth,
  (val) => {
    localStorage.setItem('aiter-uiExpandWidth', val);
  },
);
</script>

<template>
  <div class="border-r-1 flex h-full w-64 flex-col gap-2 border-gray-100 p-4" v-if="state.showConfig">
    <div class="flex-1 overflow-auto p-2">
      <el-form label-position="top">
        <el-form-item>
          <template #label>
            {{ $t('label.chat_llm') }}
          </template>
          <el-select v-model="chatLlmSelected" @change="onChatLlmChanged">
            <el-option :label="$t('label.default_llm')" value="~" />
            <el-option v-for="llm in llmStore.listByType('chat')" :key="llm.name" :value="llm.name" />
          </el-select>
        </el-form-item>

        <el-form-item>
          <template #label>
            {{ $t('label.reasoning_llm') }}
          </template>
          <el-select v-model="reasoningLlmSelected" @change="onReasoningLlmChanged">
            <el-option :label="$t('label.default_llm')" value="~" />
            <el-option v-for="llm in llmStore.listByType('reasoning')" :key="llm.name" :value="llm.name" />
          </el-select>
        </el-form-item>

        <el-form-item>
          <template #label>
            {{ $t('label.chat_temperature') }}
            <el-tooltip :content="$t('tip.chat_temperature')" placement="top" trigger="click" popper-class="max-w-20em">
              <el-button link>
                <el-icon><i class="ri-question-line"></i></el-icon>
              </el-button>
            </el-tooltip>
          </template>
          <el-slider v-model="chatStore.chatTemperature" :step="0.1" :min="0" :max="2" />
        </el-form-item>

        <el-form-item>
          <template #label>
            {{ $t('label.chat_retrace') }}
            <el-tooltip :content="$t('tip.chat_retrace')" placement="top" trigger="click" popper-class="max-w-20em">
              <el-button link>
                <el-icon><i class="ri-question-line"></i></el-icon>
              </el-button>
            </el-tooltip>
          </template>
          <el-slider v-model="chatStore.chatRetrace" :step="1" :min="0" :max="16" />
        </el-form-item>
      </el-form>
    </div>

    <el-button class="w-full" size="small" type="primary" @click="state.showConfig = false">
      {{ $t('label.close') }}
    </el-button>
  </div>

  <div class="flex h-full flex-col gap-2 py-4 pl-4 pr-0" v-else>
    <div>
      <el-button size="small" @click="state.showConfig = true">
        <el-icon><i class="ri-tools-line"></i></el-icon>
      </el-button>
    </div>

    <div>
      <el-button size="small" @click="chatStore.uiExpandWidth = !chatStore.uiExpandWidth">
        <el-icon v-if="chatStore.uiExpandWidth"><i class="ri-collapse-horizontal-fill"></i></el-icon>
        <el-icon v-else><i class="ri-expand-horizontal-line"></i></el-icon>
      </el-button>
    </div>
  </div>
</template>
