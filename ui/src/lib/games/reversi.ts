/**
 * Bentuk `view_data` Reversi (lihat `crates/games/src/reversi/mod.rs`).
 * UI hanya membaca data ini; aturan tetap di Rust.
 */

export interface ReversiView {
	/** 8 baris: `.` kosong, `X` hitam, `O` putih. */
	papan: string[];
	giliran: number | null;
	kamu: number;
	legal: string[];
	hitam: number;
	putih: number;
	terakhir: string | null;
	dibalik: string[];
	selesai: boolean;
	pemenang: number[] | null;
	/** Selesai karena menyerah. */
	menyerah: boolean;
}

export const COLS = 'abcdefgh';

export function squareName(row: number, col: number): string {
	return `${COLS[col]}${row + 1}`;
}

export function isReversiView(v: unknown): v is ReversiView {
	return !!v && typeof v === 'object' && Array.isArray((v as ReversiView).papan);
}
