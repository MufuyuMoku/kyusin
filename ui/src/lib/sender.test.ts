import assert from 'node:assert/strict';
import { mock, test } from 'node:test';

import { Sender, TIMEOUT } from './sender.ts';

function recorder() {
	const updates: [string | null, unknown][] = [];
	return { updates, onUpdate: (s: string | null, e: unknown) => updates.push([s, e]) };
}

test('satu perintah sekaligus; klik kedua diabaikan', async () => {
	const r = recorder();
	const sender = new Sender(10_000, r.onUpdate);
	let calls = 0;
	let finish!: () => void;
	const first = sender.send('check', () => {
		calls++;
		return new Promise<void>((res) => (finish = res));
	});
	await sender.send('check', async () => void calls++);
	assert.equal(calls, 1);
	assert.equal(sender.sending, 'check');
	finish();
	await first;
	assert.equal(sender.sending, null);
	assert.deepEqual(r.updates, [
		['check', null],
		[null, null]
	]);
});

test('host tidak menjawab: penanda dilepas dan TIMEOUT dilaporkan', async () => {
	mock.timers.enable({ apis: ['setTimeout'] });
	try {
		const r = recorder();
		const sender = new Sender(10_000, r.onUpdate);
		const pending = sender.send('check', () => new Promise(() => {}));
		await Promise.resolve();
		mock.timers.tick(9_999);
		await Promise.resolve();
		assert.equal(sender.sending, 'check', 'belum habis waktu');
		mock.timers.tick(1);
		await pending;
		assert.equal(sender.sending, null, 'kontrol dibuka lagi');
		assert.deepEqual(r.updates.at(-1), [null, TIMEOUT]);
		// Perintah berikutnya bisa dikirim lagi.
		let ran = false;
		await sender.send('fold', async () => void (ran = true));
		assert.ok(ran);
	} finally {
		mock.timers.reset();
	}
});

test('kesalahan host dilaporkan dan penanda dilepas', async () => {
	const r = recorder();
	const sender = new Sender(10_000, r.onUpdate);
	await sender.send('arrange x', () => Promise.reject({ id: 'ditolak', en: 'rejected' }));
	assert.equal(sender.sending, null);
	assert.deepEqual(r.updates.at(-1), [null, { id: 'ditolak', en: 'rejected' }]);
});
