/**
 * Sprite piksel kartu remi (SPEC §4, D-056): peringkat, jenis (♠♥♦♣), dan
 * bingkai digambar sebagai peta piksel, bukan glyph font. Satu kartu =
 * peta 22×30 (`X` = menyala), dirender oleh `PixelSprite`.
 *
 * Notasi kartu sama dengan mesin: peringkat `A 2–9 T J Q K` + jenis
 * `s h d c`, misalnya `Ah`, `Td`. `??` = kartu tertutup.
 */

export const CARD_W = 22;
export const CARD_H = 30;

/** Peringkat 5×7. */
export const RANKS: Record<string, readonly string[]> = {
	A: ['.XXX.', 'X...X', 'X...X', 'XXXXX', 'X...X', 'X...X', 'X...X'],
	'2': ['.XXX.', 'X...X', '....X', '...X.', '..X..', '.X...', 'XXXXX'],
	'3': ['XXXX.', '....X', '....X', '.XXX.', '....X', '....X', 'XXXX.'],
	'4': ['...X.', '..XX.', '.X.X.', 'X..X.', 'XXXXX', '...X.', '...X.'],
	'5': ['XXXXX', 'X....', 'XXXX.', '....X', '....X', 'X...X', '.XXX.'],
	'6': ['.XXX.', 'X....', 'X....', 'XXXX.', 'X...X', 'X...X', '.XXX.'],
	'7': ['XXXXX', '....X', '...X.', '..X..', '.X...', '.X...', '.X...'],
	'8': ['.XXX.', 'X...X', 'X...X', '.XXX.', 'X...X', 'X...X', '.XXX.'],
	'9': ['.XXX.', 'X...X', 'X...X', '.XXXX', '....X', '....X', '.XXX.'],
	T: ['X.XXX', 'X.X.X', 'X.X.X', 'X.X.X', 'X.X.X', 'X.X.X', 'X.XXX'],
	J: ['..XXX', '...X.', '...X.', '...X.', 'X..X.', 'X..X.', '.XX..'],
	Q: ['.XXX.', 'X...X', 'X...X', 'X...X', 'X.X.X', 'X..X.', '.XX.X'],
	K: ['X...X', 'X..X.', 'X.X..', 'XX...', 'X.X..', 'X..X.', 'X...X']
};

/** Jenis kecil 5×5 (di bawah peringkat). */
export const SUITS_SMALL: Record<string, readonly string[]> = {
	s: ['..X..', '.XXX.', 'XXXXX', '..X..', '.XXX.'],
	h: ['.X.X.', 'XXXXX', 'XXXXX', '.XXX.', '..X..'],
	d: ['..X..', '.XXX.', 'XXXXX', '.XXX.', '..X..'],
	c: ['..X..', 'X.X.X', 'XXXXX', '..X..', '.XXX.']
};

/** Jenis besar 9×9 (tengah kartu). */
export const SUITS_BIG: Record<string, readonly string[]> = {
	s: [
		'....X....',
		'...XXX...',
		'..XXXXX..',
		'.XXXXXXX.',
		'XXXXXXXXX',
		'XXXXXXXXX',
		'.XX.X.XX.',
		'....X....',
		'...XXX...'
	],
	h: [
		'.XX...XX.',
		'XXXX.XXXX',
		'XXXXXXXXX',
		'XXXXXXXXX',
		'.XXXXXXX.',
		'..XXXXX..',
		'...XXX...',
		'....X....',
		'.........'
	],
	d: [
		'....X....',
		'...XXX...',
		'..XXXXX..',
		'.XXXXXXX.',
		'XXXXXXXXX',
		'.XXXXXXX.',
		'..XXXXX..',
		'...XXX...',
		'....X....'
	],
	c: [
		'...XXX...',
		'..XXXXX..',
		'..XXXXX..',
		'XX.XXX.XX',
		'XXXXXXXXX',
		'XXXXXXXXX',
		'XX..X..XX',
		'....X....',
		'...XXX...'
	]
};

function blank(): string[][] {
	return Array.from({ length: CARD_H }, () => Array.from({ length: CARD_W }, () => '.'));
}

/** Bingkai dengan sudut terpotong. */
function frame(g: string[][]) {
	for (let x = 1; x < CARD_W - 1; x++) {
		g[0][x] = 'X';
		g[CARD_H - 1][x] = 'X';
	}
	for (let y = 1; y < CARD_H - 1; y++) {
		g[y][0] = 'X';
		g[y][CARD_W - 1] = 'X';
	}
}

function stamp(g: string[][], pixels: readonly string[], x0: number, y0: number) {
	pixels.forEach((row, y) => {
		for (let x = 0; x < row.length; x++) if (row[x] === 'X') g[y0 + y][x0 + x] = 'X';
	});
}

/** Kartu tertutup: bingkai + arsir diagonal. */
export function backPixels(): string[] {
	const g = blank();
	frame(g);
	for (let y = 2; y < CARD_H - 2; y++)
		for (let x = 2; x < CARD_W - 2; x++) if ((x + y) % 4 === 0 || (x - y + 64) % 4 === 0) g[y][x] = 'X';
	return g.map((r) => r.join(''));
}

/** Bintang joker 9×9 (Pai Gow Poker). */
export const JOKER_BIG: readonly string[] = [
	'....X....',
	'....X....',
	'...XXX...',
	'XXXXXXXXX',
	'.XXXXXXX.',
	'..XXXXX..',
	'.XXX.XXX.',
	'.XX...XX.',
	'X.......X'
];

/** Peta piksel sebuah kartu (`Ah`, `Td`, …), joker `JK`, atau `??` untuk tertutup. */
export function cardPixels(card: string): string[] {
	if (card === 'JK') {
		const g = blank();
		frame(g);
		stamp(g, RANKS.J, 2, 2);
		stamp(g, RANKS.K, 8, 2);
		stamp(g, JOKER_BIG, 11, 18);
		return g.map((r) => r.join(''));
	}
	const rank = RANKS[card[0]];
	const small = SUITS_SMALL[card[1]];
	const big = SUITS_BIG[card[1]];
	if (!rank || !small || !big || card.length !== 2) return backPixels();
	const g = blank();
	frame(g);
	stamp(g, rank, 2, 2);
	stamp(g, small, 2, 10);
	stamp(g, big, 11, 18);
	return g.map((r) => r.join(''));
}

export const SUIT_KEYS = ['s', 'h', 'd', 'c'] as const;
export const RANK_KEYS = ['A', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K'] as const;
