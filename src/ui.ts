import chalk from 'chalk';
import boxen from 'boxen';
import gradient from 'gradient-string';
import figures from 'figures';
import type { ShiftConfig, Preset } from './types.js';
import { getCurrentConfiguration } from './config.js';
import { EQUIVALENCE_GUIDE, inferCapabilities, formatCapabilities, CAPABILITY_ICONS } from './providers.js';

const purpleGradient = gradient(['#FF6B6B', '#9B51E0', '#3B82F6']);

export function printBanner(): void {
  const title = `
   ██████╗██╗      █████╗ ██╗   ██╗██████╗ ███████╗   ███████╗██╗  ██╗██╗███████╗████████╗
  ██╔════╝██║     ██╔══██╗██║   ██║██╔══██╗██╔════╝   ██╔════╝██║  ██║██║██╔════╝╚══██╔══╝
  ██║     ██║     ███████║██║   ██║██║  ██║█████╗     ███████╗███████║██║█████╗     ██║   
  ██║     ██║     ██╔══██║██║   ██║██║  ██║██╔══╝     ╚════██║██╔══██║██║██╔══╝     ██║   
  ╚██████╗███████╗██║  ██║╚██████╔╝██████╔╝███████╗   ███████║██║  ██║██║██║        ██║   
   ╚═════╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝   ╚══════╝╚═╝  ╚═╝╚═╝╚═╝        ╚═╝   
  `;

  console.log(purpleGradient.multiline(title.trimEnd()));
  console.log(chalk.gray(`  ⚡ Instant Claude Code Model & Provider Switcher · 4-Tier Control (Haiku, Medium, Large, Epic)`));
  console.log();
}

export function printStatusCard(): void {
  const current = getCurrentConfiguration();

  const title = current.isDefault
    ? chalk.bold.green(` ${figures.tick} Active: Anthropic Official (Default) `)
    : chalk.bold.cyan(` ${figures.star} Active: ${current.providerName} `);

  const lines: string[] = [];

  if (current.presetName) {
    lines.push(`${chalk.dim('Preset:')}        ${chalk.bold.magenta(current.presetName)}`);
  }

  lines.push(`${chalk.dim('Provider:')}      ${chalk.yellow(current.provider.toUpperCase())}`);

  if (current.baseUrl) {
    lines.push(`${chalk.dim('Base URL:')}      ${chalk.blueBright(current.baseUrl)}`);
  } else {
    lines.push(`${chalk.dim('Endpoint:')}      ${chalk.gray('api.anthropic.com (Standard)')}`);
  }

  const epicCaps = formatCapabilities(inferCapabilities(current.models.epic));
  const largeCaps = formatCapabilities(inferCapabilities(current.models.large));
  const mediumCaps = formatCapabilities(inferCapabilities(current.models.medium));
  const haikuCaps = formatCapabilities(inferCapabilities(current.models.haiku));

  lines.push('');
  lines.push(chalk.bold.underline('Active 4 Model Tiers:'));
  lines.push(`  ${chalk.bold.magentaBright('👑 Epic Model  ')} ${chalk.dim('(Frontier/Agents):')}   ${chalk.whiteBright.bold(current.models.epic)} ${chalk.yellow(epicCaps ? `[${epicCaps}]` : '')}`);
  lines.push(`  ${chalk.bold.redBright('🦁 Large Model ')} ${chalk.dim('(Opus/Hybrid):')}       ${chalk.whiteBright.bold(current.models.large)} ${chalk.yellow(largeCaps ? `[${largeCaps}]` : '')}`);
  lines.push(`  ${chalk.bold.cyanBright('⚡ Medium Model')} ${chalk.dim('(Sonnet/Coding):')}     ${chalk.whiteBright.bold(current.models.medium)} ${chalk.yellow(mediumCaps ? `[${mediumCaps}]` : '')}`);
  lines.push(`  ${chalk.bold.greenBright('🐇 Haiku Model ')} ${chalk.dim('(Haiku/Worker):')}      ${chalk.whiteBright.bold(current.models.haiku)} ${chalk.yellow(haikuCaps ? `[${haikuCaps}]` : '')}`);

  lines.push('');
  lines.push(chalk.dim(`Badges: 🧠 Thinking · 👁️ Vision · 🛠️ Tools · ⚡ Fast · 🌐 Cloud · 🔒 Local`));

  if (current.updatedAt) {
    lines.push(chalk.dim(`Last shifted: ${new Date(current.updatedAt).toLocaleString()}`));
  }

  const card = boxen(lines.join('\n'), {
    title,
    titleAlignment: 'left',
    padding: 1,
    margin: { top: 0, bottom: 1, left: 1, right: 1 },
    borderColor: current.isDefault ? 'green' : 'cyan',
    borderStyle: 'round',
  });

  console.log(card);
}

export function printEquivalenceGuide(): void {
  console.log(chalk.bold.cyanBright('\n  📚 Cross-Provider Model Equivalence & Capability Guide:\n'));

  for (const [tierKey, guide] of Object.entries(EQUIVALENCE_GUIDE)) {
    const badgesStr = guide.badges ? chalk.yellow(`[${guide.badges.join(' ')}] `) : '';
    console.log(`  ${chalk.bold.yellow(guide.tierName)} ${badgesStr}: ${chalk.gray(guide.role)}`);
    for (const eq of guide.equivalents) {
      const capsStr = eq.caps ? chalk.dim(` [${eq.caps.join(' ')}]`) : '';
      console.log(`    ${chalk.dim('•')} ${chalk.white.bold(eq.provider.padEnd(18))}: ${chalk.cyan(eq.model)}${capsStr}`);
    }
    console.log();
  }

  console.log(chalk.bold.underline('  Capability Badges Legend:'));
  const legend = Object.entries(CAPABILITY_ICONS)
    .map(([_, v]) => `${v.icon} ${v.label}`)
    .join('   ');
  console.log(`  ${chalk.gray(legend)}\n`);
}

export function printSuccessShift(config: ShiftConfig, backupPath: string | null): void {
  const epicCaps = formatCapabilities(inferCapabilities(config.models.epic));
  const largeCaps = formatCapabilities(inferCapabilities(config.models.large));
  const mediumCaps = formatCapabilities(inferCapabilities(config.models.medium));
  const haikuCaps = formatCapabilities(inferCapabilities(config.models.haiku));

  const lines: string[] = [
    chalk.bold.green(`${figures.tick} Claude Code successfully shifted to ${chalk.cyan(config.providerName)}!`),
    '',
    `  ${chalk.magentaBright('👑 Epic Tier:')}    ${chalk.whiteBright.bold(config.models.epic)} ${chalk.yellow(epicCaps ? `[${epicCaps}]` : '')}`,
    `  ${chalk.redBright('🦁 Large Tier:')}   ${chalk.whiteBright.bold(config.models.large)} ${chalk.yellow(largeCaps ? `[${largeCaps}]` : '')}`,
    `  ${chalk.cyanBright('⚡ Medium Tier:')}  ${chalk.whiteBright.bold(config.models.medium)} ${chalk.yellow(mediumCaps ? `[${mediumCaps}]` : '')}`,
    `  ${chalk.greenBright('🐇 Haiku Tier:')}   ${chalk.whiteBright.bold(config.models.haiku)} ${chalk.yellow(haikuCaps ? `[${haikuCaps}]` : '')}`,
  ];

  if (config.baseUrl) {
    lines.push(`  ${chalk.dim('Base URL:')}      ${chalk.blueBright(config.baseUrl)}`);
  }

  if (backupPath) {
    lines.push('');
    lines.push(chalk.dim(`Settings backup saved to: ${backupPath}`));
  }

  lines.push('');
  lines.push(chalk.yellow(`${figures.info} Run ${chalk.bold('claude')} in your terminal to start coding with this configuration.`));

  const box = boxen(lines.join('\n'), {
    title: chalk.bold.green(' Configuration Applied '),
    titleAlignment: 'center',
    padding: 1,
    margin: { top: 1, bottom: 1, left: 1, right: 1 },
    borderColor: 'green',
    borderStyle: 'round',
  });

  console.log(box);
}

export function printResetSuccess(backupPath: string | null): void {
  const lines: string[] = [
    chalk.bold.green(`${figures.tick} Claude Code has been restored to default Anthropic configuration!`),
    '',
    chalk.gray('• Custom ANTHROPIC_BASE_URL and custom tokens cleared.'),
    chalk.gray('• Model overrides and aliases reset to official defaults.'),
    chalk.gray('• Standard Claude Fable 5 / 3.7 Sonnet / Opus / 3.5 Sonnet / Haiku restored.'),
  ];

  if (backupPath) {
    lines.push('');
    lines.push(chalk.dim(`Backup saved to: ${backupPath}`));
  }

  const box = boxen(lines.join('\n'), {
    title: chalk.bold.green(' Defaults Restored '),
    padding: 1,
    margin: { top: 1, bottom: 1, left: 1, right: 1 },
    borderColor: 'green',
    borderStyle: 'round',
  });

  console.log(box);
}

export function printPresetsList(presets: Preset[]): void {
  console.log(chalk.bold.cyan('\n  Available Claude Shift Presets (4 Tiers):\n'));

  for (const p of presets) {
    const providerBadge = chalk.bgBlue.black(` ${p.provider.toUpperCase()} `);
    console.log(`  ${providerBadge} ${chalk.bold.white(p.name)} ${chalk.dim(`[--preset ${p.id}]`)}`);
    console.log(`     ${chalk.gray(p.description)}`);
    console.log(`     ${chalk.magentaBright('Epic:')} ${chalk.dim(p.models.epic)}  ${chalk.redBright('Large:')} ${chalk.dim(p.models.large)}  ${chalk.cyanBright('Medium:')} ${chalk.dim(p.models.medium)}  ${chalk.greenBright('Haiku:')} ${chalk.dim(p.models.haiku)}`);
    console.log();
  }

  console.log(chalk.dim(`  Switch instantly: ${chalk.white('claude-shift --preset <preset-id>')}\n`));
}
