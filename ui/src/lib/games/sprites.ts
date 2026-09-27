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
