import { test } from 'node:test';
import assert from 'node:assert/strict';
import { GREETINGS, pickGreeting, type GreetingContext } from './greetings.ts';

const HOUR = 3_600_000;
// Rabu sore, tanpa rentetan hari: hanya sapaan umum yang memenuhi syarat.
const base: GreetingContext = { hour: 15, weekday: 3, streak: 1, gapMs: 5 * HOUR };

test('sesi pertama mendapat sapaan perkenalan', () => {
	assert.equal(pickGreeting({ ...base, gapMs: null }, null).id, 'first');
});

test('larut malam dan lama tidak datang dikenali; yang lebih penting didahulukan', () => {
	assert.equal(pickGreeting({ ...base, hour: 2 }, null).id, 'late_night');
	const p = pickGreeting({ ...base, hour: 2, gapMs: 10 * 24 * HOUR }, null);
	assert.equal(p.id, 'long_absence');
	assert.equal(p.params.days, 10);
});

test('syarat yang datanya belum ada (M3/M4) dilewati otomatis', () => {
	for (let i = 0; i < 50; i++) {
		const id = pickGreeting(base, null).id;
		assert.ok(!['name', 'last_game', 'chips_low', 'chips_high'].includes(id), id);
	}
});

test('begitu datanya ada, sapaan bersyarat ikut aktif', () => {
	assert.equal(pickGreeting({ ...base, chips: 200 }, null).id, 'chips_low');
	const named = pickGreeting({ ...base, profile: 'Mufu' }, null);
	assert.equal(named.id, 'name');
	assert.equal(named.params.name, 'Mufu');
	const game = pickGreeting({ ...base, lastGame: 'Reversi' }, null);
	assert.deepEqual([game.id, game.params.game], ['last_game', 'Reversi']);
});

test('tidak mengulang sapaan sesi sebelumnya', () => {
	for (const prev of GREETINGS.map((g) => g.id)) {
		for (const r of [0, 0.5, 0.99]) {
			const ctx = { ...base, hour: 2 };
			assert.notEqual(pickGreeting(ctx, prev, () => r).id, prev);
		}
	}
});

test('malam, akhir pekan, dan rentetan hari', () => {
	assert.equal(pickGreeting({ ...base, hour: 19 }, null).id, 'evening');
	assert.equal(pickGreeting({ ...base, hour: 21 }, null).id, 'evening');
	assert.notEqual(pickGreeting({ ...base, hour: 22 }, null, () => 0).id, 'evening');
	for (const weekday of [0, 6]) assert.equal(pickGreeting({ ...base, weekday }, null).id, 'weekend');
	assert.notEqual(pickGreeting({ ...base, streak: 2 }, null, () => 0).id, 'streak');
	const s = pickGreeting({ ...base, streak: 4, hour: 19 }, null);
	assert.deepEqual([s.id, s.params.n], ['streak', 4]);
});

test('selalu ada cadangan umum', () => {
	const p = pickGreeting(base, null, () => 0);
	assert.match(p.id, /^generic_/);
	assert.ok(GREETINGS.filter((g) => g.priority === 0).length >= 2);
});
