import type { ProviderId, ModelTierConfig, ModelCapability } from './types.js';


export const DOWNLOADABLE_OLLAMA_MODELS = [
  { label: "qwen2.5-coder:32b (🦁 Flagship 32B Code Specialist)", value: "qwen2.5-coder:32b" },
  { label: "qwen2.5-coder:14b (⚡ Balanced 14B Daily Coder)", value: "qwen2.5-coder:14b" },
  { label: "qwen2.5-coder:7b (🐇 Fast 7B Subagent Helper)", value: "qwen2.5-coder:7b" },
  { label: "deepseek-r1:32b (👑 32B Deep Reasoning)", value: "deepseek-r1:32b" },
  { label: "deepseek-r1:14b (🦁 14B Fast Reasoning)", value: "deepseek-r1:14b" },
  { label: "deepseek-r1:8b (⚡ 8B Compact Reasoning)", value: "deepseek-r1:8b" },
  { label: "llama3.3:70b (👑 70B Frontier Model)", value: "llama3.3:70b" },
  { label: "llama3.2:latest (🐇 3B Ultra-lightweight Helper)", value: "llama3.2:latest" },
  { label: "codestral:22b (🦁 Mistral 22B Code Specialist)", value: "codestral:22b" },
];

export const CAPABILITY_ICONS: Record<ModelCapability, { icon: string; label: string }> = {
  thinking: { icon: '🧠', label: 'Thinking' },
  vision: { icon: '👁️', label: 'Vision' },
  tools: { icon: '🛠️', label: 'Tools' },
  fast: { icon: '⚡', label: 'Fast' },
  cloud: { icon: '🌐', label: 'Cloud' },
  local: { icon: '🔒', label: 'Local' },
};

/**
 * Renders capability badges string, e.g. "🧠 👁️ 🛠️"
 */
export function formatCapabilities(caps: ModelCapability[]): string {
  if (!caps || caps.length === 0) return '';
  return caps.map((c) => CAPABILITY_ICONS[c]?.icon || '').filter(Boolean).join(' ');
}

/**
 * Automatically infers model capabilities from model name/slug
 */
export function inferCapabilities(modelName: string, isCloudHint = false): ModelCapability[] {
  const m = modelName.toLowerCase();
  const caps: ModelCapability[] = [];

  // Thinking / Reasoning
  if (
    m.includes('r1') ||
    m.includes('thinking') ||
    m.includes('reason') ||
    m.includes('o1') ||
    m.includes('o3') ||
    m.includes('fable') ||
    m.includes('mythos') ||
    m.includes('sonnet-3.7') ||
    m.includes('3.7-sonnet') ||
    m.includes('pro')
  ) {
    caps.push('thinking');
  }

  // Vision / Multimodal
  if (
    m.includes('vision') ||
    m.includes('vl') ||
    m.includes('gemini') ||
    m.includes('claude-3') ||
    m.includes('claude-fable') ||
    m.includes('4o') ||
    m.includes('gemma')
  ) {
    caps.push('vision');
  }

  // Tool Use / Function Calling (Standard for coding models)
  if (
    m.includes('coder') ||
    m.includes('claude') ||
    m.includes('qwen') ||
    m.includes('deepseek') ||
    m.includes('llama') ||
    m.includes('codestral') ||
    m.includes('jan-code') ||
    m.includes('gpt')
  ) {
    caps.push('tools');
  }

  // Fast / Low-latency
  if (
    m.includes('flash') ||
    m.includes('haiku') ||
    m.includes('lite') ||
    m.includes('mini') ||
    m.includes('1.5b') ||
    m.includes('3b') ||
    m.includes('7b') ||
    m.includes('8b')
  ) {
    caps.push('fast');
  }

  // Cloud vs Local
  if (m.includes(':cloud') || isCloudHint || m.startsWith('anthropic/') || m.startsWith('google/') || m.startsWith('meta-llama/')) {
    caps.push('cloud');
  } else if (!m.includes('http') && !m.includes('/')) {
    caps.push('local');
  }

  return Array.from(new Set(caps));
}

export interface ModelOption {
  label: string;
  value: string;
  hint?: string;
  equivalentTo?: string;
  capabilities?: ModelCapability[];
}

export interface ProviderDefinition {
  id: ProviderId;
  name: string;
  badge: string;
  description: string;
  defaultBaseUrl?: string;
  requiresKey: boolean;
  keyEnvVar?: string;
  defaultModels: ModelTierConfig;
  suggestedModels: {
    epic: ModelOption[];
    large: ModelOption[];
    medium: ModelOption[];
    haiku: ModelOption[];
  };
}

export const EQUIVALENCE_GUIDE = {
  epic: {
    tierName: '👑 Epic Tier',
    role: 'Autonomous multi-step agents, long-horizon planning & frontier reasoning',
    badges: ['🧠', '👁️', '🛠️'],
    equivalents: [
      { provider: 'Anthropic Direct', model: 'claude-fable-5', caps: ['🧠', '👁️', '🛠️'] },
      { provider: 'OpenRouter', model: 'anthropic/claude-fable-5 or deepseek/deepseek-r1', caps: ['🧠', '🛠️'] },
      { provider: 'Google / Gemini', model: 'google/gemini-2.5-pro', caps: ['🧠', '👁️', '🛠️'] },
      { provider: 'Ollama Cloud', model: 'deepseek-v4-pro:cloud / deepseek-r1:cloud', caps: ['🧠', '🛠️', '🌐'] },
      { provider: 'Ollama Local', model: 'deepseek-r1:70b / llama3.3:70b', caps: ['🧠', '🛠️', '🔒'] },
    ],
  },
  large: {
    tierName: '🦁 Large Tier',
    role: 'Flagship coding, hybrid reasoning & heavy architecture (Opus tier)',
    badges: ['🧠', '🛠️'],
    equivalents: [
      { provider: 'Anthropic Direct', model: 'claude-3-7-sonnet-latest or claude-3-opus-latest', caps: ['🧠', '👁️', '🛠️'] },
      { provider: 'OpenRouter', model: 'anthropic/claude-3.7-sonnet or qwen/qwen-2.5-coder-32b-instruct', caps: ['🧠', '🛠️'] },
      { provider: 'Google / Gemini', model: 'google/gemini-2.5-pro', caps: ['🧠', '👁️', '🛠️'] },
      { provider: 'Ollama Cloud', model: 'deepseek-v4-flash:cloud / qwen2.5-coder:cloud', caps: ['⚡', '🛠️', '🌐'] },
      { provider: 'Ollama Local', model: 'qwen2.5-coder:32b / deepseek-r1:32b', caps: ['🧠', '🛠️', '🔒'] },
    ],
  },
  medium: {
    tierName: '⚡ Medium Tier',
    role: 'Fast, reliable daily coding driver & refactoring (Sonnet tier)',
    badges: ['⚡', '🛠️'],
    equivalents: [
      { provider: 'Anthropic Direct', model: 'claude-3-5-sonnet-latest', caps: ['👁️', '🛠️'] },
      { provider: 'OpenRouter', model: 'anthropic/claude-3.5-sonnet or deepseek/deepseek-chat', caps: ['⚡', '🛠️'] },
      { provider: 'Google / Gemini', model: 'google/gemini-2.5-flash', caps: ['⚡', '👁️', '🛠️'] },
      { provider: 'Ollama Cloud', model: 'deepseek-v4-flash:cloud / qwen3.5:cloud', caps: ['⚡', '🛠️', '🌐'] },
      { provider: 'Ollama Local', model: 'qwen2.5-coder:14b / llama3.1:8b', caps: ['⚡', '🛠️', '🔒'] },
      { provider: 'Jan Local', model: 'janhq/Jan-code-4b', caps: ['⚡', '🛠️', '🔒'] },
    ],
  },
  haiku: {
    tierName: '🐇 Haiku Tier',
    role: 'Ultra-fast subagents, background file indexing, git commits & searches (Haiku tier)',
    badges: ['⚡', '🛠️'],
    equivalents: [
      { provider: 'Anthropic Direct', model: 'claude-3-5-haiku-latest', caps: ['⚡', '🛠️'] },
      { provider: 'OpenRouter', model: 'anthropic/claude-3.5-haiku or google/gemini-2.5-flash-lite', caps: ['⚡', '🛠️'] },
      { provider: 'Google / Gemini', model: 'google/gemini-2.5-flash-lite', caps: ['⚡', '🛠️'] },
      { provider: 'Ollama Cloud', model: 'gemma4:cloud', caps: ['⚡', '🌐'] },
      { provider: 'Ollama Local', model: 'qwen2.5-coder:7b / qwen2.5-coder:1.5b', caps: ['⚡', '🔒'] },
      { provider: 'Jan Local', model: 'janhq/Jan-code-4b', caps: ['⚡', '🔒'] },
    ],
  },
};

export const PROVIDERS: Record<ProviderId, ProviderDefinition> = {
  openrouter: {
    id: 'openrouter',
    name: 'OpenRouter',
    badge: '🌐 OpenRouter (Cloud Gateway)',
    description: 'Access 200+ frontier models (Claude Fable 5, Claude 3.7, DeepSeek R1, Gemini 2.5)',
    defaultBaseUrl: 'https://openrouter.ai/api',
    requiresKey: true,
    keyEnvVar: 'OPENROUTER_API_KEY',
    defaultModels: {
      epic: 'anthropic/claude-fable-5',
      large: 'anthropic/claude-3.7-sonnet',
      medium: 'anthropic/claude-3.5-sonnet',
      haiku: 'anthropic/claude-3.5-haiku',
    },
    suggestedModels: {
      epic: [
        { label: 'Claude Fable 5 (Anthropic Frontier Agentic)', value: 'anthropic/claude-fable-5', hint: '👑 Autonomous multi-step coding', equivalentTo: 'Claude Fable 5', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
        { label: 'DeepSeek R1 (Full 671B Reasoning)', value: 'deepseek/deepseek-r1', hint: '👑 Deep chain-of-thought equivalent to o3/Fable', equivalentTo: 'o3 / Fable 5', capabilities: ['thinking', 'tools', 'cloud'] },
        { label: 'Google Gemini 2.5 Pro (1M Context)', value: 'google/gemini-2.5-pro', hint: '👑 Frontier reasoning & massive context', equivalentTo: 'Claude Opus / Fable', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
      ],
      large: [
        { label: 'Claude 3.7 Sonnet (Hybrid Reasoning)', value: 'anthropic/claude-3.7-sonnet', hint: '🦁 Flagship code & architecture reasoning', equivalentTo: 'Claude 3.7 Sonnet', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
        { label: 'Claude 3 Opus (Deep Reasoning)', value: 'anthropic/claude-3-opus', hint: '🦁 Anthropic Opus tier intelligence', equivalentTo: 'Claude 3 Opus', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
        { label: 'Qwen 2.5 Coder 32B Instruct', value: 'qwen/qwen-2.5-coder-32b-instruct', hint: '🦁 Top open-weights code model (~Sonnet 3.5)', equivalentTo: 'Sonnet 3.5', capabilities: ['thinking', 'tools', 'cloud'] },
        { label: 'Meta Llama 3.3 70B Instruct', value: 'meta-llama/llama-3.3-70b-instruct', hint: '🦁 High reasoning open weights', equivalentTo: 'GPT-4o', capabilities: ['thinking', 'tools', 'cloud'] },
      ],
      medium: [
        { label: 'Claude 3.5 Sonnet', value: 'anthropic/claude-3.5-sonnet', hint: '⚡ Reliable coding leader', equivalentTo: 'Sonnet 3.5', capabilities: ['vision', 'tools', 'cloud'] },
        { label: 'DeepSeek V3 (Chat)', value: 'deepseek/deepseek-chat', hint: '⚡ Fast, exceptional value (~Sonnet 3.5)', equivalentTo: 'Sonnet 3.5', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Google Gemini 2.5 Flash', value: 'google/gemini-2.5-flash', hint: '⚡ Fast, responsive daily driver', equivalentTo: 'Sonnet / GPT-4o-mini', capabilities: ['fast', 'vision', 'tools', 'cloud'] },
        { label: 'Qwen 2.5 Coder 14B Instruct', value: 'qwen/qwen-2.5-coder-14b-instruct', hint: '⚡ Balanced speed and syntax quality', equivalentTo: 'GPT-4o-mini', capabilities: ['fast', 'tools', 'cloud'] },
      ],
      haiku: [
        { label: 'Claude 3.5 Haiku', value: 'anthropic/claude-3.5-haiku', hint: '🐇 Super fast background tasks & tool use', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Google Gemini 2.5 Flash Lite', value: 'google/gemini-2.5-flash-lite', hint: '🐇 Ultra low latency & high throughput', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Meta Llama 3.1 8B Instruct', value: 'meta-llama/llama-3.1-8b-instruct', hint: '🐇 Lightweight & instantaneous', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Qwen 2.5 Coder 7B Instruct', value: 'qwen/qwen-2.5-coder-7b-instruct', hint: '🐇 Quick code helper', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'cloud'] },
      ],
    },
  },

  ollama: {
    id: 'ollama',
    name: 'Ollama (Local & Cloud)',
    badge: '🦙 Ollama (Local & Cloud Models)',
    description: 'Run local Mac/GPU models or offload large models to Ollama Cloud',
    defaultBaseUrl: 'http://localhost:11434',
    requiresKey: false,
    defaultModels: {
      epic: 'deepseek-v4-pro:cloud',
      large: 'deepseek-v4-flash:cloud',
      medium: 'qwen2.5-coder:14b',
      haiku: 'qwen2.5-coder:7b',
    },
    suggestedModels: {
      epic: [
        { label: 'DeepSeek V4 Pro (Ollama Cloud)', value: 'deepseek-v4-pro:cloud', hint: '👑 Frontier MoE with deep reasoning', equivalentTo: 'Claude Fable 5', capabilities: ['thinking', 'tools', 'cloud'] },
        { label: 'DeepSeek R1 (Ollama Cloud / 70B)', value: 'deepseek-r1:cloud', hint: '👑 Deep chain-of-thought reasoning', equivalentTo: 'Claude Fable 5 / o3', capabilities: ['thinking', 'tools', 'cloud'] },
        { label: 'Llama 3.3 70B (Ollama Cloud / Local)', value: 'llama3.3:70b', hint: '👑 Full 70B reasoning powerhouse', equivalentTo: 'GPT-4o / Opus', capabilities: ['thinking', 'tools', 'local'] },
      ],
      large: [
        { label: 'DeepSeek V4 Flash (Ollama Cloud)', value: 'deepseek-v4-flash:cloud', hint: '🦁 High-speed MoE coding & reasoning', equivalentTo: 'Claude 3.7 Sonnet', capabilities: ['thinking', 'fast', 'tools', 'cloud'] },
        { label: 'Qwen 2.5 Coder 32B (Ollama Cloud / Local)', value: 'qwen2.5-coder:32b', hint: '🦁 Best open coding model (~Sonnet 3.5/3.7)', equivalentTo: 'Claude Sonnet', capabilities: ['thinking', 'tools', 'local'] },
        { label: 'DeepSeek R1 32B (Ollama Local)', value: 'deepseek-r1:32b', hint: '🦁 Local reasoning powerhouse', equivalentTo: 'Claude Sonnet', capabilities: ['thinking', 'tools', 'local'] },
        { label: 'Codestral 22B', value: 'codestral:22b', hint: '🦁 Mistral code specialist', equivalentTo: 'Sonnet 3.5', capabilities: ['tools', 'local'] },
      ],
      medium: [
        { label: 'DeepSeek V4 Flash (Ollama Cloud)', value: 'deepseek-v4-flash:cloud', hint: '⚡ Ultra-fast daily coding driver', equivalentTo: 'Sonnet 3.5', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Qwen 2.5 Coder 14B', value: 'qwen2.5-coder:14b', hint: '⚡ Balanced local speed & code quality', equivalentTo: 'GPT-4o-mini', capabilities: ['fast', 'tools', 'local'] },
        { label: 'DeepSeek R1 14B / 8B', value: 'deepseek-r1:14b', hint: '⚡ Compact reasoning model', equivalentTo: 'GPT-4o-mini', capabilities: ['thinking', 'fast', 'tools', 'local'] },
        { label: 'Llama 3.1 8B', value: 'llama3.1:8b', hint: '⚡ Fast general workhorse', equivalentTo: 'GPT-4o-mini', capabilities: ['fast', 'tools', 'local'] },
      ],
      haiku: [
        { label: 'Qwen 2.5 Coder 7B', value: 'qwen2.5-coder:7b', hint: '🐇 Lightweight fast helper for file searches & git', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'local'] },
        { label: 'Qwen 2.5 Coder 1.5B', value: 'qwen2.5-coder:1.5b', hint: '🐇 Instant micro-model for background tasks', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'local'] },
        { label: 'Llama 3.2 3B', value: 'llama3.2:3b', hint: '🐇 Ultra-fast compact worker', equivalentTo: 'Haiku', capabilities: ['fast', 'local'] },
      ],
    },
  },

  gemini: {
    id: 'gemini',
    name: 'Google Gemini',
    badge: '♊ Google Gemini (Direct / Proxy)',
    description: 'Gemini 2.5 Pro, Flash, and Flash-Lite models with huge context windows',
    defaultBaseUrl: 'https://openrouter.ai/api',
    requiresKey: true,
    keyEnvVar: 'GEMINI_API_KEY',
    defaultModels: {
      epic: 'google/gemini-2.5-pro',
      large: 'google/gemini-2.5-pro',
      medium: 'google/gemini-2.5-flash',
      haiku: 'google/gemini-2.5-flash-lite',
    },
    suggestedModels: {
      epic: [
        { label: 'Gemini 2.5 Pro (1M Context)', value: 'google/gemini-2.5-pro', hint: '👑 Frontier reasoning equivalent to Fable/Opus', equivalentTo: 'Claude Fable 5 / Opus', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
      ],
      large: [
        { label: 'Gemini 2.5 Pro', value: 'google/gemini-2.5-pro', hint: '🦁 High reasoning & deep code context', equivalentTo: 'Claude 3.7 Sonnet', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
        { label: 'Gemini 2.0 Flash Thinking', value: 'google/gemini-2.0-flash-thinking-exp', hint: '🦁 Fast reasoning flash model', equivalentTo: 'Sonnet 3.7', capabilities: ['thinking', 'fast', 'vision', 'tools', 'cloud'] },
      ],
      medium: [
        { label: 'Gemini 2.5 Flash', value: 'google/gemini-2.5-flash', hint: '⚡ High speed daily coding', equivalentTo: 'Claude 3.5 Sonnet', capabilities: ['fast', 'vision', 'tools', 'cloud'] },
        { label: 'Gemini 2.0 Flash', value: 'google/gemini-2.0-flash', hint: '⚡ Reliable fast driver', equivalentTo: 'Sonnet', capabilities: ['fast', 'vision', 'tools', 'cloud'] },
      ],
      haiku: [
        { label: 'Gemini 2.5 Flash Lite', value: 'google/gemini-2.5-flash-lite', hint: '🐇 Super fast subagents & background tasks', equivalentTo: 'Claude 3.5 Haiku', capabilities: ['fast', 'tools', 'cloud'] },
        { label: 'Gemini 2.0 Flash Lite', value: 'google/gemini-2.0-flash-lite', hint: '🐇 Tiny, ultra-fast helper', equivalentTo: 'Claude Haiku', capabilities: ['fast', 'tools', 'cloud'] },
      ],
    },
  },

  jan: {
    id: 'jan',
    name: 'Jan Desktop (Local Server)',
    badge: '🤖 Jan AI (Local Server)',
    description: 'Connect directly to your running Jan desktop local server',
    defaultBaseUrl: 'http://127.0.0.1:1337/v1',
    requiresKey: false,
    defaultModels: {
      epic: 'deepseek-r1-distill-qwen-14b',
      large: 'qwen2.5-coder-14b-instruct',
      medium: 'janhq/Jan-code-4b',
      haiku: 'janhq/Jan-code-4b',
    },
    suggestedModels: {
      epic: [
        { label: 'DeepSeek R1 Distill 14B (Jan)', value: 'deepseek-r1-distill-qwen-14b', hint: '👑 Reasoning loaded in Jan', equivalentTo: 'o3-mini / Fable', capabilities: ['thinking', 'tools', 'local'] },
        { label: 'Qwen 2.5 Coder 14B (Jan)', value: 'qwen2.5-coder-14b-instruct', hint: '👑 High-fidelity Jan model', equivalentTo: 'Sonnet', capabilities: ['tools', 'local'] },
      ],
      large: [
        { label: 'Qwen 2.5 Coder 14B (Jan)', value: 'qwen2.5-coder-14b-instruct', hint: '🦁 Flagship Jan coding model', equivalentTo: 'Sonnet', capabilities: ['tools', 'local'] },
        { label: 'Jan-code-4b', value: 'janhq/Jan-code-4b', hint: '🦁 Jan specialized coder', equivalentTo: 'Sonnet 3.5', capabilities: ['tools', 'local'] },
      ],
      medium: [
        { label: 'Jan-code-4b', value: 'janhq/Jan-code-4b', hint: '⚡ Jan fine-tuned coding assistant', equivalentTo: 'GPT-4o-mini', capabilities: ['fast', 'tools', 'local'] },
        { label: 'Qwen 2.5 Coder 7B (Jan)', value: 'qwen2.5-coder-7b-instruct', hint: '⚡ Fast code assistant', equivalentTo: 'GPT-4o-mini', capabilities: ['fast', 'tools', 'local'] },
      ],
      haiku: [
        { label: 'Jan-code-4b', value: 'janhq/Jan-code-4b', hint: '🐇 Fast background helper', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'local'] },
        { label: 'Qwen 2.5 Coder 1.5B (Jan)', value: 'qwen2.5-coder-1.5b-instruct', hint: '🐇 Instant micro worker', equivalentTo: 'Haiku', capabilities: ['fast', 'tools', 'local'] },
      ],
    },
  },

  anthropic: {
    id: 'anthropic',
    name: 'Anthropic (Default / Direct)',
    badge: '✨ Anthropic Official (Default)',
    description: 'Official Anthropic Claude models (Fable 5, Claude 3.7 Sonnet, Opus, Haiku)',
    requiresKey: false,
    defaultModels: {
      epic: 'claude-fable-5',
      large: 'claude-3-7-sonnet-latest',
      medium: 'claude-3-5-sonnet-latest',
      haiku: 'claude-3-5-haiku-latest',
    },
    suggestedModels: {
      epic: [
        { label: 'Claude Fable 5 (Frontier Autonomous Agentic)', value: 'claude-fable-5', hint: '👑 Next-gen Mythos-class proactive reasoning', equivalentTo: 'Claude Fable 5', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
      ],
      large: [
        { label: 'Claude 3.7 Sonnet', value: 'claude-3-7-sonnet-latest', hint: '🦁 Flagship hybrid reasoning coding model', equivalentTo: 'Claude 3.7 Sonnet', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
        { label: 'Claude 3 Opus', value: 'claude-3-opus-latest', hint: '🦁 High complexity reasoning', equivalentTo: 'Claude 3 Opus', capabilities: ['thinking', 'vision', 'tools', 'cloud'] },
      ],
      medium: [
        { label: 'Claude 3.5 Sonnet', value: 'claude-3-5-sonnet-latest', hint: '⚡ Standard daily coding driver', equivalentTo: 'Claude 3.5 Sonnet', capabilities: ['vision', 'tools', 'cloud'] },
      ],
      haiku: [
        { label: 'Claude 3.5 Haiku', value: 'claude-3-5-haiku-latest', hint: '🐇 Subagent & fast background worker', equivalentTo: 'Claude 3.5 Haiku', capabilities: ['fast', 'tools', 'cloud'] },
      ],
    },
  },

  custom: {
    id: 'custom',
    name: 'Custom Endpoint / Gateway',
    badge: '⚙️ Custom Proxy / Gateway',
    description: 'Point Claude Code to any custom Anthropic-compatible API URL and token',
    requiresKey: false,
    defaultModels: {
      epic: 'custom-epic-model',
      large: 'custom-large-model',
      medium: 'custom-medium-model',
      haiku: 'custom-haiku-model',
    },
    suggestedModels: {
      epic: [{ label: 'Custom Epic Model ID', value: 'custom-epic-model' }],
      large: [{ label: 'Custom Large Model ID', value: 'custom-large-model' }],
      medium: [{ label: 'Custom Medium Model ID', value: 'custom-medium-model' }],
      haiku: [{ label: 'Custom Haiku Model ID', value: 'custom-haiku-model' }],
    },
  },
};

/**
 * Dynamically queries local Ollama instance for installed models
 */
export async function fetchLocalOllamaModels(baseUrl = 'http://localhost:11434'): Promise<string[]> {
  try {
    const url = baseUrl.replace(/\/+$/, '') + '/api/tags';
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 1200);
    const res = await fetch(url, { signal: controller.signal });
    clearTimeout(timeout);

    if (!res.ok) return [];
    const data = (await res.json()) as { models?: Array<{ name: string; model?: string }> };
    if (!data.models || !Array.isArray(data.models)) return [];
    return data.models.map((m) => m.name || m.model || '').filter(Boolean);
  } catch {
    return [];
  }
}
