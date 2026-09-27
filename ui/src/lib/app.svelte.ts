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
	error: null as string | null
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

export async function load() {
	const a = await api();
	app.info = await a.app_info();
	app.catalog = await a.catalog();
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
