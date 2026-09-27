import { test } from 'node:test';
import assert from 'node:assert/strict';
import { History, complete, parse, type Context } from './commands.ts';

const ctx: Context = {
	gameIds: ['fixture', 'catur'],
	categoryKeys: ['papan', 'uji'],
	gameCommands: ['take 1', 'take 2', 'take 3']
};

test('parse membedakan perintah global, game, dan tak dikenal', () => {
	assert.deepEqual(parse('  ', ctx), { kind: 'empty' });
	assert.deepEqual(parse('man fixture', ctx), { kind: 'global', name: 'man', args: ['fixture'] });
	assert.deepEqual(parse('take   2', ctx), { kind: 'game', command: 'take 2' });
	assert.deepEqual(parse('hit', ctx), { kind: 'unknown', name: 'hit' });
});

test('autocomplete kata pertama dari perintah global dan game', () => {
	assert.equal(complete('tu', ctx).value, 'tutorial ');
	assert.equal(complete('ta', ctx).value, 'take ');
	const t = complete('t', ctx);
	assert.deepEqual(t.options.sort(), ['take', 'theme', 'tutorial']);
	assert.equal(t.value, 't');
});

test('autocomplete argumen dari manifest', () => {
	assert.equal(complete('man f', ctx).value, 'man fixture ');
	assert.deepEqual(complete('man ', ctx).options, ['fixture', 'catur']);
	assert.equal(complete('fx glow o', ctx).value, 'fx glow o');
	assert.equal(complete('fx glow of', ctx).value, 'fx glow off ');
	assert.deepEqual(complete('take ', ctx).options, ['1', '2', '3']);
});

test('tanpa kecocokan, input tidak berubah', () => {
	assert.deepEqual(complete('zzz', ctx), { value: 'zzz', options: [] });
});

test('riwayat naik-turun dan menyimpan draf', () => {
	const h = new History();
	h.push('help');
	h.push('man fixture');
	h.push('man fixture');
	assert.deepEqual(h.entries, ['help', 'man fixture']);
	assert.equal(h.up('dra'), 'man fixture');
	assert.equal(h.up(''), 'help');
	assert.equal(h.up(''), 'help');
	assert.equal(h.down(), 'man fixture');
	assert.equal(h.down(), 'dra');
});
