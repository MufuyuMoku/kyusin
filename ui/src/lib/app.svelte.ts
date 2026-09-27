/**
 * Keadaan antarmuka: tumpukan layar, katalog dari registry, tutorial yang
 * sedang berjalan, dan keluaran konsol.
 */

import { api, type AppInfo, type Category, type TutorialState } from './backend';

export type Screen =
	| { name: 'menu' }
	| { name: 'game'; id: string }
	| { name: 'tutorial'; id: string }
	| { name: 'settings' }
	| { name: 'help' };

export const app = $state({
	stack: [{ name: 'menu' }] as Screen[],
	info: null as AppInfo | null,
	catalog: [] as Category[],
	tutorial: null as TutorialState | null,
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

export function back() {
	if (app.stack.length > 1) app.stack.pop();
	if (current().name !== 'tutorial' && app.tutorial) stopTutorial();
}

export function home() {
	app.stack = [{ name: 'menu' }];
	if (app.tutorial) stopTutorial();
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
	return app.tutorial?.actions.flatMap((a) => a.concrete ?? [a.usage]) ?? [];
}
