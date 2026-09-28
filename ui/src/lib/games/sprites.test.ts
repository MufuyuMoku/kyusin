import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CHESS, DISC_DARK, DISC_LIGHT, outline } from './sprites.ts';

const count = (p: readonly string[]) => p.join('').split('').filter((c) => c === 'X').length;

test('semua sprite 12×12', () => {
	for (const p of [DISC_LIGHT, DISC_DARK, ...Object.values(CHESS)]) {
		assert.equal(p.length, 12);
		for (const row of p) assert.equal(row.length, 12);
	}
});

test('enam bidak catur berbeda satu sama lain, penuh maupun bergaris tepi', () => {
	const filled = Object.values(CHESS).map((p) => p.join(''));
	const outlined = Object.values(CHESS).map((p) => outline(p).join(''));
	assert.equal(new Set(filled).size, 6);
	assert.equal(new Set(outlined).size, 6);
});

test('versi bergaris tepi jelas lebih kosong dari versi penuh', () => {
	for (const [name, p] of Object.entries(CHESS)) {
		const lost = count(p) - count(outline(p));
		assert.ok(lost >= 4, `${name}: hanya ${lost} piksel berkurang`);
		assert.ok(count(outline(p)) >= 12, `${name}: garis tepi terlalu tipis`);
	}
});

test('beda antar bidak cukup besar (≥8 piksel) supaya terbaca di 40×40', () => {
	const entries = Object.entries(CHESS);
	for (let i = 0; i < entries.length; i++)
		for (let j = i + 1; j < entries.length; j++) {
			const a = entries[i][1].join('');
			const b = entries[j][1].join('');
			let diff = 0;
			for (let k = 0; k < a.length; k++) if (a[k] !== b[k]) diff++;
			assert.ok(diff >= 8, `${entries[i][0]} vs ${entries[j][0]}: ${diff}`);
		}
});
