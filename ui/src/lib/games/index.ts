/**
 * Kontrol visual per game. Game tanpa entri di sini memakai kontrol generik
 * dari `ActionSpec` (tombol perintah). Menambah game = menambah satu entri;
 * menu dan layar lain tidak berubah (SPEC §5.2).
 */

import type { Component } from 'svelte';
import type { Key } from '$lib/i18n.svelte';
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

export interface GameUi {
	board: Component<BoardProps>;
	status: Component<StatusProps>;
	/** Nama kursi untuk pilihan "main sebagai". */
	seats: Key[];
}

export const GAME_UI: Record<string, GameUi> = {
	reversi: {
		board: ReversiBoard,
		status: ReversiStatus,
		seats: ['reversi.black_first', 'reversi.white_second']
	}
};
