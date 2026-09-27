/**
 * Navigasi keyboard bersama (SPEC §4: menu dan game giliran bisa dimainkan
 * dengan keyboard). Semua kontrol yang bisa dipilih memakai `NavButton`
 * (atribut `data-nav`). Kontrol nonaktif memakai `aria-disabled`, bukan
 * `disabled`, supaya tetap bisa difokus dan alasannya terbaca; panah tetap
 * bisa melewatinya (D-033). Tanpa ketergantungan DOM supaya bisa diuji
 * dengan `node --test`.
 */

export const NAV_SELECTOR = '[data-nav]';

export interface Focusable {
	focus(): void;
}

/** Indeks berikutnya dalam daftar melingkar; `-1` = belum ada fokus. */
export function nextIndex(count: number, current: number, dir: 1 | -1): number {
	if (count === 0) return -1;
	if (current < 0) return dir > 0 ? 0 : count - 1;
	return (current + dir + count) % count;
}

/** Memindahkan fokus satu langkah dan mengembalikan item yang kini fokus. */
export function moveFocus<T extends Focusable>(
	items: readonly T[],
	active: unknown,
	dir: 1 | -1
): T | undefined {
	const i = nextIndex(items.length, items.indexOf(active as T), dir);
	const next = items[i];
	next?.focus();
	return next;
}
