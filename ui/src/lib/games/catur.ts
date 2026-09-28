/**
 * Bentuk `view_data` catur (lihat `crates/games/src/catur/mod.rs`).
 * UI hanya membaca data ini; aturan tetap di Rust.
 */

export interface ChessMove {
	dari: string;
	ke: string;
	promosi: string | null;
	san: string;
}

export interface ChessLast {
	dari: string;
	ke: string;
	san: string;
	promosi: string | null;
	rokade: 'pendek' | 'panjang' | null;
	en_passant: boolean;
}

export interface ChessView {
	/** Baris 8 → 1; huruf FEN (besar = putih), `.` kosong. */
	papan: string[];
	giliran: number | null;
	kamu: number;
	langkah: ChessMove[];
	terakhir: ChessLast | null;
	skak: boolean;
	raja_skak: string | null;
	riwayat: string[];
	selesai: boolean;
	pemenang: number[] | null;
	alasan: string | null;
	jam: { menit: number; tambahan_detik: number } | null;
	fen: string;
}

export const FILES = 'abcdefgh';

export function isChessView(v: unknown): v is ChessView {
	return !!v && typeof v === 'object' && Array.isArray((v as ChessView).langkah);
}

/** Huruf bidak di petak (`.` bila kosong). */
export function pieceAt(v: ChessView, sq: string): string {
	const file = FILES.indexOf(sq[0]);
	const rank = Number(sq[1]);
	return v.papan[8 - rank]?.[file] ?? '.';
}

/** Petak di baris/kolom tampilan; `flip` = sudut pandang hitam. */
export function squareAt(row: number, col: number, flip: boolean): string {
	return flip ? `${FILES[7 - col]}${row + 1}` : `${FILES[col]}${8 - row}`;
}

export const PROMOTIONS = ['q', 'r', 'b', 'n'] as const;
