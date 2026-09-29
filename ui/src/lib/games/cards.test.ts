import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CARD_H, CARD_W, RANKS, RANK_KEYS, SUITS_BIG, SUITS_SMALL, SUIT_KEYS, backPixels, cardPixels } from './cards.ts';

const diff = (a: readonly string[], b: readonly string[]) => {
	let n = 0;
	a.forEach((row, y) => {
		for (let x = 0; x < row.length; x++) if (row[x] !== b[y][x]) n++;
	});
	return n;
};

test('ukuran glyph dan kartu tetap', () => {
	for (const g of Object.values(RANKS)) {
		assert.equal(g.length, 7);
		for (const row of g) assert.equal(row.length, 5);
	}
	for (const g of Object.values(SUITS_SMALL)) {
		assert.equal(g.length, 5);
		for (const row of g) assert.equal(row.length, 5);
	}
	for (const g of Object.values(SUITS_BIG)) {
		assert.equal(g.length, 9);
		for (const row of g) assert.equal(row.length, 9);
	}
	for (const r of RANK_KEYS)
		for (const s of SUIT_KEYS) {
			const p = cardPixels(r + s);
			assert.equal(p.length, CARD_H);
			for (const row of p) assert.equal(row.length, CARD_W);
		}
});

test('13 peringkat dan 4 jenis jelas berbeda', () => {
	const ranks = Object.values(RANKS);
	for (let i = 0; i < ranks.length; i++)
		for (let j = i + 1; j < ranks.length; j++) assert.ok(diff(ranks[i], ranks[j]) >= 3, `${RANK_KEYS[i]} vs ${RANK_KEYS[j]}`);
	for (const set of [SUITS_SMALL, SUITS_BIG]) {
		const suits = Object.values(set);
		for (let i = 0; i < suits.length; i++)
			for (let j = i + 1; j < suits.length; j++) assert.ok(diff(suits[i], suits[j]) >= 3, `jenis ${i} vs ${j}`);
	}
});

test('52 kartu berbeda dan berbeda dari kartu tertutup', () => {
	const seen = new Set<string>();
	for (const r of RANK_KEYS)
		for (const s of SUIT_KEYS) {
			const key = cardPixels(r + s).join('\n');
			assert.ok(!seen.has(key), r + s);
			seen.add(key);
		}
	assert.equal(seen.size, 52);
	assert.ok(!seen.has(backPixels().join('\n')));
	assert.deepEqual(cardPixels('??'), backPixels());
	assert.deepEqual(cardPixels('Zz'), backPixels());
});
