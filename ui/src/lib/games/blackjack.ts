/**
 * Bentuk `view_data` Blackjack (lihat `crates/games/src/blackjack/mod.rs`).
 * UI hanya membaca data ini; aturan tetap di Rust.
 */

export interface BlackjackHand {
	kartu: string[];
	taruhan: number;
	nilai: number;
	lunak: boolean;
	ganda: boolean;
	blackjack: boolean;
	selesai: boolean;
	hasil: 'menang' | 'kalah' | 'seri' | 'menyerah' | null;
	bayar: number | null;
}

export interface BlackjackView {
	fase: 'taruhan' | 'asuransi' | 'giliran' | 'selesai';
	bandar: string[];
	bandar_nilai: number | null;
	tangan: BlackjackHand[];
	aktif: number | null;
	asuransi: number | null;
	asuransi_bayar: number | null;
	bersih: number;
	taruhan_meja: number;
	biaya: Record<string, number>;
	ronde: number;
	kartu_terpakai: number;
	sisa: number;
	potong: number;
	min_taruhan: number;
	maks_taruhan: number;
	selesai: boolean;
	alasan: string | null;
}

export function isBlackjackView(v: unknown): v is BlackjackView {
	return !!v && typeof v === 'object' && Array.isArray((v as BlackjackView).tangan) && 'bandar' in (v as object);
}

/** Pilihan chip untuk menyusun taruhan. */
export const CHIP_STEPS = [10, 50, 100, 500] as const;

/** Jarak geser kartu bertumpuk dalam satu tangan (px). */
export const CARD_OFFSET = 18;

/** Hasil bersih ronde terakhir (tangan + insurance), bila sudah selesai. */
export function roundNet(v: BlackjackView): number | null {
	if (v.tangan.length === 0 || v.tangan.some((h) => h.bayar === null)) return null;
	return v.tangan.reduce((s, h) => s + (h.bayar ?? 0), 0) + (v.asuransi_bayar ?? 0);
}
