<script setup>
import { callMemVacuum } from '@/call';
import { useAiStore } from '@/stores/ai';
import prettyBytes from 'pretty-bytes';
import { computed, reactive, watch } from 'vue';

const visible = defineModel('visible');
const aiStore = useAiStore();

const state = reactive({
  vacuumingIds: [],
});

const vacuuming = computed(() => state.vacuumingIds.includes(aiStore.activeId));
const memStats = computed(() => aiStore.getActiveMemStats());

const onSwitch = (id) => {
  aiStore.active(id);
  aiStore.fetchMemStats();

  visible.value = false;
};

const onVacuum = async () => {
  state.vacuumingIds.push(aiStore.activeId);

  try {
    await callMemVacuum(aiStore.getActiveName());
    await aiStore.fetchMemStats();
  } finally {
    state.vacuumingIds = state.vacuumingIds.filter((id) => id !== aiStore.activeId);
  }
};

watch(
  () => visible.value,
  (val) => {
    if (val) {
      aiStore.fetchMemStats();
    }
  },
);
</script>

<template>
  <el-dialog v-model="visible" :title="$t('label.switch_ai')" destroy-on-close :show-close="false">
    <div class="max-h-50vh my-4 overflow-auto">
      <el-button class="!ml-0 mr-4 mt-4" @click="onSwitch()" round :type="!aiStore.activeId ? 'primary' : ''">
        {{ $t('label.default_ai') }}
      </el-button>

      <el-button
        class="!ml-0 mr-4 mt-4"
        v-for="ai in aiStore.items"
        :key="ai.id"
        @click="onSwitch(ai.id)"
        round
        :type="ai.id == aiStore.activeId ? 'primary' : ''"
      >
        {{ ai.name }}
      </el-button>
    </div>

    <template #footer>
      <div class="dialog-footer flex items-end">
        <el-popconfirm
          confirm-button-type="warning"
          placement="top"
          width="280"
          :title="$t('label.confirm_vacuum')"
          :confirm-button-text="$t('label.confirm')"
          :cancel-button-text="$t('label.cancel')"
          @confirm="onVacuum"
        >
          <template #reference>
            <el-button v-if="memStats.size" link size="small" type="info">
              <template v-if="vacuuming">
                <el-icon class="rotating el-icon--left"><i class="ri-loader-4-line"></i></el-icon>
                {{ $t('label.vacuuming') }}
              </template>
              <template v-else>
                <el-icon class="el-icon--left"><i class="ri-hard-drive-2-line"></i></el-icon>
                {{ prettyBytes(memStats.size) }}
              </template>
            </el-button>
          </template>
        </el-popconfirm>
        <span class="flex-1"></span>
        <el-button type="primary" @click="visible = false">{{ $t('label.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
