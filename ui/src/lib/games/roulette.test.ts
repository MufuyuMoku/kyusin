import assert from 'node:assert/strict';
import { test } from 'node:test';

import { ROULETTE_ROWS, combined, rouletteCoverage, rouletteSpots } from './roulette.ts';

test('daftar tempat sama jumlahnya dengan mesin Rust', () => {
	// 37 straight + 3 split nol + 57 split + 12 street + 2 trio + 22 corner
	// + 11 line + 12 luar (Eropa); Amerika +1 straight, +2 split nol, +1 trio,
	// + top line.
	assert.equal(rouletteSpots(false).length, 37 + 3 + 57 + 12 + 2 + 22 + 11 + 12);
	assert.equal(rouletteSpots(true).length, 38 + 5 + 57 + 12 + 3 + 22 + 11 + 1 + 12);
	assert.equal(new Set(rouletteSpots(true)).size, rouletteSpots(true).length);
});

test('tata letak memuat 1–36 sekali, baris atas 3 … 36', () => {
	assert.deepEqual(ROULETTE_ROWS.flat().sort((a, b) => a - b), Array.from({ length: 36 }, (_, i) => i + 1));
	assert.deepEqual(ROULETTE_ROWS[0].slice(0, 2), [3, 6]);
});

test('pilihan angka menjadi taruhan gabungan yang tepat', () => {
	assert.equal(combined(['20', '17'], false), 'split-17-20');
	assert.equal(combined(['17', '18'], false), 'split-17-18');
	assert.equal(combined(['17', '19'], false), null, 'tidak bersebelahan');
	assert.equal(combined(['3', '0'], false), 'split-0-3');
	assert.equal(combined(['3', '0'], true), null, 'Amerika: 0 bersebelahan 1 dan 2 saja');
	assert.equal(combined(['00', '0'], true), 'split-0-00');
	assert.equal(combined(['3', '00', '2'], true), 'trio-00-2-3');
	assert.equal(combined(['13', '15', '14'], false), 'street-13-14-15');
	assert.equal(combined(['17', '21', '18', '20'], false), 'corner-17-18-20-21');
	assert.equal(combined(['1', '0', '3', '2', '00'], true), 'topline');
	assert.equal(combined(['1', '0', '3', '2', '00'], false), null);
	assert.equal(combined(['4', '5', '6', '7', '8', '9'], false), 'line-4-9');
	assert.equal(combined(['4', '5', '6', '7', '8', '10'], false), null);
	assert.deepEqual(rouletteCoverage('line-4-9'), ['4', '5', '6', '7', '8', '9']);
});
