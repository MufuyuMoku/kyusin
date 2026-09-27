/**
 * Setiap karakter yang tampil di UI harus ada di font yang dibundel (IBM
 * Plex Mono). Karakter yang tidak ada diambil peramban dari font cadangan
 * dengan lebar lain, sehingga grid karakter bergeser (misalnya papan
 * Reversi dengan ● dan ○). Lihat D-035.
 */

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, '..', '..', '..');

/** Kode titik yang dipetakan tabel `cmap` format 4 (BMP). */
function coverage(path: string): Set<number> {
	const d = readFileSync(path);
	const tables = d.readUInt16BE(4);
	let cmap = -1;
	for (let i = 0; i < tables; i++) {
		const rec = 12 + i * 16;
		if (d.toString('latin1', rec, rec + 4) === 'cmap') cmap = d.readUInt32BE(rec + 8);
	}
	assert.ok(cmap >= 0, `${path}: tanpa tabel cmap`);
	const out = new Set<number>();
	const subtables = d.readUInt16BE(cmap + 2);
	for (let i = 0; i < subtables; i++) {
		const st = cmap + d.readUInt32BE(cmap + 4 + i * 8 + 4);
		if (d.readUInt16BE(st) !== 4) continue;
		const segs = d.readUInt16BE(st + 6) / 2;
		for (let s = 0; s < segs; s++) {
			const end = d.readUInt16BE(st + 14 + s * 2);
			const start = d.readUInt16BE(st + 16 + segs * 2 + s * 2);
			for (let c = start; c <= end && c !== 0xffff; c++) out.add(c);
		}
	}
	return out;
}

function walk(dir: string, exts: string[]): string[] {
	return readdirSync(dir).flatMap((name) => {
		const p = join(dir, name);
		if (name === 'node_modules' || name === 'target' || name.startsWith('.')) return [];
		if (statSync(p).isDirectory()) return walk(p, exts);
		return exts.some((e) => name.endsWith(e)) ? [p] : [];
	});
}

test('pembaca cmap mengenali font yang dibundel', () => {
	const plex = coverage(join(repo, 'ui/static/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf'));
	assert.ok(plex.has('A'.codePointAt(0)!));
	assert.ok(plex.has('─'.codePointAt(0)!));
	assert.ok(!plex.has('●'.codePointAt(0)!), 'model: ● memang tidak ada di Plex Mono');
});

test('semua karakter UI ada di font yang dibundel', () => {
	const plex = coverage(join(repo, 'ui/static/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf'));
	const sources = [
		...walk(join(repo, 'ui/src'), ['.svelte', '.ts', '.json', '.css']).filter(
			(f) => !f.endsWith('.test.ts')
		),
		...walk(join(repo, 'tutorials'), ['.toml']),
		...walk(join(repo, 'crates'), ['i18n.toml', 'manifest.toml', 'tutorial.toml']),
		// Teks tampilan game (to_text) ada di kode Rust.
		...walk(join(repo, 'crates/games/src'), ['.rs'])
	];
	const missing = new Map<string, string>();
	for (const f of sources) {
		const lines = readFileSync(f, 'utf8').split('\n');
		for (const [i, line] of lines.entries()) {
			const trimmed = line.trim();
			// Komentar kode tidak tampil di layar.
			if (/^(\/\/|\*|\/\*|<!--|#)/.test(trimmed)) continue;
			for (const ch of line) {
				const cp = ch.codePointAt(0)!;
				if (cp > 0x7f && !plex.has(cp) && !missing.has(ch)) missing.set(ch, `${f}:${i + 1}`);
			}
		}
	}
	assert.deepEqual([...missing.entries()], []);
});
