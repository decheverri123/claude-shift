#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { Command } from 'commander';
import chalk from 'chalk';
import { runInteractiveWizard, handleDownloadOllamaFlow } from './wizard.js';
import { printStatusCard, printResetSuccess, printSuccessShift, printPresetsList, printBanner, printEquivalenceGuide } from './ui.js';
import { resetClaudeConfig, applyShiftConfig } from './config.js';
import { PRESETS, findPreset } from './presets.js';
import { PROVIDERS } from './providers.js';
import type { ShiftConfig, ProviderId } from './types.js';

const program = new Command();

program
  .name('claude-shift')
  .description('A fast, intuitive CLI to shift models and providers for Claude Code across 4 tiers')
  .version('1.0.0')
  .option('-r, --reset', 'Reset Claude Code configuration back to official Anthropic defaults')
  .option('-s, --status', 'Display current active Claude Code model and provider configuration')
  .option('-p, --preset <name>', 'Quickly apply a preset (e.g. fable, sonnet37, deepseek-r1, ollama-qwen, gemini-25, jan)')
  .option('-l, --list-presets', 'List all available presets with their 4 model tiers')
  .option('-g, --guide', 'View cross-provider model equivalence guide')
  .option('--provider <provider>', 'Target provider: openrouter, ollama, gemini, jan, custom, anthropic')
  .option('--epic <model>', 'Epic Model ID (Frontier / Autonomous Agents / Mythos tier)')
  .option('--large <model>', 'Large Model ID (Opus / Flagship coding tier)')
  .option('--medium <model>', 'Medium Model ID (Sonnet / Daily coding tier)')
  .option('--haiku <model>', 'Haiku Model ID (Haiku / Background subagents tier)')
  .option('--small <model>', 'Alias for --haiku')
  .option('--base-url <url>', 'Custom API Base URL')
  .option('--key <key>', 'API key / Auth token')
  .action(async (options) => {
    // 1. Status flag
    if (options.status) {
      printBanner();
      printStatusCard();
      return;
    }

    // 2. Reset flag
    if (options.reset) {
      const { backupPath } = resetClaudeConfig();
      printBanner();
      printResetSuccess(backupPath);
      return;
    }

    // 3. List presets
    if (options.listPresets) {
      printBanner();
      printPresetsList(PRESETS);
      return;
    }

    // Direct pull flag
    if (options.pull) {
      console.log(chalk.cyan(`\n🚀 Starting ` + chalk.bold(`ollama pull ${options.pull}`) + `...\n`));
      const res = spawnSync("ollama", ["pull", options.pull], { stdio: "inherit" });
      if (res.status === 0) {
        console.log(chalk.bold.green(`\n✔ Successfully downloaded ${options.pull}!\n`));
      } else {
        console.log(chalk.red(`\n✖ Failed to download ${options.pull}. Is Ollama running?\n`));
      }
      return;
    }

    // 4. Model Equivalence Guide
    if (options.guide) {
      printBanner();
      printEquivalenceGuide();
      return;
    }

    // 5. Direct preset switch
    if (options.preset) {
      const preset = findPreset(options.preset);
      if (!preset) {
        console.error(chalk.red(`\nUnknown preset: "${options.preset}". Run with --list-presets to see options.\n`));
        process.exit(1);
      }

      if (preset.provider === 'anthropic') {
        const { backupPath } = resetClaudeConfig();
        printBanner();
        printResetSuccess(backupPath);
        return;
      }

      const key = options.key || process.env.OPENROUTER_API_KEY || process.env.ANTHROPIC_AUTH_TOKEN || (preset.provider === 'ollama' ? 'ollama' : preset.provider === 'jan' ? 'jan' : undefined);

      const config: ShiftConfig = {
        provider: preset.provider,
        providerName: preset.name,
        baseUrl: options.baseUrl || preset.baseUrl,
        authToken: key,
        models: {
          epic: options.epic || preset.models.epic,
          large: options.large || preset.models.large,
          medium: options.medium || preset.models.medium,
          haiku: options.haiku || options.small || preset.models.haiku,
        },
        presetName: preset.name,
        updatedAt: new Date().toISOString(),
      };

      const { backupPath } = applyShiftConfig(config);
      printBanner();
      printSuccessShift(config, backupPath);
      return;
    }

    // 6. Non-interactive direct provider flag
    if (options.provider) {
      const providerKey = options.provider.toLowerCase() as ProviderId;
      const providerDef = PROVIDERS[providerKey];

      if (!providerDef) {
        console.error(chalk.red(`\nUnknown provider: "${options.provider}". Choose from: openrouter, ollama, gemini, jan, custom, anthropic.\n`));
        process.exit(1);
      }

      if (providerKey === 'anthropic') {
        const { backupPath } = resetClaudeConfig();
        printBanner();
        printResetSuccess(backupPath);
        return;
      }

      const key = options.key || (providerKey === 'ollama' ? 'ollama' : providerKey === 'jan' ? 'jan' : process.env.OPENROUTER_API_KEY || process.env.ANTHROPIC_AUTH_TOKEN);

      const config: ShiftConfig = {
        provider: providerKey,
        providerName: providerDef.name,
        baseUrl: options.baseUrl || providerDef.defaultBaseUrl,
        authToken: key,
        models: {
          epic: options.epic || providerDef.defaultModels.epic,
          large: options.large || providerDef.defaultModels.large,
          medium: options.medium || providerDef.defaultModels.medium,
          haiku: options.haiku || options.small || providerDef.defaultModels.haiku,
        },
        updatedAt: new Date().toISOString(),
      };

      const { backupPath } = applyShiftConfig(config);
      printBanner();
      printSuccessShift(config, backupPath);
      return;
    }

    // 7. Default: Launch Interactive Wizard
    try {
      await runInteractiveWizard();
    } catch (err) {
      if ((err as Error)?.name === 'AbortError') {
        console.log(chalk.gray('\nCancelled.'));
        process.exit(0);
      }
      console.error(chalk.red('\nAn unexpected error occurred:'), err);
      process.exit(1);
    }
  });

program.parse(process.argv);
