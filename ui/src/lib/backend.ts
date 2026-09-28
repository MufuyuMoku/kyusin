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
	/** Jumlah level bot (0 = belum ada). */
	bot_levels: number;
	/** Perkiraan rating tiap level dari kalibrasi, bila ada. */
	bot_ratings: (number | null)[];
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
	/** Data tampilan untuk kontrol visual game. */
	view_data: unknown;
	actions: ActionView[];
	feedback: Feedback | null;
}

export type SeatKind = { kind: 'human' } | { kind: 'bot'; level: number } | { kind: 'remote' };

export interface GameResult {
	winners: number[];
	scores: number[];
	summary: Localized;
}

export interface Move {
	seat: number;
	command: string;
}

/** Catatan provably fair (SPEC §5.4). */
export interface FairRecord {
	host: string;
	commitments: Record<string, string>;
	seeds: Record<string, string>;
	excluded: string[];
	round_seed: string;
}

export type VerifyStep = 'commitments' | 'round_seed' | 'moves' | 'result' | 'state_hash';

export interface VerifyReport {
	ok: boolean;
	checks: { step: VerifyStep; ok: boolean }[];
	error: Localized | null;
}

export interface MatchState {
	game: string;
	seat: number;
	seats: SeatKind[];
	your_turn: boolean;
	bot_turn: boolean;
	over: boolean;
	view_data: unknown;
	view_text: Localized;
	actions: ActionView[];
	moves: Move[];
	/** Komitmen yang diumumkan sebelum ronde. */
	commitments: Record<string, string>;
	/** Seed yang dibuka; hanya setelah selesai. */
	reveal: FairRecord | null;
	result: GameResult | null;
	verify: VerifyReport | null;
	replay_id: number | null;
	save_error: string | null;
	/** Jam (catur), dihitung host. */
	clock: ClockState | null;
}

export interface ClockState {
	remaining_ms: [number, number];
	running: number | null;
	increment_ms: number;
}

export interface ReplaySummary {
	id: number;
	game: string;
	started_at: number;
	finished: boolean;
	moves: number;
	seats: SeatKind[];
	result: GameResult | null;
}

export interface Frame {
	index: number;
	last: Move | null;
	view_data: unknown;
	view_text: Localized;
}

export interface ReplayData {
	/** `null` untuk partai impor PGN (tidak tersimpan). */
	id: number | null;
	game: string;
	seat: number;
	seats: SeatKind[];
	frames: Frame[];
	/** Tidak ada untuk impor PGN. */
	fair: FairRecord | null;
	result: GameResult | null;
	verify: VerifyReport | null;
	/** Tag PGN (impor). */
	tags: [string, string][];
}

export interface AppInfo {
	name: string;
	version: string;
	/** Folder data aplikasi; `null` di luar Tauri. */
	data_dir: string | null;
	/** Berkas basis data dan apakah berhasil dibuka. */
	database: string | null;
	database_ok: boolean;
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
	match_start(
		id: string,
		level: number,
		seat: number,
		playerSeed: string | null,
		clock: { minutes: number; increment: number } | null
	): Promise<MatchState>;
	match_flag(): Promise<MatchState>;
	replay_pgn(id: number, lang: 'id' | 'en'): Promise<string>;
	pgn_open(text: string): Promise<ReplayData>;
	match_act(command: string): Promise<MatchState>;
	match_step(): Promise<MatchState>;
	match_leave(): Promise<void>;
	replay_list(game: string | null): Promise<ReplaySummary[]>;
	replay_open(id: number): Promise<ReplayData>;
};

const tauriApi: Api = {
	app_info: () => invoke('app_info'),
	catalog: () => invoke('catalog'),
	man: (id) => invoke('man', { id }),
	tutorial_start: (id) => invoke('tutorial_start', { id }),
	tutorial_act: (command) => invoke('tutorial_act', { command }),
	tutorial_next: () => invoke('tutorial_next'),
	tutorial_stop: () => invoke('tutorial_stop'),
	match_start: (id, level, seat, playerSeed, clock) =>
		invoke('match_start', { id, level, seat, playerSeed, clock }),
	match_flag: () => invoke('match_flag'),
	replay_pgn: (id, lang) => invoke('replay_pgn', { id, lang }),
	pgn_open: (text) => invoke('pgn_open', { text }),
	match_act: (command) => invoke('match_act', { command }),
	match_step: () => invoke('match_step'),
	match_leave: () => invoke('match_leave'),
	replay_list: (game) => invoke('replay_list', { game }),
	replay_open: (id) => invoke('replay_open', { id })
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
