/**
 * Teks `help`: perintah global dan daftar game, dibangkitkan dari manifest
 * (SPEC §5.2). Dipakai konsol dan layar Bantuan.
 */

import type { Category } from './backend';
import { GLOBAL_COMMANDS } from './commands';

export function helpText(catalog: Category[]): string {
	const lines: string[] = ['PERINTAH'];
	const width = Math.max(...GLOBAL_COMMANDS.map((c) => `${c.name} ${c.args}`.trim().length));
	for (const c of GLOBAL_COMMANDS) {
		lines.push(`  ${`${c.name} ${c.args}`.trim().padEnd(width)}  ${c.summary}`);
	}
	lines.push(
		'',
		'TOMBOL',
		'  : atau `        Buka konsol perintah',
		'  Tab             Autocomplete (di konsol) / pindah kontrol',
		'  ↑ ↓             Riwayat (di konsol) / pindah pilihan',
		'  Enter           Pilih',
		'  Esc             Kembali / tutup konsol',
		'',
		'GAME'
	);
	const games = catalog.flatMap((c) => c.games);
	if (games.length === 0) lines.push('  (belum ada cartridge)');
	const idWidth = Math.max(0, ...games.map((g) => g.id.length));
	for (const g of games) lines.push(`  ${g.id.padEnd(idWidth)}  ${g.nama}`);
	return lines.join('\n');
}

export function lsText(catalog: Category[], key?: string): string {
	const cats = key ? catalog.filter((c) => c.key === key) : catalog;
	if (cats.length === 0) return key ? `ls: tidak ada kategori \`${key}\`` : '(belum ada cartridge)';
	return cats
		.map((c) => [`${c.key}/`, ...c.games.map((g) => `  ${g.id}  ${g.nama}`)].join('\n'))
		.join('\n');
}
