/**
 * Regresi D-033: navigasi panah macet di item nonaktif (flicker saat
 * reduced motion). Model fokus di sini meniru peramban: elemen dengan
 * atribut `disabled` asli menolak `focus()`, sedangkan `aria-disabled`
 * tetap bisa difokus.
 */

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { moveFocus, nextIndex } from './nav.ts';

class Doc {
	active: Item | null = null;
}

class Item {
	doc: Doc;
	id: number;
	ariaDisabled: boolean;
	nativeDisabled: boolean;
	constructor(doc: Doc, id: number, ariaDisabled: boolean, nativeDisabled = false) {
		this.doc = doc;
		this.id = id;
		this.ariaDisabled = ariaDisabled;
		this.nativeDisabled = nativeDisabled;
	}
	focus() {
		if (!this.nativeDisabled) this.doc.active = this;
	}
}

/** Menekan panah `presses` kali dan mencatat item yang fokus tiap kali. */
function walk(items: Item[], doc: Doc, dir: 1 | -1, presses: number): number[] {
	const seen: number[] = [];
	for (let i = 0; i < presses; i++) {
		moveFocus(items, doc.active, dir);
		seen.push(doc.active?.id ?? -1);
	}
	return seen;
}

function makeList(n: number, disabledMask: number, native = false) {
	const doc = new Doc();
	const items = Array.from({ length: n }, (_, i) => {
		const off = (disabledMask >> i) & 1;
		return new Item(doc, i, off === 1 && !native, off === 1 && native);
	});
	return { doc, items };
}

test('nextIndex melingkar dan mulai dari ujung bila belum ada fokus', () => {
	assert.equal(nextIndex(3, -1, 1), 0);
	assert.equal(nextIndex(3, -1, -1), 2);
	assert.equal(nextIndex(3, 2, 1), 0);
	assert.equal(nextIndex(3, 0, -1), 2);
	assert.equal(nextIndex(0, -1, 1), -1);
});

test('panah menjelajahi seluruh daftar turun dan naik, dengan item nonaktif di posisi mana pun', () => {
	for (let n = 1; n <= 7; n++) {
		const all = Array.from({ length: n }, (_, i) => i);
		for (let mask = 1; mask < 1 << n; mask++) {
			const down = makeList(n, mask);
			assert.deepEqual(walk(down.items, down.doc, 1, n), all, `turun n=${n} mask=${mask}`);
			// Terus turun melingkar ke atas lagi.
			assert.deepEqual(walk(down.items, down.doc, 1, 1), [0]);

			const up = makeList(n, mask);
			assert.deepEqual(walk(up.items, up.doc, -1, n), [...all].reverse(), `naik n=${n} mask=${mask}`);

			// Turun sampai bawah lalu naik sampai atas dari fokus yang sama.
			const both = makeList(n, mask);
			walk(both.items, both.doc, 1, n);
			assert.deepEqual(walk(both.items, both.doc, -1, n - 1), [...all].reverse().slice(1));
		}
	}
});

test('model meniru bug lama: `disabled` asli membuat navigasi macet', () => {
	// Item 1 dari 3 dinonaktifkan dengan atribut asli, seperti flicker dulu.
	const { doc, items } = makeList(3, 0b010, true);
	assert.deepEqual(walk(items, doc, 1, 3), [0, 0, 0]);
});

const here = dirname(fileURLToPath(import.meta.url));

function svelteFiles(dir: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
		const p = join(dir, e.name);
		if (e.isDirectory()) return svelteFiles(p);
		return e.name.endsWith('.svelte') ? [p] : [];
	});
}

test('semua kontrol navigasi lewat NavButton, tanpa atribut disabled asli', () => {
	const problems: string[] = [];
	for (const f of svelteFiles(join(here, '..'))) {
		const src = readFileSync(f, 'utf8');
		const isNavButton = f.endsWith('NavButton.svelte');
		if (!isNavButton && /\bdata-nav\b/.test(src)) problems.push(`${f}: data-nav di luar NavButton`);
		if (!isNavButton && /<button\b/.test(src)) problems.push(`${f}: <button> mentah; pakai NavButton`);
		if (/<button\b[^>]*\sdisabled[\s={>]/.test(src)) problems.push(`${f}: atribut disabled asli`);
	}
	assert.deepEqual(problems, []);
});
