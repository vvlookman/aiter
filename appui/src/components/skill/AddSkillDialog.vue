<script setup>
import { callSkillAdds } from '@/call';
import { useAiStore } from '@/stores/ai';
import { useAppStore } from '@/stores/app';
import { useToolStore } from '@/stores/tool';
import { onMounted, reactive } from 'vue';

const emit = defineEmits(['added']);
const visible = defineModel('visible');
const aiStore = useAiStore();
const appStore = useAppStore();
const toolStore = useToolStore();

const state = reactive({
  requestingAdd: false,
});

const onAddsFromToolset = async (toolset) => {
  state.requestingAdd = true;

  try {
    const added = await callSkillAdds(aiStore.getActiveName(), toolset.id);
    emit('added', added);

    visible.value = false;
  } finally {
    state.requestingAdd = false;
  }
};

const onGotoSettingsToolMenu = () => {
  appStore.mainMenu = 'settings';
  appStore.settingsMenu = 'tool';

  visible.value = false;
};

onMounted(async () => {
  await toolStore.fetchToolsets();
});
</script>

<template>
  <el-dialog v-model="visible" :title="$t('label.add')" destroy-on-close :show-close="false">
    <div class="max-h-50vh my-4 overflow-auto" v-loading="state.requestingAdd">
      <template v-if="toolStore.toolsets.length > 0">
        <el-button
          class="!ml-0 mr-4 mt-4"
          v-for="toolset in toolStore.toolsets"
          :key="toolset.id"
          @click="onAddsFromToolset(toolset)"
          round
        >
          {{ toolset.title }}
        </el-button>
      </template>
      <template v-else>
        <el-alert :title="$t('message.no_tool')" type="warning" :closable="false" show-icon />
        <div class="w-full text-right">
          <el-button class="mt-4" @click="onGotoSettingsToolMenu" link>
            {{ $t('label.goto', { target: `${$t('label.settings')} > ${$t('label.settings_tool')}` }) }}
          </el-button>
        </div>
      </template>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button type="primary" @click="visible = false">{{ $t('label.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
