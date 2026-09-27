/**
 * Mode perintah (SPEC §4): daftar perintah global, pengurai, autocomplete,
 * dan riwayat. Tanpa ketergantungan pada Svelte supaya bisa diuji dengan
 * `node --test`. Daftar game dan perintah game datang dari manifest
 * (lewat `Context`), tidak ditulis di sini.
 */

export interface CommandDef {
	/** Kata perintah; selalu Inggris di kedua bahasa (SPEC §4). */
	name: string;
	args: string;
	/** Kunci terjemahan deskripsinya. */
	summary: `cmd.${string}`;
}

export const GLOBAL_COMMANDS: CommandDef[] = [
	{ name: 'help', args: '', summary: 'cmd.help' },
	{ name: 'ls', args: '[category]', summary: 'cmd.ls' },
	{ name: 'man', args: '<id>', summary: 'cmd.man' },
	{ name: 'tutorial', args: '<id>', summary: 'cmd.tutorial' },
	{ name: 'play', args: '<id> [level]', summary: 'cmd.play' },
	{ name: 'verify', args: '', summary: 'cmd.verify' },
	{ name: 'menu', args: '', summary: 'cmd.menu' },
	{ name: 'back', args: '', summary: 'cmd.back' },
	{ name: 'settings', args: '', summary: 'cmd.settings' },
	{ name: 'theme', args: '<p1|p3|p4>', summary: 'cmd.theme' },
	{ name: 'fx', args: '<effect> <on|off>', summary: 'cmd.fx' },
	{ name: 'lang', args: '<id|en>', summary: 'cmd.lang' },
	{ name: 'clear', args: '', summary: 'cmd.clear' }
];

export const THEME_IDS = ['p1', 'p3', 'p4'];
export const EFFECT_IDS = ['scanline', 'glow', 'curve', 'flicker'];
export const LANG_IDS = ['id', 'en'];

export interface Context {
	gameIds: string[];
	categoryKeys: string[];
	/** Perintah game yang sah saat ini, dalam bentuk konkret. */
	gameCommands: string[];
}

export type Parsed =
	| { kind: 'empty' }
	| { kind: 'global'; name: string; args: string[] }
	| { kind: 'game'; command: string }
	| { kind: 'unknown'; name: string };

export function tokens(input: string): string[] {
	return input.trim().split(/\s+/).filter(Boolean);
}

export function parse(input: string, ctx: Context): Parsed {
	const t = tokens(input);
	if (t.length === 0) return { kind: 'empty' };
	if (GLOBAL_COMMANDS.some((c) => c.name === t[0])) {
		return { kind: 'global', name: t[0], args: t.slice(1) };
	}
	if (ctx.gameCommands.some((c) => tokens(c)[0] === t[0])) {
		return { kind: 'game', command: t.join(' ') };
	}
	return { kind: 'unknown', name: t[0] };
}

function candidates(done: string[], ctx: Context): string[] {
	const pos = done.length;
	const games = ctx.gameCommands.map(tokens);
	if (pos === 0) {
		const verbs = games.map((g) => g[0]);
		return unique([...GLOBAL_COMMANDS.map((c) => c.name), ...verbs]);
	}
	const head = done[0];
	if (GLOBAL_COMMANDS.some((c) => c.name === head)) {
		if (pos === 1) {
			switch (head) {
				case 'man':
				case 'tutorial':
				case 'play':
					return ctx.gameIds;
				case 'ls':
					return ctx.categoryKeys;
				case 'theme':
					return THEME_IDS;
				case 'fx':
					return EFFECT_IDS;
				case 'lang':
					return LANG_IDS;
				case 'help':
					return GLOBAL_COMMANDS.map((c) => c.name);
			}
		}
		if (pos === 2 && head === 'fx') return ['on', 'off'];
		if (pos === 2 && head === 'play') return ['1', '2', '3'];
		return [];
	}
	return unique(
		games
			.filter((g) => g.length > pos && done.every((d, i) => g[i] === d))
			.map((g) => g[pos])
	);
}

function unique(xs: string[]): string[] {
	return [...new Set(xs)];
}

function commonPrefix(xs: string[]): string {
	if (xs.length === 0) return '';
	let p = xs[0];
	for (const x of xs) while (!x.startsWith(p)) p = p.slice(0, -1);
	return p;
}

export interface Completion {
	/** Isi input baru. */
	value: string;
	/** Pilihan yang cocok (ditampilkan bila lebih dari satu). */
	options: string[];
}

export function complete(input: string, ctx: Context): Completion {
	const endsWithSpace = /\s$/.test(input) || input === '';
	const t = tokens(input);
	const done = endsWithSpace ? t : t.slice(0, -1);
	const partial = endsWithSpace ? '' : (t.at(-1) ?? '');
	const options = candidates(done, ctx).filter((c) => c.startsWith(partial));
	if (options.length === 0) return { value: input, options: [] };
	const prefix = done.length ? done.join(' ') + ' ' : '';
	if (options.length === 1) return { value: prefix + options[0] + ' ', options };
	return { value: prefix + commonPrefix(options), options };
}

/** Riwayat perintah dengan panah atas/bawah. */
export class History {
	entries: string[];
	private cursor: number;
	private draft = '';
	readonly limit: number;

	constructor(entries: string[] = [], limit = 100) {
		this.limit = limit;
		this.entries = entries.slice(-limit);
		this.cursor = this.entries.length;
	}

	push(line: string) {
		const l = line.trim();
		if (l && this.entries.at(-1) !== l) this.entries.push(l);
		if (this.entries.length > this.limit) this.entries.shift();
		this.cursor = this.entries.length;
		this.draft = '';
	}

	up(current: string): string {
		if (this.cursor === this.entries.length) this.draft = current;
		if (this.cursor > 0) this.cursor -= 1;
		return this.entries[this.cursor] ?? current;
	}

	down(): string {
		if (this.cursor < this.entries.length) this.cursor += 1;
		return this.cursor === this.entries.length ? this.draft : this.entries[this.cursor];
	}
}
