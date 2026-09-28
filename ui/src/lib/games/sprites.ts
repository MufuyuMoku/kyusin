/**
 * Peta piksel sprite game (12×12 kecuali disebut lain). `X` = piksel
 * menyala. Bidak terang diisi penuh, bidak gelap berupa cincin, supaya
 * keduanya jelas berbeda dari petak kosong di ketiga tema fosfor (SPEC §4).
 */

export const DISC_LIGHT = [
	'....XXXX....',
	'..XXXXXXXX..',
	'.XXXXXXXXXX.',
	'.XXXXXXXXXX.',
	'XXXXXXXXXXXX',
	'XXXXXXXXXXXX',
	'XXXXXXXXXXXX',
	'XXXXXXXXXXXX',
	'.XXXXXXXXXX.',
	'.XXXXXXXXXX.',
	'..XXXXXXXX..',
	'....XXXX....'
] as const;

export const DISC_DARK = [
	'....XXXX....',
	'..XXXXXXXX..',
	'.XXX....XXX.',
	'.XX......XX.',
	'XX........XX',
	'XX........XX',
	'XX........XX',
	'XX........XX',
	'.XX......XX.',
	'.XXX....XXX.',
	'..XXXXXXXX..',
	'....XXXX....'
] as const;

/** Penanda langkah sah: titik 4×4. */
export const DOT = ['.XX.', 'XXXX', 'XXXX', '.XX.'] as const;

/**
 * Bidak catur (12×12, diisi penuh). Versi bergaris tepi untuk hitam
 * dibangkitkan otomatis dengan `outline()`, jadi keduanya selalu konsisten
 * (D-041, D-043).
 */
export const CHESS = {
	k: [
		'.....XX.....',
		'....XXXX....',
		'.....XX.....',
		'..XX.XX.XX..',
		'.XXXXXXXXXX.',
		'.XXXXXXXXXX.',
		'..XXXXXXXX..',
		'...XXXXXX...',
		'...XXXXXX...',
		'..XXXXXXXX..',
		'.XXXXXXXXXX.',
		'............'
	],
	q: [
		'X....XX....X',
		'XX...XX...XX',
		'XXX.XXXX.XXX',
		'.XXXXXXXXXX.',
		'.XXXXXXXXXX.',
		'..XXXXXXXX..',
		'...XXXXXX...',
		'...XXXXXX...',
		'...XXXXXX...',
		'..XXXXXXXX..',
		'.XXXXXXXXXX.',
		'............'
	],
	r: [
		'............',
		'.XX.XXXX.XX.',
		'.XXXXXXXXXX.',
		'..XXXXXXXX..',
		'...XXXXXX...',
		'...XXXXXX...',
		'...XXXXXX...',
		'...XXXXXX...',
		'..XXXXXXXX..',
		'.XXXXXXXXXX.',
		'.XXXXXXXXXX.',
		'............'
	],
	b: [
		'.....XX.....',
		'....XXXX....',
		'...XXX.XX...',
		'...XX.XXX...',
		'...XXXXXX...',
		'....XXXX....',
		'....XXXX....',
		'.....XX.....',
		'....XXXX....',
		'..XXXXXXXX..',
		'.XXXXXXXXXX.',
		'............'
	],
	n: [
		'............',
		'.....XX.....',
		'....XXXXX...',
		'...XXXXXXX..',
		'..XXX.XXXXX.',
		'.XXXXXXXXXX.',
		'.XXX..XXXXX.',
		'......XXXXX.',
		'.....XXXXX..',
		'....XXXXXX..',
		'...XXXXXXXX.',
		'............'
	],
	p: [
		'............',
		'............',
		'.....XX.....',
		'....XXXX....',
		'....XXXX....',
		'.....XX.....',
		'....XXXX....',
		'.....XX.....',
		'....XXXX....',
		'...XXXXXX...',
		'..XXXXXXXX..',
		'............'
	]
} as const;

export type ChessPiece = keyof typeof CHESS;

/** Piksel tepi saja: piksel menyala yang bertetangga (4 arah) dengan kosong. */
export function outline(pixels: readonly string[]): string[] {
	const on = (x: number, y: number) => pixels[y]?.[x] === 'X';
	return pixels.map((row, y) =>
		row
			.split('')
			.map((ch, x) =>
				ch === 'X' && (!on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1)) ? 'X' : '.'
			)
			.join('')
	);
}
