<script setup>
import { callToolImport, callToolParse } from '@/call';
import { useToolStore } from '@/stores/tool';
import { invoke } from '@tauri-apps/api/core';
import JsonEditorVue from 'json-editor-vue';
import { onMounted, reactive, watch } from 'vue';

const emit = defineEmits(['added']);
const toolStore = useToolStore();

const state = reactive({
  currentType: 'AHP',
  currentIsTesting: false,
  currentTestResult: [],
  currentIsSaving: false,

  ahpUrl: 'http://',
  ahpHeadersJson: {},

  mcpJson: {
    mcpServers: {
      'sample-mcp': {
        command: 'npx',
        args: ['-y', 'mcp-package'],
        env: {
          API_KEY: '',
        },
      },
    },
  },
  mcpSupportNpx: false,
  mcpSupportUv: false,
});

const visible = defineModel('visible');

const generateToolsetsOptions = () => {
  const toolsetsOptions = [];

  if (state.currentType == 'AHP') {
    if (!state.ahpUrl.trim().length) {
      throw new Error('No AHP URL defined');
    }

    const options = [];
    options.push(`url:${state.ahpUrl}`);

    if (state.ahpHeadersJson) {
      for (const headerKey in state.ahpHeadersJson) {
        options.push(`header:${headerKey}:${state.ahpHeadersJson[headerKey]}`);
      }
    }

    toolsetsOptions.push(options);
  } else if (state.currentType == 'MCP') {
    if (!state.mcpJson.mcpServers) {
      throw new Error('No MCP server defined');
    }

    for (const k in state.mcpJson.mcpServers) {
      const mcp = state.mcpJson.mcpServers[k];

      const options = [];
      if (mcp.command) {
        options.push(`cmd:${mcp.command}`);
      }
      if (mcp.args) {
        for (const arg of mcp.args) {
          options.push(`arg:${arg}`);
        }
      }
      if (mcp.env) {
        for (const envKey in mcp.env) {
          options.push(`env:${envKey}:${mcp.env[envKey]}`);
        }
      }

      toolsetsOptions.push(options);
    }
  }

  return toolsetsOptions;
};

const onSave = async () => {
  state.currentIsSaving = true;

  try {
    const toolsets = [];

    const toolsetsOptions = generateToolsetsOptions();
    for (const options of toolsetsOptions) {
      const tools = await callToolImport(state.currentType, null, options);
      if (tools.length > 0) {
        const tool = tools[0];
        toolsets.push({
          id: tool.toolset_id,
          title: tool.toolset_title,
        });
      }
    }

    if (toolsets.length > 0) {
      await toolStore.fetchToolsets();

      emit('added', toolsets);
    }
  } finally {
    state.currentIsSaving = false;
  }

  visible.value = false;
};

const onTest = async () => {
  state.currentIsTesting = true;

  try {
    const toolsets = [];

    const toolsetsOptions = generateToolsetsOptions();
    for (const options of toolsetsOptions) {
      const tools = await callToolParse(state.currentType, options);
      if (tools.length > 0) {
        const tool = tools[0];
        toolsets.push({
          id: tool.toolset_id,
          title: tool.toolset_title,
        });
      }
    }

    state.currentTestResult = toolsets;
  } catch (err) {
    state.currentTestResult = [];
    throw err;
  } finally {
    state.currentIsTesting = false;
  }
};

onMounted(async () => {
  state.mcpSupportNpx = await invoke('is_npx_installed');
  state.mcpSupportUv = await invoke('is_uv_installed');
});

watch(
  () => state.currentType,
  () => {
    state.currentIsTesting = false;
    state.currentTestResult = [];
  },
);
</script>

<template>
  <el-dialog v-model="visible" :title="$t('label.add')" destroy-on-close>
    <div class="flex flex-wrap">
      <el-button
        class="!ml-0 mb-4 mr-4"
        @click="state.currentType = 'AHP'"
        round
        :type="state.currentType == 'AHP' ? 'primary' : ''"
      >
        AHP
      </el-button>

      <el-button
        class="!ml-0 mb-4 mr-4"
        @click="state.currentType = 'MCP'"
        round
        :type="state.currentType == 'MCP' ? 'primary' : ''"
      >
        MCP
      </el-button>
    </div>

    <div class="max-h-50vh overflow-auto" v-if="state.currentType == 'AHP'">
      <el-input v-model="state.ahpUrl" :placeholder="$t('label.tool_ahp_url_placeholder')"></el-input>

      <div class="mt-4">{{ $t('label.tool_ahp_headers') }}</div>
      <JsonEditorVue
        v-model="state.ahpHeadersJson"
        :mainMenuBar="false"
        :statusBar="false"
        :navigationBar="false"
        :stringified="false"
        mode="text"
      />
    </div>
    <div class="max-h-50vh overflow-auto" v-else-if="state.currentType == 'MCP'">
      <JsonEditorVue
        v-model="state.mcpJson"
        :mainMenuBar="false"
        :statusBar="false"
        :navigationBar="false"
        :stringified="false"
        mode="text"
      />
      <div class="mt-4 flex items-center">
        <el-tag class="mr-2" v-if="state.mcpSupportNpx" size="small" disable-transitions>
          <el-icon><i class="ri-checkbox-circle-fill"></i></el-icon>
          <span class="ml-2">npx</span>
        </el-tag>

        <el-tag class="mr-2" v-if="state.mcpSupportUv" size="small" disable-transitions>
          <el-icon><i class="ri-checkbox-circle-fill"></i></el-icon>
          <span class="ml-2">uv</span>
        </el-tag>
      </div>
    </div>

    <transition name="fade">
      <div class="mt-4 flex flex-wrap gap-2 text-xs font-light text-gray-400" v-if="state.currentTestResult.length > 0">
        <div v-for="toolset in state.currentTestResult" :key="toolset.id">
          <el-icon><i class="ri-checkbox-circle-fill"></i></el-icon>
          {{ toolset.title }}
        </div>
      </div>
    </transition>

    <template #footer>
      <div class="dialog-footer flex">
        <el-button :disabled="!state.ahpUrl || state.currentIsTesting" @click="onTest">
          <template v-if="state.currentIsTesting">
            <el-icon class="rotating el-icon--left"><i class="ri-loader-4-line"></i></el-icon>
          </template>
          <template v-else>
            <el-icon class="el-icon--left"><i class="ri-send-plane-line"></i></el-icon>
          </template>
          {{ $t('label.test') }}
        </el-button>

        <span class="flex-1"></span>

        <el-button @click="visible = false">{{ $t('label.cancel') }}</el-button>
        <el-button type="primary" @click="onSave" :disabled="state.currentIsSaving">
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
    </template>
  </el-dialog>
</template>
