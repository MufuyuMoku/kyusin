/**
 * Tata letak dan tempat taruhan roulette (M6a), tanpa teks: daftar tempat
 * sama dengan `roulette::spots` di Rust, dan pilihan angka mode GABUNG
 * diterjemahkan ke tempat taruhan yang mencakup tepat angka itu.
 */

export const range = (a: number, b: number) => Array.from({ length: b - a + 1 }, (_, i) => a + i);

export const RED = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];

/** Urutan kantong untuk nama tempat: 0, 00, lalu 1–36. */
const order = (p: string) => (p === '0' ? -2 : p === '00' ? -1 : Number(p));

/** Semua tempat roulette, sama dengan `roulette::spots` di Rust. */
export function rouletteSpots(american: boolean): string[] {
	const v: string[] = ['straight-0'];
	if (american) v.push('straight-00');
	for (const n of range(1, 36)) v.push(`straight-${n}`);
	for (const s of american ? ['0-00', '0-1', '0-2', '00-2', '00-3'] : ['0-1', '0-2', '0-3']) v.push(`split-${s}`);
	for (const n of range(1, 35)) if (n % 3 !== 0) v.push(`split-${n}-${n + 1}`);
	for (const n of range(1, 33)) v.push(`split-${n}-${n + 3}`);
	for (let n = 1; n <= 34; n += 3) v.push(`street-${n}-${n + 1}-${n + 2}`);
	for (const s of american ? ['0-1-2', '0-00-2', '00-2-3'] : ['0-1-2', '0-2-3']) v.push(`trio-${s}`);
	for (const n of range(1, 32)) if (n % 3 !== 0) v.push(`corner-${n}-${n + 1}-${n + 3}-${n + 4}`);
	for (let n = 1; n <= 31; n += 3) v.push(`line-${n}-${n + 5}`);
	if (american) v.push('topline');
	v.push('dozen-1', 'dozen-2', 'dozen-3', 'column-1', 'column-2', 'column-3', 'red', 'black', 'odd', 'even', 'low', 'high');
	return v;
}

/** Kantong yang dicakup sebuah tempat roulette (label `0`, `00`, `1`…). */
export function rouletteCoverage(spot: string): string[] {
	const [kind, ...rest] = spot.split('-');
	const nums = (xs: number[]) => xs.map(String);
	switch (kind) {
		case 'straight':
		case 'split':
		case 'street':
		case 'trio':
		case 'corner':
			return rest;
		case 'line':
			return nums(range(Number(rest[0]), Number(rest[1])));
		case 'topline':
			return ['0', '00', '1', '2', '3'];
		case 'dozen':
			return nums(range(12 * (Number(rest[0]) - 1) + 1, 12 * Number(rest[0])));
		case 'column':
			return nums(range(0, 11).map((r) => 3 * r + Number(rest[0])));
		case 'red':
			return nums(RED);
		case 'black':
			return nums(range(1, 36).filter((n) => !RED.includes(n)));
		case 'odd':
			return nums(range(1, 36).filter((n) => n % 2 === 1));
		case 'even':
			return nums(range(1, 36).filter((n) => n % 2 === 0));
		case 'low':
			return nums(range(1, 18));
		case 'high':
			return nums(range(19, 36));
		default:
			return [];
	}
}

/**
 * Tempat taruhan dalam yang dicakup tepat oleh kantong pilihan
 * (2 = split, 3 = street/trio, 4 = corner, 5 = top line, 6 = line), atau
 * `null` bila pilihan itu bukan taruhan yang ada.
 */
export function combined(selected: string[], american: boolean): string | null {
	if (selected.length < 2) return null;
	const sorted = [...selected].sort((a, b) => order(a) - order(b));
	const n = sorted.length;
	const zero = sorted.some((p) => p === '0' || p === '00');
	let spot: string;
	if (n === 2) spot = `split-${sorted.join('-')}`;
	else if (n === 3) spot = `${zero ? 'trio' : 'street'}-${sorted.join('-')}`;
	else if (n === 4) spot = `corner-${sorted.join('-')}`;
	else if (n === 5) spot = 'topline';
	else if (n === 6) spot = `line-${sorted[0]}-${sorted[5]}`;
	else return null;
	if (!rouletteSpots(american).includes(spot)) return null;
	const cover = rouletteCoverage(spot);
	return cover.length === n && sorted.every((p) => cover.includes(p)) ? spot : null;
}

/** Kantong roulette per baris tata letak (atas: 3, 6, …, 36). */
export const ROULETTE_ROWS = [3, 2, 1].map((top) => range(0, 11).map((c) => 3 * c + top));
