import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import type { ClaudeSettings, ShiftConfig, ProviderId, ModelTierConfig } from './types.js';
import { PROVIDERS } from './providers.js';

const HOME_DIR = os.homedir();
const CLAUDE_DIR = path.join(HOME_DIR, '.claude');
const SETTINGS_FILE = path.join(CLAUDE_DIR, 'settings.json');
const STATE_FILE = path.join(CLAUDE_DIR, 'claude-shift-state.json');

/**
 * Ensures ~/.claude directory exists
 */
export function ensureClaudeDir(): void {
  if (!fs.existsSync(CLAUDE_DIR)) {
    fs.mkdirSync(CLAUDE_DIR, { recursive: true });
  }
}

/**
 * Reads ~/.claude/settings.json safely
 */
export function readClaudeSettings(): ClaudeSettings {
  ensureClaudeDir();
  if (!fs.existsSync(SETTINGS_FILE)) {
    return {};
  }
  try {
    const raw = fs.readFileSync(SETTINGS_FILE, 'utf-8');
    return JSON.parse(raw) as ClaudeSettings;
  } catch {
    return {};
  }
}

/**
 * Creates a backup of ~/.claude/settings.json before modifying
 */
export function backupClaudeSettings(): string | null {
  if (!fs.existsSync(SETTINGS_FILE)) {
    return null;
  }
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const backupFile = path.join(CLAUDE_DIR, `settings.json.cshift-backup-${timestamp}`);
  try {
    fs.copyFileSync(SETTINGS_FILE, backupFile);
    return backupFile;
  } catch {
    return null;
  }
}

/**
 * Writes updated settings to ~/.claude/settings.json atomically
 */
export function writeClaudeSettings(settings: ClaudeSettings): void {
  ensureClaudeDir();
  const tempFile = `${SETTINGS_FILE}.tmp.${Date.now()}`;
  fs.writeFileSync(tempFile, JSON.stringify(settings, null, 2), 'utf-8');
  fs.renameSync(tempFile, SETTINGS_FILE);
}

/**
 * Saves active shift state
 */
export function saveShiftState(config: ShiftConfig): void {
  ensureClaudeDir();
  try {
    fs.writeFileSync(STATE_FILE, JSON.stringify(config, null, 2), 'utf-8');
  } catch {
    // Non-critical
  }
}

/**
 * Reads saved shift state
 */
export function readShiftState(): ShiftConfig | null {
  if (!fs.existsSync(STATE_FILE)) {
    return null;
  }
  try {
    const raw = fs.readFileSync(STATE_FILE, 'utf-8');
    return JSON.parse(raw) as ShiftConfig;
  } catch {
    return null;
  }
}

/**
 * Clears saved shift state
 */
export function clearShiftState(): void {
  if (fs.existsSync(STATE_FILE)) {
    try {
      fs.unlinkSync(STATE_FILE);
    } catch {
      // Ignore
    }
  }
}

/**
 * Reads the current active configuration from both settings.json and state
 */
export function getCurrentConfiguration(): {
  isDefault: boolean;
  provider: ProviderId;
  providerName: string;
  baseUrl?: string;
  models: ModelTierConfig;
  presetName?: string;
  updatedAt?: string;
} {
  const settings = readClaudeSettings();
  const savedState = readShiftState();

  const env = settings.env || {};
  const baseUrl = env.ANTHROPIC_BASE_URL;
  const epicModel = env.ANTHROPIC_DEFAULT_EPIC_MODEL;
  const opusModel = env.ANTHROPIC_DEFAULT_OPUS_MODEL;
  const sonnetModel = env.ANTHROPIC_DEFAULT_SONNET_MODEL;
  const haikuModel = env.ANTHROPIC_DEFAULT_HAIKU_MODEL;

  const isDefault = !baseUrl && !epicModel && !opusModel && !sonnetModel && !haikuModel && (!settings.modelOverrides || Object.keys(settings.modelOverrides).length === 0);

  if (isDefault) {
    return {
      isDefault: true,
      provider: 'anthropic',
      providerName: 'Anthropic Official (Default)',
      models: {
        epic: 'claude-fable-5',
        large: 'claude-3-7-sonnet / claude-3-opus',
        medium: 'claude-3-5-sonnet',
        haiku: 'claude-3-5-haiku',
      },
    };
  }

  // Derive active provider
  let provider: ProviderId = savedState?.provider || 'custom';
  if (baseUrl) {
    if (baseUrl.includes('openrouter.ai')) provider = 'openrouter';
    else if (baseUrl.includes('11434') || baseUrl.includes('ollama')) provider = 'ollama';
    else if (baseUrl.includes('1337') || baseUrl.includes('6767') || baseUrl.includes('jan')) provider = 'jan';
    else if (baseUrl.includes('gemini') || baseUrl.includes('google')) provider = 'gemini';
  }

  const providerDef = PROVIDERS[provider] || PROVIDERS.custom;

  return {
    isDefault: false,
    provider,
    providerName: savedState?.providerName || providerDef.name,
    baseUrl,
    presetName: savedState?.presetName,
    updatedAt: savedState?.updatedAt,
    models: {
      epic: epicModel || savedState?.models.epic || 'claude-fable-5',
      large: opusModel || savedState?.models.large || 'claude-3-7-sonnet',
      medium: sonnetModel || savedState?.models.medium || 'claude-3-5-sonnet',
      haiku: haikuModel || savedState?.models.haiku || 'claude-3-5-haiku',
    },
  };
}

/**
 * Resets Claude Code configuration back to official Anthropic defaults
 */
export function resetClaudeConfig(): { backupPath: string | null } {
  const backupPath = backupClaudeSettings();
  const settings = readClaudeSettings();

  if (settings.env) {
    delete settings.env.ANTHROPIC_BASE_URL;
    delete settings.env.ANTHROPIC_AUTH_TOKEN;
    delete settings.env.ANTHROPIC_API_KEY;
    delete settings.env.ANTHROPIC_DEFAULT_EPIC_MODEL;
    delete settings.env.ANTHROPIC_DEFAULT_OPUS_MODEL;
    delete settings.env.ANTHROPIC_DEFAULT_SONNET_MODEL;
    delete settings.env.ANTHROPIC_DEFAULT_HAIKU_MODEL;

    // Remove empty env object if empty
    if (Object.keys(settings.env).length === 0) {
      delete settings.env;
    }
  }

  if (settings.modelOverrides) {
    delete settings.modelOverrides;
  }

  delete settings.model;

  writeClaudeSettings(settings);
  clearShiftState();

  return { backupPath };
}

/**
 * Applies a new shift configuration to ~/.claude/settings.json
 */
export function applyShiftConfig(config: ShiftConfig): { backupPath: string | null } {
  if (config.provider === 'anthropic') {
    return resetClaudeConfig();
  }

  const backupPath = backupClaudeSettings();
  const settings = readClaudeSettings();

  if (!settings.env) {
    settings.env = {};
  }

  // Set Base URL
  if (config.baseUrl) {
    settings.env.ANTHROPIC_BASE_URL = config.baseUrl;
  } else {
    delete settings.env.ANTHROPIC_BASE_URL;
  }

  // Set Auth Token / API Key
  if (config.authToken) {
    settings.env.ANTHROPIC_AUTH_TOKEN = config.authToken;
    settings.env.ANTHROPIC_API_KEY = ''; // Clear default key to force custom gateway auth
  } else if (config.provider === 'ollama') {
    settings.env.ANTHROPIC_AUTH_TOKEN = 'ollama';
    settings.env.ANTHROPIC_API_KEY = '';
  } else if (config.provider === 'jan') {
    settings.env.ANTHROPIC_AUTH_TOKEN = 'jan';
    settings.env.ANTHROPIC_API_KEY = '';
  }

  // Set Model Tier Aliases
  settings.env.ANTHROPIC_DEFAULT_EPIC_MODEL = config.models.epic;
  settings.env.ANTHROPIC_DEFAULT_OPUS_MODEL = config.models.large;
  settings.env.ANTHROPIC_DEFAULT_SONNET_MODEL = config.models.medium;
  settings.env.ANTHROPIC_DEFAULT_HAIKU_MODEL = config.models.haiku;

  // Set Model Overrides for direct version strings
  settings.modelOverrides = {
    'claude-fable-5': config.models.epic,
    'claude-mythos-5': config.models.epic,
    'claude-3-opus-latest': config.models.large,
    'claude-opus-3-20240229': config.models.large,
    'claude-3-7-sonnet-latest': config.models.large,
    'claude-3-7-sonnet-20250219': config.models.large,
    'claude-3-5-sonnet-latest': config.models.medium,
    'claude-3-5-sonnet-20241022': config.models.medium,
    'claude-3-5-haiku-latest': config.models.haiku,
    'claude-3-5-haiku-20241022': config.models.haiku,
  };

  settings.model = config.models.large || config.models.medium;

  writeClaudeSettings(settings);
  saveShiftState(config);

  return { backupPath };
}
