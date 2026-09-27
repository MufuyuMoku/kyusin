#!/usr/bin/env node
// Pemeriksa lisensi paket npm yang masuk build produksi (SPEC §3, D-006).
//
// Daftar paket diambil dari bundle klien yang sebenarnya: plugin
// `kyusin-bundled-packages` di ui/vite.config.ts mencatatnya ke
// ui/.svelte-kit/bundled-packages.json saat `npm run build`. Alat build
// (Vite, pemampat CSS, dan sebagainya) tidak ikut dikirim, jadi tidak
// diperiksa. Ekspresi SPDX dicocokkan dengan daftar izin; GPL/LGPL/AGPL
// ditolak karena tidak ada di daftar.
//
// Jalankan setelah `npm run build`:  node scripts/check-npm-licenses.mjs

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ALLOW = new Set([
	'MIT',
	'MIT-0',
	'Apache-2.0',
	'BSD-2-Clause',
	'BSD-3-Clause',
	'0BSD',
	'ISC',
	'Zlib',
	'OFL-1.1',
	'CC0-1.0',
	'Unicode-3.0'
]);

/** Mengevaluasi ekspresi SPDX sederhana: OR = salah satu, AND = semua. */
function allowed(expr) {
	if (!expr || typeof expr !== 'string') return false;
	const e = expr.trim().replace(/^\((.*)\)$/, '$1');
	if (/\sOR\s/.test(e)) return e.split(/\s+OR\s+/).some(allowed);
	if (/\sAND\s/.test(e)) return e.split(/\s+AND\s+/).every(allowed);
	return ALLOW.has(e.replace(/[()]/g, '').trim());
}

function licenseOf(pkg) {
	if (typeof pkg.license === 'string') return pkg.license;
	if (pkg.license && typeof pkg.license.type === 'string') return pkg.license.type;
	if (Array.isArray(pkg.licenses)) return pkg.licenses.map((l) => l.type ?? l).join(' OR ');
	return null;
}

const ui = join(dirname(fileURLToPath(import.meta.url)), '..', 'ui');
const listPath = join(ui, '.svelte-kit', 'bundled-packages.json');
if (!existsSync(listPath)) {
	console.error(`${listPath} belum ada; jalankan \`npm run build\` di ui/ dulu.`);
	process.exit(2);
}

const dirs = JSON.parse(readFileSync(listPath, 'utf8'));
const bad = [];
for (const dir of dirs) {
	const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
	const lic = licenseOf(pkg);
	const ok = allowed(lic);
	console.log(`  ${ok ? 'ok ' : 'NO '} ${pkg.name}@${pkg.version}  ${lic ?? '(tanpa lisensi)'}`);
	if (!ok) bad.push(pkg.name);
}

console.log(`Paket npm di bundle produksi: ${dirs.length}`);
if (dirs.length === 0) {
	console.error('Daftar kosong: plugin pencatat tidak berjalan?');
	process.exit(2);
}
if (bad.length) {
	console.error(`Lisensi tidak diizinkan: ${bad.join(', ')}`);
	process.exit(1);
}
console.log('Semua lisensi diizinkan.');
