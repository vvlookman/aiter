/*
 * Call functions via IPC or HTTP, depending on if baseUrl is set.
 */

import api from '@/api';
import { invoke } from '@tauri-apps/api/core';

const baseUrl = localStorage.getItem('aiter-base-url');

export function isRemoteCall() {
  return !!baseUrl;
}

export async function callCoreVersion() {
  if (baseUrl) {
    return await api.get('/version');
  } else {
    return await invoke('core_version');
  }
}

export async function callAiAdd(name) {
  if (baseUrl) {
    return await api.post('/ai/add', { name });
  } else {
    return await invoke('ai_add', { name });
  }
}

export async function callAiDelete(name) {
  if (baseUrl) {
    return await api.post('/ai/delete', { name });
  } else {
    return await invoke('ai_delete', { name });
  }
}

export async function callAiList() {
  if (baseUrl) {
    return await api.post('/ai/list');
  } else {
    return await invoke('ai_list');
  }
}

export async function callAiRename(name, newName) {
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
    return hooks.chatAbortCtrl?.abort();
  } else {
    return await invoke('chat_abort', hooks);
  }
}

export async function callChatClear(ai, session) {
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
    return await api.post('/llm/list');
  } else {
    return await invoke('llm_list');
  }
}

export async function callLlmListActivedNames() {
  if (baseUrl) {
    return await api.post('/llm/list-actived-names');
  } else {
    return await invoke('llm_list_actived_names');
  }
}

export async function callLlmTestChat(prompt, name, protocol, options, timeoutSecs, hooks) {
  if (baseUrl) {
    return await api.sse(
      '/llm/test-chat',
      {
        prompt,
        name,
        protocol,
        options,
        timeout_secs: timeoutSecs,
      },
      hooks,
    );
  } else {
    return await invoke('llm_test_chat', {
      prompt,
      name,
      protocol,
      options,
      timeoutSecs,
      channel: hooks.channel,
    });
  }
}

export async function callSkillAdd(ai, toolId, trigger) {
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
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
  if (baseUrl) {
    return await api.post('/tool/list-toolsets');
  } else {
    return await invoke('tool_list_toolsets');
  }
}

export async function callToolParse(type, options) {
  if (baseUrl) {
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
  if (baseUrl) {
    return await api.post('/tool/query-by-toolset', {
      toolset_id: toolsetId,
    });
  } else {
    return await invoke('tool_query_by_toolset', {
      toolsetId,
    });
  }
}
