/**
 * Perintah Tauri yang dipakai antarmuka. Bentuknya mengikuti
 * `src-tauri/src/lib.rs`. Antarmuka tidak menyimpan aturan game apa pun:
 * katalog datang dari registry, aksi dikirim sebagai perintah teks. Semua
 * teks untuk pemain datang dalam dua bahasa (`Localized`), begitu juga pesan
 * kesalahan yang dilempar perintah.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Localized } from './i18n.svelte';

export type ParamKind =
	| { type: 'int'; min: number; max: number; step: number }
	| { type: 'choice'; options: string[] };

export type ActionSpec =
	| { kind: 'fixed'; command: string }
	| { kind: 'template'; verb: string; params: ({ name: string } & ParamKind)[] };

export interface CommandDoc {
	pola: string;
	ringkas: Localized;
}

/** Manifest cartridge (SPEC §5.2); kunci mengikuti SPEC. */
export interface Game {
	id: string;
	nama: Localized;
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
	rtp_line: Localized | null;
}

export interface Category {
	key: string;
	label: Localized;
	games: Game[];
}

export interface Step {
	teks: Localized;
	sebelum: string[];
	aksi: string | null;
	sorot: string[];
	petunjuk: Localized | null;
}

export type Feedback = { kind: 'correct' } | { kind: 'wrong'; hint: Localized };

export interface ActionView {
	spec: ActionSpec;
	usage: string;
	concrete: string[] | null;
}

export interface TutorialState {
	game: string;
	title: Localized;
	index: number;
	total: number;
	finished: boolean;
	step: Step | null;
	view_text: Localized;
	actions: ActionView[];
	feedback: Feedback | null;
}

export interface AppInfo {
	name: string;
	version: string;
	/** Folder data aplikasi; `null` di luar Tauri. */
	data_dir: string | null;
	/**
	 * Data profil untuk sapaan boot (SPEC §4). Belum dikirim backend; datang
	 * di M3 (nama, game terakhir) dan M4 (chip). Sapaan yang membutuhkannya
	 * aktif otomatis begitu ada.
	 */
	profile?: { name?: string; last_game?: string; chips?: number };
}

type Api = {
	app_info(): Promise<AppInfo>;
	catalog(): Promise<Category[]>;
	man(id: string): Promise<Localized>;
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
