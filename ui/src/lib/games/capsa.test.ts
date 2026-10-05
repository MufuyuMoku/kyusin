import assert from 'node:assert/strict';
import { test } from 'node:test';

import { PEEK, ROW_GAP, arrangeCommand, rowsWidth, slotLeft, splitRows } from './capsa.ts';

test('slot kartu tiga baris tetap dan tidak bertabrakan', () => {
	assert.equal(slotLeft(0, 0, PEEK), 0);
	assert.equal(slotLeft(0, 2, PEEK), 2 * PEEK);
	assert.equal(slotLeft(1, 0, PEEK), 2 * PEEK + 44 + ROW_GAP);
	assert.equal(slotLeft(2, 0, PEEK), slotLeft(1, 4, PEEK) + 44 + ROW_GAP);
	assert.equal(rowsWidth(PEEK), slotLeft(2, 4, PEEK) + 44);
});

test('perintah arrange hanya untuk 3/5/5 kartu', () => {
	const cards = 'Qs Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c'.split(' ');
	const rows = splitRows(cards);
	assert.deepEqual(rows.map((r) => r.length), [3, 5, 5]);
	assert.equal(arrangeCommand(rows), `arrange ${cards.join(' ')}`);
	assert.equal(arrangeCommand([rows[0], rows[1], rows[2].slice(1)]), null);
});
