/**
 * Meja casino antar-pemain M5b (Texas Hold'em, Omaha, Teen Patti, Domino
 * QiuQiu): bentuk
 * `view_data` (lihat `crates/games/src/poker_meja` dan `teen_patti`) dan
 * bantuan untuk kontrol taruhan. Aturan tetap di Rust.
 */

export interface SeatView {
	tumpukan: number;
	awal: number;
	taruhan?: number;
	taruhan_tangan: number;
	status: string;
	kartu: string[];
	tangan: string | null;
	siap: boolean;
	menang: number;
	terlihat?: boolean;
	buta_ke?: number;
	// Domino QiuQiu
	nilai?: [number, number] | null;
}

export interface TableView {
	fase: 'main' | 'antara' | 'selesai';
	kamu: number;
	kursi: SeatView[];
	pot: number;
	dealer: number;
	giliran: number | null;
	tangan_ke: number;
	log: [number, string][];
	bersih: number;
	taruhan_meja: number;
	biaya: Record<string, number>;
	alasan: string | null;
	// Poker
	meja_kartu?: string[];
	sb?: number;
	bb?: number;
	blind?: [number, number];
	panggil?: number;
	naik_min?: number | null;
	naik_maks?: number | null;
	pot_limit?: boolean;
	// Teen Patti
	stake?: number;
	boot?: number;
	sideshow?: [number, number] | null;
	alasan_tangan?: string | null;
	// Domino QiuQiu
	ante?: number;
	putaran?: number;
}

export function isTableView(v: unknown): v is TableView {
	return !!v && typeof v === 'object' && Array.isArray((v as TableView).kursi) && 'kamu' in (v as object);
}

/** Jumlah kartu tangan per kursi untuk tiap game. */
export const HOLE: Record<string, number> = { 'texas-holdem': 2, omaha: 4, 'teen-patti': 3, 'domino-qiuqiu': 4 };

/** Total taruhan saat ini (taruhan tertinggi di babak berjalan). */
export function currentBet(v: TableView): number {
	return Math.max(0, ...v.kursi.map((k) => k.taruhan ?? 0));
}

/** Pilihan cepat jumlah bet/raise (total di babak ini), dibatasi min/maks. */
export function presets(v: TableView): { key: 'min' | 'half' | 'pot' | 'max'; amount: number }[] {
	const min = v.naik_min ?? 0;
	const max = v.naik_maks ?? 0;
	if (!max) return [];
	const cur = currentBet(v);
	const call = v.panggil ?? 0;
	const clamp = (n: number) => Math.max(min, Math.min(max, Math.round(n)));
	return [
		{ key: 'min', amount: min },
		{ key: 'half', amount: clamp(cur + (v.pot + call) / 2) },
		{ key: 'pot', amount: clamp(cur + v.pot + call) },
		{ key: 'max', amount: max }
	];
}

/** Lebar geser kartu bertumpuk (px) dan kartu meja yang terpisah. */
export const OFFSET = 18;
export const BOARD_OFFSET = 48;
