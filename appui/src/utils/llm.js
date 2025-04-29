const chatLlms = [
  {
    name: 'openai-compatible',
    protocol: 'openai',
    models: [],
  },
  {
    name: 'deepseek',
    protocol: 'openai',
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
    baseUrl: 'https://api.deepseek.com/v1',
    models: ['deepseek-chat'],
  },
  {
    name: 'qwen',
    protocol: 'openai',
    apiKeyUrl: 'https://bailian.console.aliyun.com/?apiKey=1',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    models: [
      'qwen-max',
      'qwen-max-latest',
      'qwen-plus',
      'qwen-plus-latest',
      'qwen-turbo',
      'qwen-turbo-latest',
      'qwen-long',
      'qwen-long-latest',
    ],
  },
];

const reasoningLlms = [
  {
    name: 'openai-compatible',
    protocol: 'openai',
    models: [],
  },
  {
    name: 'deepseek',
    protocol: 'openai',
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
    baseUrl: 'https://api.deepseek.com/v1',
    models: ['deepseek-reasoner'],
  },
  {
    name: 'qwen',
    protocol: 'openai',
    apiKeyUrl: 'https://bailian.console.aliyun.com/?apiKey=1',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    models: ['qwq-32b', 'qwq-plus', 'qwq-plus-latest'],
  },
];

const guessChatLlm = (llm) => {
  return chatLlms.find((preset) => preset.baseUrl === llm?.options?.base_url);
};

const guessReasoningLlm = (llm) => {
  return reasoningLlms.find((preset) => preset.baseUrl === llm?.options?.base_url);
};

export { chatLlms, guessChatLlm, guessReasoningLlm, reasoningLlms };
