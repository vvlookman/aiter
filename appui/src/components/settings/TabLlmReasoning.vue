<script setup>
import { callAppGetRemoteUrl, callLlmActive, callLlmDelete, callLlmEdit, callLlmTestChat } from '@/call';
import AddReasoningLlmDialog from '@/components/settings/AddReasoningLlmDialog.vue';
import SettingsMenuItem from '@/components/settings/SettingsMenuItem.vue';
import { useLlmStore } from '@/stores/llm';
import { guessReasoningLlm } from '@/utils/llm';
import { Channel } from '@tauri-apps/api/core';
import { ClickOutside as vClickOutside } from 'element-plus';
import { ElMessage, ElMessageBox } from 'element-plus';
import { computed, onMounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const llmStore = useLlmStore();

const state = reactive({
  showAddReasoningLlmDialog: false,
  requestingDeleteLlmNames: [],

  currentName: '',
  currentIsSaving: false,
  currentIsTesting: false,
  currentIsTestingName: null,
  currentShowTestContent: false,
  currentTestContent: '',
});

const currentLlm = reactive({
  apiKey: '',
  baseUrl: '',
  model: '',
  name: '',
  protocol: '',
});

const currentLlmPreset = computed(() => {
  const llm = llmStore.getByName(state.currentName);
  return guessReasoningLlm(llm);
});

const onAdded = (llm) => {
  onSelected(llm.name);
};

const onConfirmDeleteLlm = (llm) => {
  ElMessageBox.confirm(t('label.confirm_delete', { name: llm.name }), t('label.warning'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
    confirmButtonClass: 'el-button--danger',
    type: 'warning',
  })
    .then(() => {
      requestDeleteLlm(llm);
    })
    .catch(() => {});
};

const onSave = async () => {
  state.currentIsSaving = true;

  try {
    const oldName = state.currentName;
    const name = currentLlm.name;
    const protocol = currentLlm.protocol;
    const options = {
      api_key: currentLlm.apiKey,
      base_url: currentLlm.baseUrl,
      model: currentLlm.model,
    };

    const saved = await callLlmEdit(oldName, name, protocol, options);
    llmStore.upsert(oldName, saved);

    state.currentName = saved.name;

    ElMessage({
      type: 'success',
      message: t('message.success_save'),
    });
  } finally {
    state.currentIsSaving = false;
  }
};

const onPromptInputModel = () => {
  ElMessageBox.prompt('', t('label.llm.input_model'), {
    confirmButtonText: t('label.confirm'),
    cancelButtonText: t('label.cancel'),
  })
    .then(({ value }) => {
      let name = value.trim();

      if (!name.length) {
        ElMessage.error(t('message.invalid_name'));
      } else {
        currentLlm.model = name;
      }
    })
    .catch(() => {});
};

const onSelected = (name) => {
  state.currentName = name;
};

const onSetCurrentAsDefault = async () => {
  await callLlmActive('reasoning', currentLlm.name);
  llmStore.defaultReasoningLlmName = currentLlm.name;
};

const onTest = async () => {
  state.currentIsTesting = true;

  try {
    state.currentIsTestingName = state.currentName;
    state.currentTestContent = '';
    state.currentShowTestContent = true;

    let hooks;
    const remoteUrl = await callAppGetRemoteUrl();
    if (remoteUrl) {
      const eventCallback = (event) => {
        let { data } = event;

        if (data === '[DONE]') {
          return;
        }

        if (state.currentIsTestingName !== state.currentName) {
          state.currentShowTestContent = false;
          return;
        }

        if (data.trim().length > 0) {
          state.currentShowTestContent = true;
        }

        state.currentTestContent = state.currentTestContent + data;
      };

      hooks = {
        eventCallback,
      };
    } else {
      const channel = new Channel();
      channel.onmessage = (data) => {
        if (state.currentIsTestingName !== state.currentName) {
          state.currentShowTestContent = false;
          return;
        }

        if (data.trim().length > 0) {
          state.currentShowTestContent = true;
        }

        state.currentTestContent = state.currentTestContent + data;
      };

      hooks = {
        channel,
      };
    }

    const name = currentLlm.name;
    const protocol = currentLlm.protocol;
    const options = {
      api_key: currentLlm.apiKey,
      base_url: currentLlm.baseUrl,
      model: currentLlm.model,
    };

    await callLlmTestChat(t('message.who_are_you'), name, protocol, options, hooks);
  } finally {
    state.currentIsTesting = false;
  }
};

const requestDeleteLlm = async (llm) => {
  state.requestingDeleteLlmNames.push(llm.name);

  try {
    const deleted = await callLlmDelete(llm.name);
    llmStore.delete(deleted.name);

    selectFirstLlm();
  } finally {
    state.requestingDeleteLlmNames.splice(state.requestingDeleteLlmNames.indexOf(llm.name), 1);
  }
};

const selectFirstLlm = () => {
  const llms = llmStore.listByType('reasoning');
  if (llms.length > 0) {
    onSelected(llms[0].name);
  }
};

onMounted(async () => {
  await llmStore.fetch();
  await llmStore.fetchActivedNames();

  selectFirstLlm();
});

watch(
  () => state.currentName,
  (name) => {
    const llm = llmStore.getByName(name);
    if (llm) {
      currentLlm.name = llm.name;
      currentLlm.protocol = llm.protocol;
      currentLlm.apiKey = llm.options.api_key;
      currentLlm.baseUrl = llm.options.base_url;
      currentLlm.model = llm.options.model;
    }
  },
);
</script>

<template>
  <div class="flex h-full w-full">
    <div class="border-l-1 flex flex-col gap-2 border-neutral-100 bg-white p-4">
      <el-button class="mb-2 min-w-48" round @click="state.showAddReasoningLlmDialog = true">
        <el-icon><i class="ri-add-line"></i></el-icon>
        {{ $t('label.add') }}
      </el-button>

      <SettingsMenuItem
        v-for="llm in llmStore.listByType('reasoning')"
        :key="llm.name"
        @select="onSelected(llm.name)"
        :active="state.currentName === llm.name"
      >
        <div class="flex w-full items-center">
          <template v-if="guessReasoningLlm(llm)">
            <el-image
              class="border-1 h-6 w-6 rounded border-neutral-200"
              :src="`/vendor/${guessReasoningLlm(llm).name}.png`"
              fit="contain"
            >
              <template #error>
                <div class="text-center">
                  <el-icon><i class="ri-cpu-line"></i></el-icon>
                </div>
              </template>
            </el-image>
          </template>

          <span class="mx-2">{{ llm.name }}</span>

          <template v-if="llm.name === llmStore.defaultReasoningLlmName">
            <span class="text-green-500">●</span>
          </template>

          <span class="min-w-20 flex-1"></span>

          <el-button @click="onConfirmDeleteLlm(llm)" link type="danger">
            <template v-if="state.requestingDeleteLlmNames.includes(llm.name)">
              <el-icon class="rotating">
                <i class="ri-loader-4-line"></i>
              </el-icon>
            </template>
            <template v-else>
              <el-icon><i class="ri-delete-bin-6-line"></i></el-icon>
            </template>
          </el-button>
        </div>
      </SettingsMenuItem>
    </div>

    <div class="flex flex-1 flex-col gap-2 p-2" v-if="state.currentName">
      <div class="flex flex-1 flex-col gap-2 overflow-auto">
        <div class="px-2 py-4">
          <div class="flex">
            <span class="flex-1"></span>
            <el-button
              @click="onSetCurrentAsDefault"
              :disabled="state.currentName === llmStore.defaultReasoningLlmName"
            >
              <span class="mr-2 text-green-500">●</span>
              {{ $t('label.set_as_default') }}
            </el-button>
          </div>

          <el-form label-position="top">
            <el-form-item :label="$t('label.llm.name')">
              <el-input v-model="currentLlm.name" />
            </el-form-item>

            <el-form-item :label="$t('label.llm.api_key')">
              <div class="flex w-full gap-2">
                <el-input v-model="currentLlm.apiKey" type="password" show-password />

                <el-popover
                  :content="state.currentTestContent"
                  :visible="state.currentShowTestContent && !!state.currentTestContent"
                  :width="300"
                  placement="bottom-end"
                >
                  <template #reference>
                    <el-button
                      v-click-outside="
                        () => {
                          state.testAbortCtrl?.abort();
                          state.currentShowTestContent = false;
                        }
                      "
                      :disabled="state.currentIsTesting"
                      @click="onTest"
                    >
                      <template v-if="state.currentIsTesting">
                        <el-icon class="rotating el-icon--left"><i class="ri-loader-4-line"></i></el-icon>
                      </template>
                      <template v-else>
                        <el-icon class="el-icon--left"><i class="ri-send-plane-line"></i></el-icon>
                      </template>
                      {{ $t('label.test') }}
                    </el-button>
                  </template>
                </el-popover>
              </div>

              <div v-if="currentLlmPreset?.apiKeyUrl">
                <el-link :underline="false" :href="currentLlmPreset.apiKeyUrl" target="_blank">
                  <div class="text-xs">
                    <el-icon><i class="ri-link"></i></el-icon>
                    {{ $t('label.llm.api_key_url') }}
                  </div>
                </el-link>
              </div>
            </el-form-item>

            <el-form-item :label="$t('label.llm.base_url')">
              <el-input v-model="currentLlm.baseUrl" />
            </el-form-item>

            <el-form-item :label="$t('label.llm.protocol')">
              <el-select v-model="currentLlm.protocol" disabled placeholder="">
                <el-option label="OpenAI" value="openai" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('label.llm.model')">
              <el-button
                class="!ml-0 mr-2 mt-2"
                v-for="model in currentLlmPreset?.models"
                :key="model"
                @click="currentLlm.model = model"
                round
                :type="model == currentLlm.model ? 'primary' : ''"
              >
                {{ model }}
              </el-button>

              <el-button
                class="!ml-0 mr-2 mt-2"
                v-if="!currentLlmPreset?.models.includes(currentLlm.model) && currentLlm.model"
                round
                type="primary"
              >
                {{ currentLlm.model }}
              </el-button>

              <el-button class="!ml-0 mr-2 mt-2" @click="onPromptInputModel()" round type="info">
                {{ $t('label.llm.input_model') }}
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <div class="flex px-2">
        <el-button class="flex-1" type="primary" :disabled="state.currentIsSaving" @click="onSave">
          <template v-if="state.currentIsSaving">
            <el-icon class="rotating">
              <i class="ri-loader-4-line"></i>
            </el-icon>
          </template>
          <template v-else>
            {{ $t('label.save') }}
          </template>
        </el-button>
      </div>
    </div>
  </div>

  <AddReasoningLlmDialog v-model:visible="state.showAddReasoningLlmDialog" @added="onAdded" />
</template>
