/**
 * Sprite piksel kartu domino (SPEC §4; M5b-2): bingkai, garis tengah, dan
 * bulatan digambar sebagai peta piksel seukuran kartu remi (22×30), jadi
 * meja antar-pemain memakai slot yang sama. Notasi sama dengan mesin:
 * `besar-kecil`, misalnya `6-4`; `??` = tertutup.
 */

import { CARD_H, CARD_W, backPixels } from './cards.ts';

/** Satu bulatan: tanda tambah 3×3 (5 piksel). */
const PIP = ['.X.', 'XXX', '.X.'];

/** Kolom dan baris bulatan dalam satu paruh (x kiri, y atas). */
const COLS = [4, 9, 14];
const ROWS = [0, 4, 8];

/** Letak bulatan untuk 0–6, sebagai (kolom, baris). */
const LAYOUT: Record<number, [number, number][]> = {
	0: [],
	1: [[1, 1]],
	2: [
		[0, 0],
		[2, 2]
	],
	3: [
		[0, 0],
		[1, 1],
		[2, 2]
	],
	4: [
		[0, 0],
		[2, 0],
		[0, 2],
		[2, 2]
	],
	5: [
		[0, 0],
		[2, 0],
		[1, 1],
		[0, 2],
		[2, 2]
	],
	6: [
		[0, 0],
		[2, 0],
		[0, 1],
		[2, 1],
		[0, 2],
		[2, 2]
	]
};

/** Baris atas tiap paruh. */
const HALF_TOP = [2, 17];
/** Baris garis tengah. */
const DIVIDER = 14;

export const PIP_PIXELS = 5;

export function parseTile(tile: string): [number, number] | null {
	const m = /^([0-6])-([0-6])$/.exec(tile);
	if (!m) return null;
	const a = Number(m[1]);
	const b = Number(m[2]);
	return [Math.max(a, b), Math.min(a, b)];
}

/** Peta piksel sebuah kartu domino (`6-4`), atau tertutup untuk `??`. */
export function dominoPixels(tile: string): string[] {
	const t = parseTile(tile);
	if (!t) return backPixels();
	const g = Array.from({ length: CARD_H }, () => Array.from({ length: CARD_W }, () => '.'));
	for (let x = 1; x < CARD_W - 1; x++) {
		g[0][x] = 'X';
		g[CARD_H - 1][x] = 'X';
	}
	for (let y = 1; y < CARD_H - 1; y++) {
		g[y][0] = 'X';
		g[y][CARD_W - 1] = 'X';
	}
	for (let x = 3; x < CARD_W - 3; x++) g[DIVIDER][x] = 'X';
	t.forEach((n, half) => {
		for (const [c, r] of LAYOUT[n]) {
			PIP.forEach((row, dy) => {
				for (let dx = 0; dx < row.length; dx++)
					if (row[dx] === 'X') g[HALF_TOP[half] + ROWS[r] + dy][COLS[c] + dx] = 'X';
			});
		}
	});
	return g.map((r) => r.join(''));
}
