import type { Preset } from './types.js';

export const PRESETS: Preset[] = [
  {
    id: 'openrouter-fable',
    name: 'OpenRouter · Claude Fable 5 & Frontier Suite',
    description: 'Claude Fable 5 (Epic), Claude 3.7 Sonnet (Large), Claude 3.5 Sonnet (Medium), Claude 3.5 Haiku (Haiku)',
    provider: 'openrouter',
    baseUrl: 'https://openrouter.ai/api',
    authTokenPlaceholder: 'sk-or-v1-...',
    models: {
      epic: 'anthropic/claude-fable-5',
      large: 'anthropic/claude-3.7-sonnet',
      medium: 'anthropic/claude-3.5-sonnet',
      haiku: 'anthropic/claude-3.5-haiku',
    },
    tags: ['openrouter', 'fable', 'frontier', 'recommended'],
  },
  {
    id: 'openrouter-sonnet37',
    name: 'OpenRouter · Claude 3.7 Sonnet Power Suite',
    description: 'Claude Fable 5 (Epic), Claude 3.7 Sonnet (Large), Claude 3.5 Sonnet (Medium), Claude 3.5 Haiku (Haiku)',
    provider: 'openrouter',
    baseUrl: 'https://openrouter.ai/api',
    authTokenPlaceholder: 'sk-or-v1-...',
    models: {
      epic: 'anthropic/claude-fable-5',
      large: 'anthropic/claude-3.7-sonnet',
      medium: 'anthropic/claude-3.5-sonnet',
      haiku: 'anthropic/claude-3.5-haiku',
    },
    tags: ['openrouter', 'sonnet3.7', 'coding'],
  },
  {
    id: 'deepseek-r1',
    name: 'OpenRouter · DeepSeek R1 & V3 Suite',
    description: 'DeepSeek R1 full 671B (Epic), DeepSeek R1 (Large), DeepSeek V3 (Medium), DeepSeek Chat (Haiku)',
    provider: 'openrouter',
    baseUrl: 'https://openrouter.ai/api',
    authTokenPlaceholder: 'sk-or-v1-...',
    models: {
      epic: 'deepseek/deepseek-r1',
      large: 'deepseek/deepseek-r1',
      medium: 'deepseek/deepseek-chat',
      haiku: 'deepseek/deepseek-chat',
    },
    tags: ['openrouter', 'deepseek', 'reasoning', 'low-cost'],
  },
  {
    id: 'gemini-25',
    name: 'Gemini · 2.5 Pro & Flash Suite',
    description: 'Gemini 2.5 Pro (Epic & Large), Gemini 2.5 Flash (Medium), Flash Lite (Haiku)',
    provider: 'gemini',
    baseUrl: 'https://openrouter.ai/api',
    authTokenPlaceholder: 'sk-or-v1-...',
    models: {
      epic: 'google/gemini-2.5-pro',
      large: 'google/gemini-2.5-pro',
      medium: 'google/gemini-2.5-flash',
      haiku: 'google/gemini-2.5-flash-lite',
    },
    tags: ['gemini', 'google', 'long-context', 'fast'],
  },
  {
    id: 'ollama-qwen',
    name: 'Ollama · Qwen 2.5 Coder Suite (Local)',
    description: 'DeepSeek R1 32B (Epic), Qwen 2.5 Coder 32B (Large), 14B (Medium), 7B (Haiku)',
    provider: 'ollama',
    baseUrl: 'http://localhost:11434',
    models: {
      epic: 'deepseek-r1:32b',
      large: 'qwen2.5-coder:32b',
      medium: 'qwen2.5-coder:14b',
      haiku: 'qwen2.5-coder:7b',
    },
    tags: ['ollama', 'local', 'offline', 'privacy'],
  },
  {
    id: 'ollama-deepseek',
    name: 'Ollama · DeepSeek R1 + Qwen (Local)',
    description: 'DeepSeek R1 32B (Epic), DeepSeek R1 14B (Large), Qwen 14B (Medium), Qwen 7B (Haiku)',
    provider: 'ollama',
    baseUrl: 'http://localhost:11434',
    models: {
      epic: 'deepseek-r1:32b',
      large: 'deepseek-r1:14b',
      medium: 'qwen2.5-coder:14b',
      haiku: 'qwen2.5-coder:7b',
    },
    tags: ['ollama', 'local', 'reasoning'],
  },
  {
    id: 'jan-code',
    name: 'Jan · Jan-Code-4B & Local (Jan Server)',
    description: 'DeepSeek R1 Distill (Epic), Qwen 2.5 Coder (Large), Jan-Code-4b (Medium & Haiku)',
    provider: 'jan',
    baseUrl: 'http://127.0.0.1:1337/v1',
    models: {
      epic: 'deepseek-r1-distill-qwen-14b',
      large: 'qwen2.5-coder-14b-instruct',
      medium: 'janhq/Jan-code-4b',
      haiku: 'janhq/Jan-code-4b',
    },
    tags: ['jan', 'local', 'desktop'],
  },
  {
    id: 'default',
    name: 'Anthropic · Official Claude Defaults (Reset)',
    description: 'Official Anthropic API with standard Claude Fable 5 / 3.7 Sonnet / Opus / 3.5 Sonnet / Haiku',
    provider: 'anthropic',
    models: {
      epic: 'claude-fable-5',
      large: 'claude-3-7-sonnet-latest',
      medium: 'claude-3-5-sonnet-latest',
      haiku: 'claude-3-5-haiku-latest',
    },
    tags: ['anthropic', 'default', 'official', 'reset'],
  },
];

export function findPreset(idOrTag: string): Preset | undefined {
  const query = idOrTag.toLowerCase().trim();
  return PRESETS.find(
    (p) =>
      p.id.toLowerCase() === query ||
      p.id.replace(/[-_]/g, '') === query.replace(/[-_]/g, '') ||
      p.tags.includes(query)
  );
}
