/**
 * Casino lotere dan instan M6b (SPEC §6.5): bentuk view Keno, Bingo, kartu
 * gosok, Hi-Lo, dan Video Poker, serta format angka bersama. Aturan dan
 * bayaran tetap di Rust; UI hanya membaca view dan mengirim perintah teks.
 */

import { lang } from '$lib/i18n.svelte';
import type { MejaView } from './meja';

export type V = MejaView & Record<string, any>;

/** Pengali `value / scale` sebagai teks bahasa aktif, misalnya `3,8×`. */
export function times(value: number, scale = 1): string {
	return `${new Intl.NumberFormat(lang(), { maximumFractionDigits: 2 }).format(value / scale)}×`;
}

/** Angka dengan pemisah ribuan bahasa aktif. */
export function num(n: number): string {
	return new Intl.NumberFormat(lang()).format(n);
}

export const signed = (n: number) => (n > 0 ? `+${n}` : String(n));

/** Kolom bingo (B I N G O) dan rentang angkanya. */
export const BINGO_COLUMNS = ['B', 'I', 'N', 'G', 'O'];

/** Posisi yang ditahan (1–5) untuk perintah `draw`. */
export function drawCommand(held: boolean[]): string {
	const pos = held.flatMap((h, i) => (h ? [String(i + 1)] : []));
	return pos.length ? `draw ${pos.join(' ')}` : 'draw';
}

/** Posisi (0–4) yang disorot tutorial dari `draw 1 2`. */
export function litPositions(highlight: Set<string>): Set<number> {
	const out = new Set<number>();
	for (const h of highlight) {
		const m = /^draw((?: [1-5])*)$/.exec(h);
		if (m) for (const p of m[1].trim().split(' ').filter(Boolean)) out.add(Number(p) - 1);
	}
	return out;
}

/** Jumlah taruhan dari sorotan tutorial (`bet 10`). */
export function litAmount(highlight: Set<string>): number | null {
	for (const h of highlight) {
		const m = /^bet (\d+)$/.exec(h);
		if (m) return Number(m[1]);
	}
	return null;
}
