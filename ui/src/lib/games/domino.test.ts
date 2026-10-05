import assert from 'node:assert/strict';
import { test } from 'node:test';

import { CARD_H, CARD_W, backPixels } from './cards.ts';
import { PIP_PIXELS, dominoPixels, parseTile } from './domino.ts';

const lit = (rows: string[]) => rows.join('').split('').filter((c) => c === 'X').length;

test('kartu domino seukuran kartu remi dengan bulatan sebanyak angkanya', () => {
	const empty = lit(dominoPixels('0-0'));
	for (let hi = 0; hi <= 6; hi++)
		for (let lo = 0; lo <= hi; lo++) {
			const p = dominoPixels(`${hi}-${lo}`);
			assert.equal(p.length, CARD_H);
			assert.ok(p.every((r) => r.length === CARD_W));
			assert.equal(lit(p) - empty, (hi + lo) * PIP_PIXELS, `${hi}-${lo}`);
		}
});

test('urutan sisi bebas; selain kartu domino = tertutup', () => {
	assert.deepEqual(parseTile('4-6'), [6, 4]);
	assert.deepEqual(dominoPixels('4-6'), dominoPixels('6-4'));
	assert.deepEqual(dominoPixels('??'), backPixels());
	assert.equal(parseTile('7-1'), null);
});
