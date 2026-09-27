/**
 * Teks `help` dan `ls`: perintah global dan daftar game, dibangkitkan dari
 * manifest (SPEC §5.2), dalam bahasa aktif. Dipakai konsol dan layar Bantuan.
 */

import type { Category } from './backend';
import { GLOBAL_COMMANDS } from './commands';
import { L, t, type Key } from './i18n.svelte';

export function helpText(catalog: Category[]): string {
	const lines: string[] = [t('help.commands')];
	const width = Math.max(...GLOBAL_COMMANDS.map((c) => `${c.name} ${c.args}`.trim().length));
	for (const c of GLOBAL_COMMANDS) {
		lines.push(`  ${`${c.name} ${c.args}`.trim().padEnd(width)}  ${t(c.summary as Key)}`);
	}
	const keys: [string, Key][] = [
		[': / `', 'help.key.console'],
		['Tab', 'help.key.tab'],
		['↑ ↓', 'help.key.arrows'],
		['Enter', 'help.key.enter'],
		['Esc', 'help.key.esc']
	];
	lines.push('', t('help.keys'));
	for (const [k, desc] of keys) lines.push(`  ${k.padEnd(14)}  ${t(desc)}`);
	lines.push('', t('help.games'));
	const games = catalog.flatMap((c) => c.games);
	if (games.length === 0) lines.push(`  ${t('help.no_games')}`);
	const idWidth = Math.max(0, ...games.map((g) => g.id.length));
	for (const g of games) lines.push(`  ${g.id.padEnd(idWidth)}  ${L(g.nama)}`);
	return lines.join('\n');
}

export function lsText(catalog: Category[], key?: string): string {
	const cats = key ? catalog.filter((c) => c.key === key) : catalog;
	if (cats.length === 0) return key ? t('console.no_category', { key }) : t('help.no_games');
	return cats
		.map((c) => [`${c.key}/`, ...c.games.map((g) => `  ${g.id}  ${L(g.nama)}`)].join('\n'))
		.join('\n');
}
