/**
 * Sprite piksel dadu (SPEC §4; M6a): bingkai 15×15 dengan bulatan tanda
 * tambah 3×3 seperti domino, jadi dadu dan ubin terlihat sekeluarga.
 */

export const DIE = 15;

const PIP = ['.X.', 'XXX', '.X.'];
/** Kiri/atas bulatan di kolom/baris 0, 1, 2. */
const AT = [2, 6, 10];

const LAYOUT: Record<number, [number, number][]> = {
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

/** Peta piksel dadu bernilai `n` (1–6); nilai lain = bingkai kosong. */
export function diePixels(n: number): string[] {
	const g = Array.from({ length: DIE }, () => Array.from({ length: DIE }, () => '.'));
	for (let i = 1; i < DIE - 1; i++) {
		g[0][i] = 'X';
		g[DIE - 1][i] = 'X';
		g[i][0] = 'X';
		g[i][DIE - 1] = 'X';
	}
	for (const [c, r] of LAYOUT[n] ?? []) {
		PIP.forEach((row, dy) => {
			for (let dx = 0; dx < row.length; dx++) if (row[dx] === 'X') g[AT[r] + dy][AT[c] + dx] = 'X';
		});
	}
	return g.map((r) => r.join(''));
}

/** Jumlah bulatan yang tergambar (untuk tes): lima piksel per bulatan. */
export function pipCount(pixels: string[]): number {
	const inner = pixels.slice(1, -1).map((r) => r.slice(1, -1));
	return inner.join('').split('').filter((c) => c === 'X').length / 5;
}
