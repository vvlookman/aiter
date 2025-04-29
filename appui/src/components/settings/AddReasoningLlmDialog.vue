<script setup>
import { callLlmConfig } from '@/call';
import { useLlmStore } from '@/stores/llm';
import { reasoningLlms } from '@/utils/llm';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['added']);
const visible = defineModel('visible');
const { t } = useI18n();
const llmStore = useLlmStore();

const onSelect = async (preset) => {
  let name = t(`vendor.${preset.name}`);
  while (llmStore.getByName(name)) {
    name = name + '-1';
  }

  const protocol = preset.protocol;
  const options = {
    base_url: preset.baseUrl,
    model: preset.models[0],
  };

  const added = await callLlmConfig(name, 'reasoning', protocol, options);
  llmStore.upsert(name, added);

  emit('added', added);

  visible.value = false;
};
</script>

<template>
  <el-dialog v-model="visible" :title="$t('label.add')" destroy-on-close>
    <div class="max-h-50vh my-4 overflow-auto">
      <el-button
        class="!ml-0 mr-4 mt-4"
        v-for="preset in reasoningLlms"
        @click="onSelect(preset)"
        round
        :key="preset.name"
      >
        <template v-if="!preset.name.endsWith('-compatible')">
          <el-image class="mr-2 h-6 w-6 rounded" :src="`/vendor/${preset.name}.png`" fit="contain" />
        </template>
        {{ $t(`vendor.${preset.name}`) }}
      </el-button>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button type="primary" @click="visible = false">{{ $t('label.cancel') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
