/**
 * Kontrol visual per game. Game tanpa entri di sini memakai kontrol generik
 * dari `ActionSpec` (tombol perintah). Menambah game = menambah satu entri;
 * menu dan layar lain tidak berubah (SPEC §5.2).
 */

import type { Component } from 'svelte';
import type { Key } from '$lib/i18n.svelte';
import BlackjackStatus from './BlackjackStatus.svelte';
import CapsaStatus from './CapsaStatus.svelte';
import CapsaTable from './CapsaTable.svelte';
import BlackjackTable from './BlackjackTable.svelte';
import ChessBoard from './ChessBoard.svelte';
import ChessStatus from './ChessStatus.svelte';
import MejaStatus from './MejaStatus.svelte';
import MejaTable from './MejaTable.svelte';
import PokerStatus from './PokerStatus.svelte';
import PokerTable from './PokerTable.svelte';
import ReversiBoard from './ReversiBoard.svelte';
import ReversiStatus from './ReversiStatus.svelte';

export interface BoardProps {
	view: unknown;
	/** Id game (meja casino bersama memilih modelnya dari sini). */
	game?: string;
	/** Meja satu ronde per sesi: ronde selesai, taruhan memulai ronde baru. */
	nextRound?: boolean;
	interactive?: boolean;
	highlight?: Set<string>;
	onplay?: (command: string) => void;
	/** Bentuk perintah yang sah sekarang (`usage`), untuk kontrol meja. */
	actions?: string[];
	/** Saldo chip profil (game casino); `null` di tutorial dan replay. */
	chips?: number | null;
}

export interface StatusProps {
	view: unknown;
	botTurn?: boolean;
	/** Penonton (replay): tanpa kalimat giliranmu. */
	observer?: boolean;
}

/** Jam: menit per pemain + tambahan detik per langkah. */
export interface ClockOption {
	minutes: number;
	increment: number;
}

export interface GameUi {
	board: Component<BoardProps>;
	status: Component<StatusProps>;
	/** Nama kursi untuk pilihan "main sebagai". */
	seats: Key[];
	/** Pilihan jam (`null` = tanpa jam); kosong = game tanpa jam. */
	clocks?: (ClockOption | null)[];
	/** Game ini punya PGN (ekspor dari replay, impor ke penampil). */
	pgn?: boolean;
	/** Casino satu ronde per sesi (commit-reveal per ronde, M5a). */
	perRound?: boolean;
}

const meja = (perRound: boolean): GameUi => ({ board: MejaTable, status: MejaStatus, seats: [], perRound });

export const GAME_UI: Record<string, GameUi> = {
	catur: {
		board: ChessBoard,
		status: ChessStatus,
		seats: ['catur.white_first', 'catur.black_second'],
		clocks: [null, { minutes: 5, increment: 0 }, { minutes: 10, increment: 5 }, { minutes: 15, increment: 10 }],
		pgn: true
	},
	reversi: {
		board: ReversiBoard,
		status: ReversiStatus,
		seats: ['reversi.black_first', 'reversi.white_second']
	},
	blackjack: {
		board: BlackjackTable,
		status: BlackjackStatus,
		seats: []
	},
	baccarat: meja(false),
	'dragon-tiger': meja(false),
	'casino-war': meja(false),
	'red-dog': meja(false),
	'andar-bahar': meja(true),
	'three-card-poker': meja(true),
	'caribbean-stud': meja(true),
	'casino-holdem': meja(true),
	'let-it-ride': meja(true),
	'pai-gow': meja(true),
	'texas-holdem': { board: PokerTable, status: PokerStatus, seats: [] },
	omaha: { board: PokerTable, status: PokerStatus, seats: [] },
	'teen-patti': { board: PokerTable, status: PokerStatus, seats: [] },
	'domino-qiuqiu': { board: PokerTable, status: PokerStatus, seats: [] },
	'capsa-susun': { board: CapsaTable, status: CapsaStatus, seats: [] }
};
