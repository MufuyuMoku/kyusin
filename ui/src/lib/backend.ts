/**
 * Perintah Tauri yang dipakai antarmuka. Bentuknya mengikuti
 * `src-tauri/src/lib.rs`. Antarmuka tidak menyimpan aturan game apa pun:
 * katalog datang dari registry, aksi dikirim sebagai perintah teks.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

export type ParamKind =
	| { type: 'int'; min: number; max: number; step: number }
	| { type: 'choice'; options: string[] };

export type ActionSpec =
	| { kind: 'fixed'; command: string }
	| { kind: 'template'; verb: string; params: ({ name: string } & ParamKind)[] };

export interface CommandDoc {
	pola: string;
	ringkas: string;
}

/** Manifest cartridge (SPEC §5.2); kunci mengikuti SPEC. */
export interface Game {
	id: string;
	nama: string;
	kategori: string;
	pemain_min: number;
	pemain_maks: number;
	jenis: 'giliran' | 'real-time';
	lawan: 'bandar' | 'bot' | 'tidak-ada';
	kompetitif: boolean;
	lan: boolean;
	agen: boolean;
	rtp: number | null;
	tutorial: string;
	perintah: CommandDoc[];
	rtp_line: string | null;
}

export interface Category {
	key: string;
	label: string;
	games: Game[];
}

export interface Step {
	teks: string;
	sebelum: string[];
	aksi: string | null;
	sorot: string[];
	petunjuk: string | null;
}

export type Feedback = { kind: 'correct' } | { kind: 'wrong'; hint: string };

export interface ActionView {
	spec: ActionSpec;
	usage: string;
	concrete: string[] | null;
}

export interface TutorialState {
	game: string;
	title: string;
	index: number;
	total: number;
	finished: boolean;
	step: Step | null;
	view_text: string;
	actions: ActionView[];
	feedback: Feedback | null;
}

export interface AppInfo {
	name: string;
	version: string;
}

type Api = {
	app_info(): Promise<AppInfo>;
	catalog(): Promise<Category[]>;
	man(id: string): Promise<string>;
	tutorial_start(id: string): Promise<TutorialState>;
	tutorial_act(command: string): Promise<TutorialState>;
	tutorial_next(): Promise<TutorialState>;
	tutorial_stop(): Promise<void>;
};

const tauriApi: Api = {
	app_info: () => invoke('app_info'),
	catalog: () => invoke('catalog'),
	man: (id) => invoke('man', { id }),
	tutorial_start: (id) => invoke('tutorial_start', { id }),
	tutorial_act: (command) => invoke('tutorial_act', { command }),
	tutorial_next: () => invoke('tutorial_next'),
	tutorial_stop: () => invoke('tutorial_stop')
};

let resolved: Api | null = null;

/**
 * Di luar jendela Tauri (hanya `npm run dev` di peramban, untuk pengecekan
 * tampilan) dipakai tiruan berisi fixture. Build produksi tidak memuatnya.
 */
export async function api(): Promise<Api> {
	if (resolved) return resolved;
	if (!isTauri() && import.meta.env.DEV) {
		resolved = (await import('./devmock')).devApi;
	} else {
		resolved = tauriApi;
	}
	return resolved;
}
