import { test } from 'node:test';
import assert from 'node:assert/strict';
import { localDay, streak } from './session.ts';

const at = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime();

test('rentetan hari kalender berturut-turut, termasuk lintas bulan', () => {
	const days = ['2026-09-29', '2026-09-30', '2026-10-01', '2026-10-02'];
	assert.equal(streak(days, at(2026, 10, 2)), 4);
	assert.equal(streak(days, at(2026, 10, 3)), 0, 'hari ini belum tercatat');
	assert.equal(streak(['2026-10-01', '2026-10-03'], at(2026, 10, 3)), 1, 'ada hari bolong');
});

test('tanggal lokal berformat YYYY-MM-DD', () => {
	assert.equal(localDay(at(2026, 1, 5)), '2026-01-05');
});
