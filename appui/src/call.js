/*
 * Call functions via IPC or HTTP, depending on if remoteUrl is set.
 */

import api from '@/api';
import { invoke } from '@tauri-apps/api/core';

async function getAppConfigRemoteUrl() {
  const appConfig = (await invoke('app_config')) ?? {};
  return appConfig.remote_url;
}

export async function callAppGetRemoteUrl() {
  return await invoke('app_get_remote_url');
}

export async function callAppSetRemote(remoteUrl, remoteToken) {
  return await invoke('app_set_remote', {
    remoteUrl,
    remoteToken,
  });
}

export async function callCoreVersion() {
  if (await getAppConfigRemoteUrl()) {
    return await api.get('/version');
  } else {
    return await invoke('core_version');
  }
}

export async function callAiAdd(name) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/ai/add', { name });
  } else {
    return await invoke('ai_add', { name });
  }
}

export async function callAiDelete(name) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/ai/delete', { name });
  } else {
    return await invoke('ai_delete', { name });
  }
}

export async function callAiList() {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/ai/list');
  } else {
    return await invoke('ai_list');
  }
}

export async function callAiRename(name, newName) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/ai/rename', { name, new_name: newName });
  } else {
    return await invoke('ai_rename', {
      name,
      newName,
    });
  }
}

export async function callChat(
  ai,
  message,
  exchange,
  session,
  llmForChat,
  llmForReasoning,
  llmOptions,
  options,
  hooks,
) {
  if (await getAppConfigRemoteUrl()) {
    return await api.sse(
      '/chat/',
      {
        ai,
        message,
        exchange,
        session,
        llm_for_chat: llmForChat,
        llm_for_reasoning: llmForReasoning,
        llm_options: llmOptions,
        deep: options.deep,
        retrace: options.retrace,
        strict: options.strict,
      },
      hooks,
    );
  } else {
    return await invoke('chat', {
      ai,
      message,
      exchange,
      session,
      llmForChat,
      llmForReasoning,
      llmOptions,
      deep: options.deep,
      retrace: options.retrace,
      strict: options.strict,
      channel: hooks.channel,
    });
  }
}

export async function callChatAbort(hooks) {
  if (await getAppConfigRemoteUrl()) {
    return hooks.chatAbortCtrl?.abort();
  } else {
    if (hooks.exchange) {
      return await invoke('chat_abort', hooks);
    }
  }
}

export async function callChatClear(ai, session) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/chat/clear', {
      ai,
      session,
    });
  } else {
    return await invoke('chat_clear', {
      ai,
      session,
    });
  }
}

export async function callChatDelete(ai, session, exchange) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/chat/delete', {
      ai,
      session,
      exchange,
    });
  } else {
    return await invoke('chat_delete', {
      ai,
      session,
      exchange,
    });
  }
}

export async function callChatHistory(ai, session) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/chat/history', {
      ai,
      session,
    });
  } else {
    return await invoke('chat_history', {
      ai,
      session,
    });
  }
}

export async function callDocCountPart(ai, id) {
  if (await getAppConfigRemoteUrl()) {
    const count = await api.post('/doc/count-part', {
      ai,
      id,
    });

    return parseInt(count) ?? 0;
  } else {
    return await invoke('doc_count_part', {
      ai,
      id,
    });
  }
}

export async function callDocDelete(ai, id) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/doc/delete', {
      ai,
      id,
    });
  } else {
    return await invoke('doc_delete', {
      ai,
      id,
    });
  }
}

export async function callDocGetPart(ai, id, index) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/doc/get-part', {
      ai,
      id,
      index,
    });
  } else {
    return await invoke('doc_get_part', {
      ai,
      id,
      index,
    });
  }
}

export async function callDocLearn(ai, file) {
  if (await getAppConfigRemoteUrl()) {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('ai', ai || '');
    formData.append('filename', file.name);

    return await api.postForm(`/doc/learn`, formData);
  } else {
    const fileData = await file.arrayBuffer();
    const fileName = file.name;

    return await invoke('doc_learn', {
      ai,
      fileData,
      fileName,
    });
  }
}

export async function callDocList(ai, search, limit, offset) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/doc/list', {
      ai,
      search,
      limit,
      offset,
    });
  } else {
    return await invoke('doc_list', {
      ai,
      search,
      limit,
      offset,
    });
  }
}

export async function callDocListByIds(ai, ids) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/doc/list-by-ids', {
      ai,
      ids,
    });
  } else {
    return await invoke('doc_list_by_ids', {
      ai,
      ids,
    });
  }
}

export async function callDocListDigestingIds(ai, limit) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/doc/list-digesting-ids', {
      ai,
      limit,
    });
  } else {
    return await invoke('doc_list_digesting_ids', {
      ai,
      limit,
    });
  }
}

export async function callLlmActive(type, name) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/active', {
      type,
      name,
    });
  } else {
    return await invoke('llm_active', {
      type,
      name,
    });
  }
}

export async function callLlmConfig(name, type, protocol, options) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/config', {
      name,
      type,
      protocol,
      options,
    });
  } else {
    return await invoke('llm_config', {
      name,
      type,
      protocol,
      options,
    });
  }
}

export async function callLlmDelete(name) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/delete', {
      name,
    });
  } else {
    return await invoke('llm_delete', {
      name,
    });
  }
}

export async function callLlmEdit(oldName, name, protocol, options) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/edit', {
      old_name: oldName,
      name,
      protocol,
      options,
    });
  } else {
    return await invoke('llm_edit', {
      oldName,
      name,
      protocol,
      options,
    });
  }
}

export async function callLlmList() {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/list');
  } else {
    return await invoke('llm_list');
  }
}

export async function callLlmListActivedNames() {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/llm/list-actived-names');
  } else {
    return await invoke('llm_list_actived_names');
  }
}

export async function callLlmTestChat(prompt, exchange, name, protocol, options, hooks) {
  if (await getAppConfigRemoteUrl()) {
    return await api.sse(
      '/llm/test-chat',
      {
        prompt,
        exchange,
        name,
        protocol,
        options,
      },
      hooks,
    );
  } else {
    return await invoke('llm_test_chat', {
      prompt,
      exchange,
      name,
      protocol,
      options,
      channel: hooks.channel,
    });
  }
}

export async function callMemStats(ai) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/mem/stats', {
      ai,
    });
  } else {
    return await invoke('mem_stats', {
      ai,
    });
  }
}

export async function callMemVacuum(ai) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/mem/vacuum', {
      ai,
    });
  } else {
    return await invoke('mem_vacuum', {
      ai,
    });
  }
}

export async function callSkillAdd(ai, toolId, trigger) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/skill/add', {
      ai,
      tool_id: toolId,
      trigger,
    });
  } else {
    return await invoke('skill_add', {
      ai,
      toolId,
      trigger,
    });
  }
}

export async function callSkillAdds(ai, toolsetId) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/skill/adds', {
      ai,
      toolset_id: toolsetId,
    });
  } else {
    return await invoke('skill_adds', {
      ai,
      toolsetId,
    });
  }
}

export async function callSkillDelete(ai, id) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/skill/delete', {
      ai,
      id,
    });
  } else {
    return await invoke('skill_delete', {
      ai,
      id,
    });
  }
}

export async function callSkillList(ai, search, limit, offset) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/skill/list', {
      ai,
      search,
      limit,
      offset,
    });
  } else {
    return await invoke('skill_list', {
      ai,
      search,
      limit,
      offset,
    });
  }
}

export async function callToolDeleteByToolset(toolsetId) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/delete-by-toolset', {
      toolset_id: toolsetId,
    });
  } else {
    return await invoke('tool_delete_by_toolset', {
      toolsetId,
    });
  }
}

export async function callToolGet(id) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/get', {
      id,
    });
  } else {
    return await invoke('tool_get', {
      id,
    });
  }
}

export async function callToolImport(type, title, options) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/import', {
      type,
      title,
      options,
    });
  } else {
    return await invoke('tool_import', {
      type,
      title,
      options,
    });
  }
}

export async function callToolListByIds(ids) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/list-by-ids', {
      ids,
    });
  } else {
    return await invoke('tool_list_by_ids', {
      ids,
    });
  }
}

export async function callToolListToolsets() {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/list-toolsets');
  } else {
    return await invoke('tool_list_toolsets');
  }
}

export async function callToolParse(type, options) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/parse', {
      type,
      options,
    });
  } else {
    return await invoke('tool_parse', {
      type,
      options,
    });
  }
}

export async function callToolQueryByToolset(toolsetId) {
  if (await getAppConfigRemoteUrl()) {
    return await api.post('/tool/query-by-toolset', {
      toolset_id: toolsetId,
    });
  } else {
    return await invoke('tool_query_by_toolset', {
      toolsetId,
    });
  }
}
