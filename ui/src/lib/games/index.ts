/**
 * Kontrol visual per game. Game tanpa entri di sini memakai kontrol generik
 * dari `ActionSpec` (tombol perintah). Menambah game = menambah satu entri;
 * menu dan layar lain tidak berubah (SPEC §5.2).
 */

import type { Component } from 'svelte';
import type { Key } from '$lib/i18n.svelte';
import ChessBoard from './ChessBoard.svelte';
import ChessStatus from './ChessStatus.svelte';
import ReversiBoard from './ReversiBoard.svelte';
import ReversiStatus from './ReversiStatus.svelte';

export interface BoardProps {
	view: unknown;
	interactive?: boolean;
	highlight?: Set<string>;
	onplay?: (command: string) => void;
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
}

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
	}
};
