/**
 * Cek kelengkapan terjemahan antarmuka (SPEC §4, D-027). CI gagal bila:
 * - ada kunci yang hanya ada di salah satu bahasa, atau nilainya kosong;
 * - kode memakai kunci yang tidak ada di berkas terjemahan;
 * - markup Svelte berisi teks UI yang ditulis langsung.
 */

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { EFFECT_IDS, GLOBAL_COMMANDS, LANG_IDS, THEME_IDS } from './commands.ts';
import { GREETINGS, greetingKey } from './boot/greetings.ts';

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, '..');
const id: Record<string, string> = JSON.parse(readFileSync(join(here, 'i18n/id.json'), 'utf8'));
const en: Record<string, string> = JSON.parse(readFileSync(join(here, 'i18n/en.json'), 'utf8'));

function files(dir: string, ext: string[]): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
		const p = join(dir, e.name);
		if (e.isDirectory()) return files(p, ext);
		return ext.some((x) => e.name.endsWith(x)) ? [p] : [];
	});
}

test('kunci terjemahan sama di id dan en, tanpa nilai kosong', () => {
	const onlyId = Object.keys(id).filter((k) => !(k in en));
	const onlyEn = Object.keys(en).filter((k) => !(k in id));
	assert.deepEqual(onlyId, [], `hilang di en.json: ${onlyId}`);
	assert.deepEqual(onlyEn, [], `hilang di id.json: ${onlyEn}`);
	const empty = Object.entries({ ...id }).filter(([k, v]) => !v.trim() || !en[k]?.trim());
	assert.deepEqual(empty, []);
});

test('setiap literal berbentuk kunci terjemahan ada di berkas', () => {
	// Awalan kunci yang dikenal (bagian sebelum titik pertama).
	const prefixes = new Set(Object.keys(id).map((k) => k.split('.')[0]));
	const missing: string[] = [];
	for (const f of files(src, ['.svelte', '.ts'])) {
		if (f.endsWith('.test.ts') || f.includes('devmock')) continue;
		const text = readFileSync(f, 'utf8');
		for (const m of text.matchAll(/'([a-z_]+(?:\.[a-z0-9_]+)+)'/g)) {
			const key = m[1];
			if (prefixes.has(key.split('.')[0]) && !(key in id)) missing.push(`${f}: ${key}`);
		}
	}
	assert.deepEqual(missing, []);
});

test('setiap sapaan punya parameter yang sama di kedua bahasa', () => {
	for (const g of GREETINGS) {
		const k = greetingKey(g.id);
		const params = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
		assert.deepEqual(params(id[k]), params(en[k]), k);
	}
});

test('setiap kunci yang dipakai kode ada di berkas terjemahan', () => {
	const used = new Set<string>();
	for (const f of files(src, ['.svelte', '.ts'])) {
		if (f.endsWith('.test.ts')) continue;
		const text = readFileSync(f, 'utf8');
		for (const m of text.matchAll(/\bt\(\s*'([^']+)'/g)) used.add(m[1]);
		// Kunci pilihan di dalam t(kondisi ? 'a' : 'b').
		for (const m of text.matchAll(/\bt\(\s*[^'()]*\?\s*'([^']+)'\s*:\s*'([^']+)'/g)) {
			used.add(m[1]);
			used.add(m[2]);
		}
	}
	// Kunci yang dibentuk dari nilai (template literal).
	for (const c of GLOBAL_COMMANDS) used.add(c.summary);
	for (const s of ['menu', 'game', 'tutorial', 'settings', 'help']) used.add(`crumb.${s}`);
	for (const th of THEME_IDS) used.add(`settings.theme.${th}`);
	for (const fx of EFFECT_IDS) used.add(`settings.fx.${fx}`);
	for (const l of LANG_IDS) used.add(`settings.lang.${l}`);
	for (const m of ['verbose', 'cinematic', 'greeting', 'off']) used.add(`settings.boot.${m}`);
	for (const s of ['match', 'replay']) used.add(`crumb.${s}`);
	for (const l of [1, 2, 3]) used.add(`level.${l}`);
	for (const s of ['commitments', 'round_seed', 'moves', 'result', 'state_hash'])
		used.add(`verify.step.${s}`);
	for (const g of GREETINGS) used.add(greetingKey(g.id));

	const missing = [...used].filter((k) => !(k in id) || !(k in en));
	assert.deepEqual(missing, []);
});

/** Nama yang tidak diterjemahkan (SPEC: nama produk ditulis persis). */
const ALLOWED_WORDS = new Set(['KyuSin']);

/** Teks UI mentah di markup Svelte (atribut dan simpul teks). */
function hardcoded(source: string): string[] {
	const found: string[] = [];
	let text = source
		.replace(/<script[\s\S]*?<\/script>/g, '')
		.replace(/<style[\s\S]*?<\/style>/g, '')
		.replace(/<!--[\s\S]*?-->/g, '');
	for (const m of text.matchAll(/\b(aria-label|title|placeholder|alt)="([^"{]*[A-Za-z][^"]*)"/g)) {
		found.push(`${m[1]}="${m[2]}"`);
	}
	// Buang ekspresi {…} (berulang untuk yang bersarang), tag, dan entitas.
	let prev = '';
	while (prev !== text) {
		prev = text;
		text = text.replace(/\{[^{}]*\}/g, '');
	}
	text = text.replace(/<[^>]*>/g, ' ').replace(/&[a-z]+;/g, ' ');
	for (const word of text.match(/[A-Za-z][A-Za-z']+/g) ?? []) {
		if (!ALLOWED_WORDS.has(word)) found.push(word);
	}
	return found;
}

test('pemeriksa teks mentah menangkap teks dan atribut', () => {
	assert.deepEqual(hardcoded(`<p>Halo</p><b>{t('x')}</b>&nbsp;`), ['Halo']);
	assert.deepEqual(hardcoded('<input aria-label="Perintah" />'), ['aria-label="Perintah"']);
	assert.deepEqual(hardcoded('<span>{a ? `x` : {b}}</span> KyuSin'), []);
});

test('markup Svelte tidak berisi teks UI yang ditulis langsung', () => {
	const problems = files(src, ['.svelte']).flatMap((f) =>
		hardcoded(readFileSync(f, 'utf8')).map((w) => `${f}: ${w}`)
	);
	assert.deepEqual(problems, []);
});
