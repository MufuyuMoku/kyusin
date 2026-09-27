#!/usr/bin/env node
// Menjalankan tes jendela asli (SPEC §11) lewat tauri-driver.
//
// Prasyarat:
//  - aplikasi sudah dibangun: `npx tauri build --debug --no-bundle`
//    (atau set KYUSIN_APP ke berkasnya);
//  - `tauri-driver` di PATH (`cargo install tauri-driver --locked`);
//  - Windows: msedgedriver yang versinya sama dengan WebView2, lewat
//    MSEDGEDRIVER (path berkas) atau folder di env `EdgeWebDriver`
//    (sudah ada di runner GitHub Windows).
//
// Tangkapan layar disimpan di ui/e2e/artifacts/.
//
// Catatan: tes memakai folder data aplikasi yang sama dengan pemakaian
// biasa. Pengaturan dikembalikan di akhir, tetapi pertandingan uji tercatat
// sebagai replay yang ditinggalkan.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { run as board } from './board.e2e.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '..', '..');
const exe = process.platform === 'win32' ? 'kyusin.exe' : 'kyusin';
const application = process.env.KYUSIN_APP ?? join(repo, 'target', 'debug', exe);
const artifacts = join(here, 'artifacts');
const port = 4444;
const base = `http://127.0.0.1:${port}`;

if (!existsSync(application)) {
	console.error(`Aplikasi tidak ditemukan: ${application}\nBangun dulu: npx tauri build --debug --no-bundle`);
	process.exit(2);
}

const args = ['--port', String(port)];
const native =
	process.env.MSEDGEDRIVER ??
	(process.env.EdgeWebDriver ? join(process.env.EdgeWebDriver, 'msedgedriver.exe') : undefined);
if (native) args.push('--native-driver', native);

const driver = spawn('tauri-driver', args, { stdio: ['ignore', 'inherit', 'inherit'] });
driver.on('error', (e) => {
	console.error(`tauri-driver gagal dijalankan: ${e.message}`);
	process.exit(2);
});

async function ready() {
	for (let i = 0; i < 100; i++) {
		try {
			const res = await fetch(`${base}/status`);
			if (res.ok) return;
		} catch {
			// belum siap
		}
		await new Promise((r) => setTimeout(r, 100));
	}
	throw new Error('tauri-driver tidak menjawab');
}

let code = 0;
try {
	await ready();
	console.log('papan Reversi (jendela asli):');
	await board({ base, application, artifacts });
	console.log(`LULUS. Tangkapan layar di ${artifacts}`);
} catch (e) {
	console.error(`GAGAL: ${e.message}`);
	code = 1;
} finally {
	driver.kill();
}
process.exit(code);
