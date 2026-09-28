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

import { execFileSync, spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { run as board } from './board.e2e.mjs';
import { run as chess } from './chess.e2e.mjs';
import { run as pause } from './pause.e2e.mjs';
import { Session } from './webdriver.mjs';
import { mkdirSync } from 'node:fs';

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
const driver = spawn('tauri-driver', args, { stdio: ['ignore', 'inherit', 'inherit'] });
driver.on('error', (e) => {
	console.error(`tauri-driver gagal dijalankan: ${e.message}`);
	process.exit(2);
});

// Aplikasi dijalankan sendiri dengan port DevTools tetap, lalu msedgedriver
// menempel lewat debuggerAddress (D-040).
const app = spawn(application, [], {
	stdio: ['ignore', 'inherit', 'inherit'],
	env: {
		...process.env,
		RUST_BACKTRACE: '1',
		KYUSIN_DATA_DIR: join(dataDir, 'data'),
		WEBVIEW2_USER_DATA_FOLDER: join(dataDir, 'webview'),
		// Port 0: sistem memilih port bebas; nomornya dibaca dari DevToolsActivePort.
		WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=0'
	}
});

let appExit = null;
app.on('exit', (code, signal) => {
	appExit = `kode ${code}, sinyal ${signal}`;
	console.error(`aplikasi berhenti (${appExit})`);
});
app.on('error', (e) => console.error(`aplikasi gagal dijalankan: ${e.message}`));

/** Nomor port DevTools dari berkas yang ditulis WebView2 di folder datanya. */
function activePort() {
	const file = join(dataDir, 'webview', 'EBWebView', 'DevToolsActivePort');
	return existsSync(file) ? Number(readFileSync(file, 'utf8').split(/\r?\n/)[0]) : null;
}

async function devtools() {
	for (let i = 0; i < 450; i++) {
		if (appExit) throw new Error(`aplikasi berhenti sebelum port DevTools terbuka (${appExit})`);
		const port = activePort();
		if (port) {
			try {
				const res = await fetch(`http://127.0.0.1:${port}/json/version`);
				if (res.ok) return port;
			} catch {
				// belum menerima koneksi
			}
		}
		await new Promise((r) => setTimeout(r, 200));
	}
	throw new Error('port DevTools WebView2 tidak terbuka');
}

/** Diagnostik saat gagal: proses WebView2 dan isi folder data uji. */
function diagnose() {
	try {
		const ps =
			"Get-CimInstance Win32_Process | Where-Object { $_.Name -match 'kyusin|msedgewebview2' } | " +
			"ForEach-Object { $_.Name + ' :: ' + $_.CommandLine }";
		console.error(execFileSync('powershell', ['-NoProfile', '-Command', ps], { encoding: 'utf8' }));
		const list = (d) => (existsSync(d) ? readdirSync(d, { recursive: true }).slice(0, 40) : '(tidak ada)');
		console.error('folder webview:', list(join(dataDir, 'webview')));
		console.error('folder data:', list(join(dataDir, 'data')));
	} catch (e) {
		console.error(`diagnostik gagal: ${e.message}`);
	}
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
	const port = await devtools();
	console.log(`WebView2 DevTools di port ${port}`);
	mkdirSync(artifacts, { recursive: true });
	const s = await Session.attach(base, `127.0.0.1:${port}`);
	const log = (m) => console.log(`  ${m}`);
	try {
		await s.waitFor(() => !!document.querySelector('.crt'), 'jendela KyuSin');
		console.log('papan Reversi (jendela asli):');
		await board(s, artifacts, log);
		console.log('papan catur (jendela asli):');
		await chess(s, artifacts, log);
		console.log('menu jeda, penundaan, dan kursor (jendela asli):');
		await pause(s, artifacts, log);
	} finally {
		await s.end().catch(() => {});
	}
	console.log(`LULUS. Tangkapan layar di ${artifacts}`);
} catch (e) {
	console.error(`GAGAL: ${e.message}`);
	if (process.platform === 'win32') diagnose();
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
