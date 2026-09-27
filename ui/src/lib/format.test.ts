import { test } from 'node:test';
import assert from 'node:assert/strict';
import { format } from './format.ts';

test('parameter biasa', () => {
	assert.equal(format('Halo {nama}', { nama: 'A' }, 'id'), 'Halo A');
});

test('tunggal/jamak bahasa Inggris', () => {
	const tpl = "You're down to {chips} {chips|chip|chips}.";
	assert.equal(format(tpl, { chips: 1 }, 'en'), "You're down to 1 chip.");
	assert.equal(format(tpl, { chips: 0 }, 'en'), "You're down to 0 chips.");
	assert.equal(format(tpl, { chips: 2 }, 'en'), "You're down to 2 chips.");
	assert.equal(format('{n} {n|day|days}. {n} {n|day|days}.', { n: 1 }, 'en'), '1 day. 1 day.');
});

test('nilai bukan angka memakai bentuk jamak', () => {
	assert.equal(format('{n} {n|player|players}', { n: '2–4' }, 'en'), '2–4 players');
});

test('bahasa tanpa bentuk tunggal khusus', () => {
	assert.equal(format('{n} {n|x|y}', { n: 1 }, 'id'), '1 y');
});
