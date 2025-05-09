<script setup>
import { callAppGetRemoteUrl, callChat, callChatAbort, callChatClear, callChatDelete, callChatHistory } from '@/call';
import BotMessage from '@/components/chat/BotMessage.vue';
import ChatSidebar from '@/components/chat/ChatSidebar.vue';
import UserInput from '@/components/chat/UserInput.vue';
import UserMessage from '@/components/chat/UserMessage.vue';
import { useAiStore } from '@/stores/ai';
import { useAppStore } from '@/stores/app';
import { useChatStore } from '@/stores/chat';
import { useLlmStore } from '@/stores/llm';
import { Channel } from '@tauri-apps/api/core';
import { ulid } from 'ulid';
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const visible = defineModel('visible');
const aiStore = useAiStore();
const appStore = useAppStore();
const chatStore = useChatStore();
const llmStore = useLlmStore();
const { t } = useI18n();

const refScroll = ref();
const state = reactive({
  input: '',
  session: null,

  messages: [],
  fetchingMessages: false,

  exchange: null,
  deletingExchanges: [],

  chatSending: false,
  chatReceiving: false,
  chatAbortCtrl: null,
});

const waiting = computed(() => state.chatReceiving || state.chatSending);

const fetchMessages = async () => {
  state.fetchingMessages = true;

  try {
    const chat_history = await callChatHistory(aiStore.getActiveName(), state.session);

    state.messages = chat_history.map((item) => {
      try {
        const json = JSON.parse(item.content);
        item.content = json.content;
        item.reasoning = json.reasoning;
        item.call_tools = json.call_tools;
      } catch {
        // empty
      }

      return item;
    });
  } finally {
    state.fetchingMessages = false;
    nextTick(scrollToBottom);
  }
};

const processData = (data) => {
  if (data.trim().length > 0) {
    state.chatReceiving = true;
  }

  let content, reasoning, call_tool_start, call_tool_end, call_tool_error;
  try {
    const json = JSON.parse(data);
    content = json.content;
    reasoning = json.reasoning;
    call_tool_start = json.call_tool_start;
    call_tool_end = json.call_tool_end;
    call_tool_error = json.call_tool_error;
  } catch {
    content = data;
  }

  const currentMessage = state.messages[state.messages.length - 1];
  if (currentMessage && currentMessage.exchange == state.exchange) {
    if (content) {
      currentMessage.content += content;
    } else if (reasoning) {
      currentMessage.reasoning += reasoning;
    } else if (call_tool_start) {
      currentMessage.call_tools.push({ task: call_tool_start });
    } else if (call_tool_end) {
      const i = currentMessage.call_tools.findIndex((item) => item?.task.id == call_tool_end.id);
      if (i >= 0) {
        currentMessage.call_tools[i]['time'] = call_tool_end['time'];
      }
    } else if (call_tool_error) {
      const i = currentMessage.call_tools.findIndex((item) => item?.task.id == call_tool_error.id);
      if (i >= 0) {
        currentMessage.call_tools[i]['error'] = call_tool_error['error'];
      }
    }
  } else {
    onStop();
  }
};

const onClear = async () => {
  const currentSession = state.session;

  await onStop();
  state.messages = [];
  state.session = ulid();
  const sessionKey = `aiter-session-${aiStore.getActiveName() ?? '~'}`;
  localStorage.setItem(sessionKey, state.session);

  try {
    await callChatClear(aiStore.getActiveName(), currentSession);
  } catch {
    // Allow failure, the history in database may have not been cleared, but here we always start a new session.
  }
};

const onDeleteExchange = async (e) => {
  state.deletingExchanges.push(e.exchange);

  try {
    await callChatDelete(aiStore.getActiveName(), state.session, e.exchange);

    for (let i = state.messages.length - 1; i >= 0; i--) {
      if (state.messages[i].exchange === e.exchange) {
        state.messages.splice(i, 1);
      }
    }
  } finally {
    state.deletingExchanges = state.deletingExchanges.filter((item) => item !== e.exchange);
  }
};

const onReuse = (e) => {
  state.input = e.content;
};

const onSend = async (e) => {
  const message = e.input.trim();
  if (!message) {
    return;
  }

  try {
    state.exchange = ulid();
    state.chatSending = true;

    state.messages.push({
      role: 'user',
      content: e.input,
      exchange: state.exchange,
    });

    state.messages.push({
      role: 'bot',
      content: '',
      reasoning: '',
      call_tools: [],
      exchange: state.exchange,
    });

    nextTick(scrollToBottom);

    let hooks;
    const remoteUrl = await callAppGetRemoteUrl();
    if (remoteUrl) {
      const abortCallback = (abortCtrl) => {
        state.chatAbortCtrl = abortCtrl;
      };

      const eventCallback = (event) => {
        let { data } = event;

        if (data === '[DONE]') {
          return;
        }

        processData(data);

        nextTick(() => {
          scrollToBottom(true);
        });
      };

      hooks = {
        abortCallback,
        eventCallback,
      };
    } else {
      const channel = new Channel();
      channel.onmessage = (data) => {
        processData(data);

        nextTick(() => {
          scrollToBottom(true);
        });
      };

      hooks = {
        channel,
      };
    }

    await callChat(
      aiStore.getActiveName(),
      message,
      state.exchange,
      state.session,
      llmStore.chatLlmName,
      llmStore.reasoningLlmName,
      [`temperature:${chatStore.chatTemperature}`],
      {
        deep: e.deep,
        retrace: chatStore.chatRetrace,
        strict: e.strict,
      },
      hooks,
    );

    if (appStore.mainMenu !== 'chat') {
      chatStore.setNoticed(aiStore.getActiveName(), true);
    }
  } catch (err) {
    const currentMessage = state.messages[state.messages.length - 1];
    if (currentMessage && currentMessage.exchange == state.exchange) {
      currentMessage.content = t('message.error', { error: err.message || err });
    }

    nextTick(scrollToBottom);
  } finally {
    state.chatAbortCtrl?.abort();
    state.chatSending = false;
    state.chatReceiving = false;
  }
};

const onStop = async () => {
  let hooks;
  const remoteUrl = await callAppGetRemoteUrl();
  if (remoteUrl) {
    hooks = {
      chatAbortCtrl: state.chatAbortCtrl,
    };
  } else {
    hooks = {
      exchange: state.exchange,
    };
  }

  await callChatAbort(hooks);

  state.chatSending = false;
  state.chatReceiving = false;

  const currentMessage = state.messages[state.messages.length - 1];
  if (currentMessage && currentMessage.exchange == state.exchange) {
    if (!currentMessage.content) {
      state.messages = state.messages.slice(0, -1);
    }
  }
};

const scrollToBottom = (ignoreAtMiddle) => {
  const wrap = refScroll.value;
  if (wrap) {
    if (ignoreAtMiddle && wrap.scrollTop + wrap.clientHeight < wrap.scrollHeight - 100) {
      return;
    }

    wrap.scrollTop = wrap.scrollHeight - wrap.clientHeight;
  }
};

watch(
  () => aiStore.activeId,
  async () => {
    await onStop();
    state.messages = [];
    const sessionKey = `aiter-session-${aiStore.getActiveName() ?? '~'}`;
    state.session = localStorage.getItem(sessionKey) ?? ulid();
    localStorage.setItem(sessionKey, state.session);

    await fetchMessages();
  },
);

watch(
  () => appStore.mainMenu,
  async (val) => {
    if (val === 'chat') {
      chatStore.setNoticed(aiStore.getActiveName(), false);
    }
  },
);

onMounted(async () => {
  const sessionKey = `aiter-session-${aiStore.getActiveName() ?? '~'}`;
  state.session = localStorage.getItem(sessionKey) ?? ulid();
  localStorage.setItem(sessionKey, state.session);

  if (appStore.mainMenu === 'chat') {
    chatStore.setNoticed(aiStore.getActiveName(), false);
  }

  await fetchMessages();
});
</script>

<template>
  <div class="flex h-full w-full" v-show="visible">
    <ChatSidebar />

    <div class="flex h-full flex-1 flex-col items-center gap-2 p-4">
      <div class="flex w-full flex-1 flex-col items-center overflow-auto" ref="refScroll">
        <div
          class="flex w-full flex-1 flex-col gap-2 px-6 pb-4"
          :class="chatStore.uiExpandWidth ? '' : 'md:w-5/6 xl:w-2/3'"
        >
          <div
            class="border-1 w-full select-none rounded border-solid border-gray-100 bg-gray-50 px-4 py-2 text-gray-400"
            v-if="!state.messages.length"
          >
            <el-icon><i class="ri-robot-3-fill"></i></el-icon>
            <span class="ml-2">
              {{ $t('label.chat_no_message_placeholder', { who: aiStore.getActiveName() ?? $t('label.default_ai') }) }}
            </span>
          </div>

          <template v-for="message in state.messages">
            <template v-if="message.role.toLowerCase() == 'user'">
              <UserMessage
                class="mt-3"
                :key="message.id"
                :message="message"
                :deleting="state.deletingExchanges.findIndex((item) => item === message.exchange) >= 0"
                @delete-exchange="onDeleteExchange"
                @reuse="onReuse"
              />
            </template>
            <template v-else-if="message.role.toLowerCase() == 'bot'">
              <BotMessage
                :key="message.id"
                :message="message"
                :deleting="state.deletingExchanges.findIndex((item) => item === message.exchange) >= 0"
                :receiving="state.chatReceiving"
                :waiting="message.exchange == state.exchange && state.chatSending && !state.chatReceiving"
                @delete-exchange="onDeleteExchange"
              />
            </template>
          </template>

          <div class="mt-4">
            <Transition name="fade">
              <el-button v-if="waiting" @click="onStop">
                <el-icon class="el-icon--left"><i class="ri-stop-fill"></i></el-icon>
                {{ $t('label.chat_interrupt') }}
              </el-button>
            </Transition>
          </div>
        </div>
      </div>

      <div class="flex justify-center" v-if="state.messages.length > 0">
        <el-button size="small" @click="onClear">
          <el-icon class="el-icon--left"><i class="ri-delete-bin-7-line"></i></el-icon>
          {{ $t('label.chat_clear') }}
        </el-button>
      </div>

      <div class="w-full px-4" :class="chatStore.uiExpandWidth ? '' : 'md:w-5/6 xl:w-2/3'">
        <UserInput v-model:input="state.input" :waiting="waiting" @send="onSend" @stop="onStop" />
        <div class="mt-2 text-center text-xs text-gray-400">{{ $t('label.ai_need_screening') }}</div>
      </div>
    </div>
  </div>
</template>
