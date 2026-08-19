export type ProviderId =
  | 'anthropic'
  | 'openrouter'
  | 'ollama'
  | 'gemini'
  | 'jan'
  | 'custom';

export type ModelCapability =
  | 'thinking' // 🧠 Reasoning / Chain-of-Thought
  | 'vision'   // 👁️ Multimodal / Vision
  | 'tools'    // 🛠️ Function Calling & Agent Tool Use
  | 'fast'     // ⚡ Low-latency / High throughput
  | 'cloud'    // 🌐 Cloud-hosted
  | 'local';   // 🔒 100% Local / Private

export interface ModelTierConfig {
  epic: string;    // Frontier / Mythos / Autonomous agents (Claude Fable 5, DeepSeek R1 671B, Gemini 2.5 Pro)
  large: string;   // Opus tier / Heavy reasoning & flagship coding (Claude 3.7 Sonnet, Qwen 32B)
  medium: string;  // Sonnet tier / Daily coding & refactoring (Claude 3.5 Sonnet, DeepSeek V3, Gemini Flash)
  haiku: string;   // Haiku tier / Background workers, fast searches & sub-agents (Claude 3.5 Haiku, Flash Lite)
}

export interface ShiftConfig {
  provider: ProviderId;
  providerName: string;
  baseUrl?: string;
  authToken?: string;
  apiKey?: string;
  models: ModelTierConfig;
  presetName?: string;
  updatedAt: string;
}

export interface Preset {
  id: string;
  name: string;
  description: string;
  provider: ProviderId;
  baseUrl?: string;
  authTokenPlaceholder?: string;
  models: ModelTierConfig;
  tags: string[];
}

export interface ClaudeSettings {
  env?: Record<string, string>;
  modelOverrides?: Record<string, string>;
  model?: string;
  [key: string]: any;
}
