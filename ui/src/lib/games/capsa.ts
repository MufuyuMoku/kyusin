/**
 * Meja Capsa Susun M5b-2: bentuk `view_data` (lihat
 * `crates/games/src/capsa_susun`) dan susunan yang sedang disusun di UI.
 * Aturan (sah atau tidak) tetap di Rust: UI hanya memastikan 3/5/5 kartu
 * lalu mengirim `arrange`; susunan yang salah ditolak mesin.
 */

export interface CapsaSeat {
	tumpukan: number;
	awal: number;
	status: string;
	kartu: string[];
	baris: string[][] | null;
	nama_baris: string[] | null;
	istimewa: string | null;
	siap: boolean;
	poin: number;
	menang: number;
}

export interface CapsaView {
	fase: 'susun' | 'antara' | 'selesai';
	kamu: number;
	kursi: CapsaSeat[];
	dealer: number;
	tangan_ke: number;
	poin_chip: number;
	saran: string[] | null;
	bersih: number;
	taruhan_meja: number;
	alasan: string | null;
}

export function isCapsaView(v: unknown): v is CapsaView {
	return !!v && typeof v === 'object' && Array.isArray((v as CapsaView).kursi) && 'poin_chip' in (v as object);
}

/** Ukuran baris: depan, tengah, belakang. */
export const ROW_SIZES = [3, 5, 5] as const;

/** Geser kartu bertumpuk: lawan (rapat) dan milikmu (lebih lebar supaya mudah dipilih). */
export const PEEK = 18;
export const PICK = 30;
/** Jarak antarbaris dalam satu deret. */
export const ROW_GAP = 12;

/** Posisi kiri kartu ke-`j` di baris `row` untuk geser `step`. */
export function slotLeft(row: number, j: number, step: number, cardW = 44): number {
	let x = 0;
	for (let r = 0; r < row; r++) x += (ROW_SIZES[r] - 1) * step + cardW + ROW_GAP;
	return x + j * step;
}

/** Lebar seluruh deret tiga baris. */
export function rowsWidth(step: number, cardW = 44): number {
	return slotLeft(2, ROW_SIZES[2] - 1, step, cardW) + cardW;
}

/** Tiga baris dari 13 kartu berurutan (depan 3, tengah 5, belakang 5). */
export function splitRows(cards: string[]): string[][] {
	return [cards.slice(0, 3), cards.slice(3, 8), cards.slice(8, 13)];
}

/** Perintah `arrange` dari tiga baris yang penuh, atau `null`. */
export function arrangeCommand(rows: string[][]): string | null {
	if (rows.some((r, i) => r.length !== ROW_SIZES[i])) return null;
	return `arrange ${rows.flat().join(' ')}`;
}
