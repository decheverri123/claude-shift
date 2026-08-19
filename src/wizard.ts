import * as p from '@clack/prompts';
import chalk from 'chalk';
import { PROVIDERS, fetchLocalOllamaModels, EQUIVALENCE_GUIDE, formatCapabilities, inferCapabilities } from './providers.js';
import { PRESETS } from './presets.js';
import type { ProviderId, ModelTierConfig, ShiftConfig } from './types.js';
import { applyShiftConfig, resetClaudeConfig, getCurrentConfiguration } from './config.js';
import { printBanner, printStatusCard, printSuccessShift, printResetSuccess, printEquivalenceGuide } from './ui.js';

export async function runInteractiveWizard(): Promise<void> {
  printBanner();
  printStatusCard();

  const action = await p.select({
    message: 'What would you like to do?',
    options: [
      { value: 'preset', label: '⚡ Quick Presets', hint: 'Instant switch to Fable 5, Claude 3.7, DeepSeek R1, Ollama, Gemini...' },
      { value: 'provider', label: '🛠️  Configure 4 Model Tiers (Haiku, Medium, Large, Epic)', hint: 'Customize all 4 model tiers with capability badges' },
      { value: 'guide', label: '📚 View Model Equivalence & Capability Guide', hint: 'See model capabilities (🧠 Thinking, 👁️ Vision, 🛠️ Tools, ⚡ Fast)' },
      { value: 'status', label: '📊 View Current Active Configuration', hint: 'Inspect ~/.claude/settings.json status' },
      { value: 'reset', label: '🔄 Reset to Claude Code Defaults', hint: 'Remove overrides and restore official Anthropic models' },
      { value: 'exit', label: '🚪 Exit' },
    ],
  });

  if (p.isCancel(action) || action === 'exit') {
    p.outro(chalk.gray('Goodbye!'));
    return;
  }

  if (action === 'guide') {
    printEquivalenceGuide();
    p.outro(chalk.cyan('Run cshift again to switch models.'));
    return;
  }

  if (action === 'status') {
    p.outro(chalk.cyan('Showing current active configuration above.'));
    return;
  }

  if (action === 'reset') {
    const confirm = await p.confirm({
      message: 'Are you sure you want to reset Claude Code back to official Anthropic defaults?',
      initialValue: true,
    });
    if (p.isCancel(confirm) || !confirm) {
      p.cancel('Reset cancelled.');
      return;
    }
    const { backupPath } = resetClaudeConfig();
    printResetSuccess(backupPath);
    p.outro(chalk.green('Reset complete.'));
    return;
  }

  if (action === 'preset') {
    await handlePresetFlow();
    return;
  }

  if (action === 'provider') {
    await handleProviderFlow();
    return;
  }
}

async function handlePresetFlow(): Promise<void> {
  const selectedPresetId = await p.select({
    message: 'Select a quick configuration preset (4-tier):',
    options: PRESETS.map((preset) => ({
      value: preset.id,
      label: preset.name,
      hint: preset.description,
    })),
  });

  if (p.isCancel(selectedPresetId)) {
    p.cancel('Cancelled preset selection.');
    return;
  }

  const preset = PRESETS.find((pr) => pr.id === selectedPresetId);
  if (!preset) return;

  if (preset.provider === 'anthropic') {
    const { backupPath } = resetClaudeConfig();
    printResetSuccess(backupPath);
    p.outro(chalk.green('Restored Anthropic defaults.'));
    return;
  }

  let authToken: string | undefined;

  if (preset.provider === 'openrouter' || preset.provider === 'gemini') {
    const envKey = process.env.OPENROUTER_API_KEY || process.env.ANTHROPIC_AUTH_TOKEN;
    if (envKey) {
      const useEnv = await p.confirm({
        message: `Found existing API key in environment (${envKey.slice(0, 8)}...). Use this key?`,
        initialValue: true,
      });
      if (p.isCancel(useEnv)) return;
      if (useEnv) {
        authToken = envKey;
      }
    }

    if (!authToken) {
      const enteredKey = await p.password({
        message: `Enter your ${preset.provider === 'gemini' ? 'Gemini/OpenRouter' : 'OpenRouter'} API Key:`,
        mask: '•',
      });
      if (p.isCancel(enteredKey)) return;
      authToken = enteredKey as string;
    }
  }

  const config: ShiftConfig = {
    provider: preset.provider,
    providerName: preset.name,
    baseUrl: preset.baseUrl,
    authToken,
    models: preset.models,
    presetName: preset.name,
    updatedAt: new Date().toISOString(),
  };

  const { backupPath } = applyShiftConfig(config);
  printSuccessShift(config, backupPath);
  p.outro(chalk.green('Preset applied successfully!'));
}

async function handleProviderFlow(): Promise<void> {
  const providerId = (await p.select({
    message: 'Select an AI provider / backend:',
    options: Object.values(PROVIDERS).map((pr) => ({
      value: pr.id,
      label: pr.badge,
      hint: pr.description,
    })),
  })) as ProviderId | symbol;

  if (p.isCancel(providerId)) {
    p.cancel('Cancelled.');
    return;
  }

  if (providerId === 'anthropic') {
    const { backupPath } = resetClaudeConfig();
    printResetSuccess(backupPath);
    p.outro(chalk.green('Anthropic defaults applied.'));
    return;
  }

  const provider = PROVIDERS[providerId];
  let dynamicOllamaModels: string[] = [];

  if (providerId === 'ollama') {
    const s = p.spinner();
    s.start('Detecting running Ollama models...');
    dynamicOllamaModels = await fetchLocalOllamaModels(provider.defaultBaseUrl);
    if (dynamicOllamaModels.length > 0) {
      s.stop(chalk.green(`Found ${dynamicOllamaModels.length} local Ollama models!`));
    } else {
      s.stop(chalk.yellow('Ollama not currently detected or no local tags found (using standard presets).'));
    }
  }

  // 1. Epic Model Selection (Frontier / Autonomous Agents / Mythos)
  const epicOptions = [
    ...(dynamicOllamaModels.length > 0
      ? dynamicOllamaModels.map((m) => {
          const caps = formatCapabilities(inferCapabilities(m));
          return {
            label: `${m} ${caps ? `[${caps}]` : ''}`,
            value: m,
            hint: 'Installed Ollama Model',
          };
        })
      : provider.suggestedModels.epic.map((m) => {
          const caps = formatCapabilities(m.capabilities || inferCapabilities(m.value));
          return {
            label: `${m.label} ${caps ? `[${caps}]` : ''}`,
            value: m.value,
            hint: m.equivalentTo ? `≈ ${m.equivalentTo} · ${m.hint}` : m.hint,
          };
        })),
    { label: '✏️  Custom Model ID...', value: '__custom__', hint: 'Type custom model string' },
  ];

  const selectedEpic = await p.select({
    message: '👑 1/4 Select EPIC Model (Frontier / Autonomous Agents / Mythos Tier):',
    options: epicOptions,
  });
  if (p.isCancel(selectedEpic)) return;

  let epicModel = selectedEpic as string;
  if (epicModel === '__custom__') {
    const custom = await p.text({
      message: 'Enter custom Epic Model ID:',
      placeholder: provider.defaultModels.epic,
      defaultValue: provider.defaultModels.epic,
    });
    if (p.isCancel(custom)) return;
    epicModel = custom as string;
  }

  // 2. Large Model Selection (Opus Tier / Flagship Coding)
  const largeOptions = [
    ...(dynamicOllamaModels.length > 0
      ? dynamicOllamaModels.map((m) => {
          const caps = formatCapabilities(inferCapabilities(m));
          return {
            label: `${m} ${caps ? `[${caps}]` : ''}`,
            value: m,
            hint: 'Installed Ollama Model',
          };
        })
      : provider.suggestedModels.large.map((m) => {
          const caps = formatCapabilities(m.capabilities || inferCapabilities(m.value));
          return {
            label: `${m.label} ${caps ? `[${caps}]` : ''}`,
            value: m.value,
            hint: m.equivalentTo ? `≈ ${m.equivalentTo} · ${m.hint}` : m.hint,
          };
        })),
    { label: '✏️  Custom Model ID...', value: '__custom__', hint: 'Type custom model string' },
  ];

  const selectedLarge = await p.select({
    message: '🦁 2/4 Select LARGE Model (Opus tier / High reasoning & flagship coding):',
    options: largeOptions,
  });
  if (p.isCancel(selectedLarge)) return;

  let largeModel = selectedLarge as string;
  if (largeModel === '__custom__') {
    const custom = await p.text({
      message: 'Enter custom Large Model ID:',
      placeholder: provider.defaultModels.large,
      defaultValue: provider.defaultModels.large,
    });
    if (p.isCancel(custom)) return;
    largeModel = custom as string;
  }

  // 3. Medium Model Selection (Sonnet Tier / Daily Coding Driver)
  const mediumOptions = [
    ...(dynamicOllamaModels.length > 0
      ? dynamicOllamaModels.map((m) => {
          const caps = formatCapabilities(inferCapabilities(m));
          return {
            label: `${m} ${caps ? `[${caps}]` : ''}`,
            value: m,
            hint: 'Installed Ollama Model',
          };
        })
      : provider.suggestedModels.medium.map((m) => {
          const caps = formatCapabilities(m.capabilities || inferCapabilities(m.value));
          return {
            label: `${m.label} ${caps ? `[${caps}]` : ''}`,
            value: m.value,
            hint: m.equivalentTo ? `≈ ${m.equivalentTo} · ${m.hint}` : m.hint,
          };
        })),
    { label: '✏️  Custom Model ID...', value: '__custom__', hint: 'Type custom model string' },
  ];

  const selectedMedium = await p.select({
    message: '⚡ 3/4 Select MEDIUM Model (Sonnet tier / Daily coding driver):',
    options: mediumOptions,
  });
  if (p.isCancel(selectedMedium)) return;

  let mediumModel = selectedMedium as string;
  if (mediumModel === '__custom__') {
    const custom = await p.text({
      message: 'Enter custom Medium Model ID:',
      placeholder: provider.defaultModels.medium,
      defaultValue: provider.defaultModels.medium,
    });
    if (p.isCancel(custom)) return;
    mediumModel = custom as string;
  }

  // 4. Haiku Model Selection (Haiku Tier / Subagents & background tasks)
  const haikuOptions = [
    ...(dynamicOllamaModels.length > 0
      ? dynamicOllamaModels.map((m) => {
          const caps = formatCapabilities(inferCapabilities(m));
          return {
            label: `${m} ${caps ? `[${caps}]` : ''}`,
            value: m,
            hint: 'Installed Ollama Model',
          };
        })
      : provider.suggestedModels.haiku.map((m) => {
          const caps = formatCapabilities(m.capabilities || inferCapabilities(m.value));
          return {
            label: `${m.label} ${caps ? `[${caps}]` : ''}`,
            value: m.value,
            hint: m.equivalentTo ? `≈ ${m.equivalentTo} · ${m.hint}` : m.hint,
          };
        })),
    { label: '✏️  Custom Model ID...', value: '__custom__', hint: 'Type custom model string' },
  ];

  const selectedHaiku = await p.select({
    message: '🐇 4/4 Select HAIKU Model (Haiku tier / Background & sub-agents):',
    options: haikuOptions,
  });
  if (p.isCancel(selectedHaiku)) return;

  let haikuModel = selectedHaiku as string;
  if (haikuModel === '__custom__') {
    const custom = await p.text({
      message: 'Enter custom Haiku Model ID:',
      placeholder: provider.defaultModels.haiku,
      defaultValue: provider.defaultModels.haiku,
    });
    if (p.isCancel(custom)) return;
    haikuModel = custom as string;
  }

  // 5. Base URL Configuration
  const baseUrlInput = await p.text({
    message: 'Base URL (Endpoint):',
    defaultValue: provider.defaultBaseUrl || 'https://openrouter.ai/api',
    placeholder: provider.defaultBaseUrl || 'https://openrouter.ai/api',
  });
  if (p.isCancel(baseUrlInput)) return;

  // 6. API Key / Auth Token
  let authToken: string | undefined;
  if (provider.requiresKey) {
    const enteredKey = await p.password({
      message: `Enter ${provider.name} API Key / Token:`,
      mask: '•',
    });
    if (p.isCancel(enteredKey)) return;
    authToken = enteredKey as string;
  } else if (providerId === 'ollama') {
    authToken = 'ollama';
  } else if (providerId === 'jan') {
    authToken = 'jan';
  }

  const config: ShiftConfig = {
    provider: providerId,
    providerName: provider.name,
    baseUrl: baseUrlInput as string,
    authToken,
    models: {
      epic: epicModel,
      large: largeModel,
      medium: mediumModel,
      haiku: haikuModel,
    },
    updatedAt: new Date().toISOString(),
  };

  const { backupPath } = applyShiftConfig(config);
  printSuccessShift(config, backupPath);
  p.outro(chalk.green('4-tier configuration successfully activated!'));
}
