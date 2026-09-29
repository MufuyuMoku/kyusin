/**
 * Keadaan antarmuka: tumpukan layar, katalog dari registry, tutorial yang
 * sedang berjalan, dan keluaran konsol.
 */

import {
	api,
	type AppInfo,
	type Allowance,
	type Category,
	type MatchState,
	type ReplayData,
	type TutorialState
} from './backend';
import { motion, settings } from './settings.svelte';

export type Screen =
	| { name: 'menu' }
	| { name: 'game'; id: string }
	| { name: 'tutorial'; id: string }
	| { name: 'match'; id: string }
	| { name: 'replay'; id: number }
	| { name: 'pgn' }
	| { name: 'settings' }
	| { name: 'help' }
	| { name: 'profile' }
	| { name: 'stats' };

export const app = $state({
	stack: [{ name: 'menu' }] as Screen[],
	info: null as AppInfo | null,
	catalog: [] as Category[],
	tutorial: null as TutorialState | null,
	/** Pertandingan yang sedang berjalan melawan bot. */
	match: null as MatchState | null,
	/** Replay yang sedang dibuka. */
	replay: null as ReplayData | null,
	/** Jam pertandingan terakhir, untuk "main lagi". */
	lastClock: null as { minutes: number; increment: number } | null,
	/** Tunjangan harian yang diperiksa saat aplikasi dibuka (M4). */
	allowance: null as Allowance | null,
	/** Menu jeda pertandingan terbuka (SPEC §4). */
	pauseMenu: false,
	/** Konsol sedang dibuka lewat `:` atau `` ` ``. */
	consoleOpen: false,
	output: [] as string[],
	/** Kesalahan memuat katalog (teks backend dua bahasa atau lainnya). */
	error: null as unknown
});

export function current(): Screen {
	return app.stack[app.stack.length - 1];
}

export function go(screen: Screen) {
	app.stack.push(screen);
}

/** Menutup tutorial/pertandingan bila layarnya tidak lagi tampil. */
function cleanup() {
	const names = app.stack.map((s) => s.name);
	if (!names.includes('tutorial') && app.tutorial) stopTutorial();
	if (!names.includes('match') && app.match) leaveMatch();
	if (!names.includes('replay')) app.replay = null;
}

export function back() {
	if (app.stack.length > 1) app.stack.pop();
	cleanup();
}

/**
 * `Esc`, tombol kembali, atau perintah `back`. Di tengah pertandingan yang
 * belum selesai ini membuka menu jeda, bukan keluar (SPEC §4 Rev. 9); saat
 * menu jeda terbuka, ini sama dengan Lanjutkan.
 */
export function requestBack() {
	if (current().name === 'match' && app.match && !app.match.over) {
		if (app.pauseMenu) resumePlay();
		else openPause();
		return;
	}
	back();
}

export async function openPause() {
	app.pauseMenu = true;
	const a = await api();
	app.match = await a.match_pause();
}

/** Lanjutkan: menu jeda ditutup, jam dan bot berjalan lagi. */
export async function resumePlay() {
	app.pauseMenu = false;
	const a = await api();
	app.match = await a.match_unpause();
	runBots();
}

/** Tunda & keluar: kembali ke layar game, pertandingan bisa dilanjutkan. */
export async function suspendMatch() {
	const a = await api();
	await a.match_suspend();
	app.pauseMenu = false;
	app.match = null;
	back();
}

/** Menyerah dari menu jeda. */
export async function resignMatch() {
	app.pauseMenu = false;
	const a = await api();
	app.match = await a.match_resign();
}

/** Melanjutkan pertandingan tertunda dari layar game. */
export async function resumeSuspended(id: string) {
	const a = await api();
	app.pauseMenu = false;
	app.match = await a.match_resume(id);
	if (current().name === 'match') app.stack.pop();
	go({ name: 'match', id });
	runBots();
}

/** Pertandingan tertunda dihitung menyerah, lalu pertandingan baru dimulai. */
export async function forfeitAndStart(id: string, level: number, seat: number, clock: Clock = null) {
	const a = await api();
	await a.suspended_forfeit(id);
	await startMatch(id, level, seat, clock);
}

export function home() {
	app.stack = [{ name: 'menu' }];
	cleanup();
}

export function print(text: string) {
	app.output.push(...text.replace(/\n$/, '').split('\n'));
	if (app.output.length > 300) app.output.splice(0, app.output.length - 300);
}

export function games() {
	return app.catalog.flatMap((c) => c.games);
}

export function findGame(id: string) {
	return games().find((g) => g.id === id);
}

/**
 * Waktu (ms sejak jendela dibuka, `performance.now()`) saat tiap langkah
 * startup benar-benar selesai. Boot Verbose menampilkannya apa adanya
 * (SPEC §4: baris boot mencerminkan startup sungguhan).
 */
export const marks = {
	settings: 0,
	mount: 0,
	info: null as number | null,
	catalog: null as number | null,
	fonts: null as number | null
};

/** Font yang dibundel dan apakah benar-benar termuat. */
export const fonts: { name: string; ok: boolean }[] = [];

const FONT_FACES = ['VT323', 'IBM Plex Mono'];

export async function load() {
	marks.mount = performance.now();
	const a = await api();
	app.info = await a.app_info();
	marks.info = performance.now();
	app.catalog = await a.catalog();
	marks.catalog = performance.now();
	// Tunjangan harian (SPEC §6.7) memakai tanggal lokal pemain; saldo
	// sebelumnya sudah ada di `app.info.profile` untuk sapaan (D-039).
	app.allowance = await a.chips_daily(localDate()).catch(() => null);
	await document.fonts.ready;
	for (const name of FONT_FACES) {
		// Memuat eksplisit supaya hasilnya tidak bergantung pada apakah teks
		// dengan font itu sudah tampil.
		const loaded = await document.fonts.load(`16px "${name}"`).catch(() => []);
		fonts.push({ name, ok: loaded.length > 0 });
	}
	marks.fonts = performance.now();
}

export async function startTutorial(id: string) {
	const a = await api();
	app.tutorial = await a.tutorial_start(id);
	if (current().name === 'tutorial') app.stack.pop();
	go({ name: 'tutorial', id });
}

export async function tutorialAct(command: string) {
	const a = await api();
	app.tutorial = await a.tutorial_act(command);
}

export async function tutorialNext() {
	const a = await api();
	app.tutorial = await a.tutorial_next();
}

function stopTutorial() {
	app.tutorial = null;
	api().then((a) => a.tutorial_stop());
}

/** Perintah game yang sah saat ini (konkret), untuk konsol. */
export function gameCommands(): string[] {
	const actions = current().name === 'match' ? app.match?.actions : app.tutorial?.actions;
	return actions?.flatMap((a) => a.concrete ?? [a.usage]) ?? [];
}

/** Jeda sebelum langkah bot, supaya langkahnya terlihat (tanpa jeda saat reduced motion). */
const BOT_DELAY_MS = 450;

let stepping = false;

/** Menjalankan langkah bot satu per satu selama giliran bot. */
async function runBots() {
	if (stepping) return;
	stepping = true;
	try {
		const a = await api();
		while (app.match?.bot_turn && !app.match.paused && current().name === 'match') {
			await new Promise((r) => setTimeout(r, motion.reduced ? 0 : BOT_DELAY_MS));
			if (current().name !== 'match') break;
			app.match = await a.match_step();
		}
	} finally {
		stepping = false;
	}
}

export type Clock = { minutes: number; increment: number } | null;

/** Tanggal kalender lokal `YYYY-MM-DD`. */
export function localDate(d = new Date()): string {
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export async function startMatch(id: string, level: number, seat: number, clock: Clock = null) {
	const a = await api();
	// Game casino: tunjangan harian bisa berlaku lagi (hari berganti, atau
	// saldo sudah di bawah ambang).
	if (findGame(id)?.lawan === 'bandar') {
		const got = await a.chips_daily(localDate()).catch(() => null);
		if (got?.granted) app.allowance = got;
	}
	const seed = settings.playerSeed.trim() || null;
	app.lastClock = clock;
	app.pauseMenu = false;
	app.match = await a.match_start(id, level, seat, seed, clock);
	if (current().name === 'match') app.stack.pop();
	go({ name: 'match', id });
	runBots();
}

export async function matchAct(command: string) {
	const a = await api();
	app.match = await a.match_act(command);
	runBots();
}

/** Jam pemain di layar habis; host memeriksa dengan jamnya sendiri. */
export async function matchFlag() {
	const a = await api();
	app.match = await a.match_flag();
	runBots();
}

/** Membuka teks PGN di penampil replay (tidak disimpan). */
export async function openPgn(text: string) {
	const a = await api();
	app.replay = await a.pgn_open(text);
	go({ name: 'replay', id: 0 });
}

function leaveMatch() {
	app.match = null;
	app.pauseMenu = false;
	api().then((a) => a.match_leave());
}

export async function openReplay(id: number) {
	const a = await api();
	app.replay = await a.replay_open(id);
	go({ name: 'replay', id });
}
