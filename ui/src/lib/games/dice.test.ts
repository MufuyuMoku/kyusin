import assert from 'node:assert/strict';
import { test } from 'node:test';

import { DIE, diePixels, pipCount } from './dice.ts';

test('setiap muka dadu berukuran tetap dan bulatannya sesuai nilai', () => {
	for (let n = 1; n <= 6; n++) {
		const p = diePixels(n);
		assert.equal(p.length, DIE);
		assert.ok(p.every((r) => r.length === DIE));
		assert.equal(pipCount(p), n);
	}
});
