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
// Tes memakai folder data sementara (KYUSIN_DATA_DIR untuk SQLite; folder
// WebView2 disiapkan msedgedriver), jadi data pemain tidak tersentuh (D-040).

import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
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

// Folder data sementara: WebView2 (WEBVIEW2_USER_DATA_FOLDER) dan SQLite
// (KYUSIN_DATA_DIR), jadi data pemain tidak tersentuh (D-040).
const dataDir = mkdtempSync(join(tmpdir(), 'kyusin-e2e-'));
const debugPort = 9222;
const driver = spawn('tauri-driver', args, { stdio: ['ignore', 'inherit', 'inherit'] });
driver.on('error', (e) => {
	console.error(`tauri-driver gagal dijalankan: ${e.message}`);
	process.exit(2);
});

// Aplikasi dijalankan sendiri dengan port DevTools tetap, lalu msedgedriver
// menempel lewat debuggerAddress (D-040).
const app = spawn(application, [], {
	stdio: 'ignore',
	env: {
		...process.env,
		KYUSIN_DATA_DIR: join(dataDir, 'data'),
		WEBVIEW2_USER_DATA_FOLDER: join(dataDir, 'webview'),
		WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${debugPort}`
	}
});

async function devtools() {
	for (let i = 0; i < 150; i++) {
		try {
			const res = await fetch(`http://127.0.0.1:${debugPort}/json/version`);
			if (res.ok) return;
		} catch {
			// WebView2 belum siap
		}
		await new Promise((r) => setTimeout(r, 200));
	}
	throw new Error('port DevTools WebView2 tidak terbuka');
}

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
	await devtools();
	console.log('papan Reversi (jendela asli):');
	await board({ base, debuggerAddress: `127.0.0.1:${debugPort}`, artifacts });
	console.log(`LULUS. Tangkapan layar di ${artifacts}`);
} catch (e) {
	console.error(`GAGAL: ${e.message}`);
	code = 1;
} finally {
	app.kill();
	driver.kill();
	await new Promise((r) => setTimeout(r, 500));
	try {
		rmSync(dataDir, { recursive: true, force: true });
	} catch {
		// Berkas mungkin masih dipakai aplikasi yang sedang ditutup.
	}
}
process.exit(code);
