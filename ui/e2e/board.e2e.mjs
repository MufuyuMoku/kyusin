// Tes jendela asli papan Reversi (SPEC §4, §11; D-038): keselarasan,
// kursor keyboard dan hover ke setiap sel tanpa menggeser sel, satu langkah
// melawan bot, dan tiga tema. Tangkapan layar disimpan sebagai artefak.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { checkAlignment, readCells, resetToMenu, sameCells, sweepHover, sweepKeyboard, theme } from './helpers.mjs';

export async function run(s, artifacts, log) {
	await resetToMenu(s);
	await s.click(await s.find('xpath', "//button[contains(., 'Reversi')]"));
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64, 'papan 64 sel');
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"].can').length > 0, 'giliran pemain');

	const base = await s.exec(readCells);
	const shape = checkAlignment(base);
	log(`selaras: sel ${shape.cell}, jarak ${shape.pitch}px`);
	log(`kursor keyboard: ${await sweepKeyboard(s, base)} sel, tidak ada sel yang bergeser`);
	const legal = await s.exec(() => document.querySelector('[role="gridcell"].can').dataset.sq);
	log(`hover: ${await sweepHover(s, base)} sel, tidak ada sel yang bergeser`);

	await s.hover(await s.find('css selector', `[data-sq="${legal}"]`));
	writeFileSync(join(artifacts, 'reversi-p1-window.png'), await s.screenshot());
	const wrap = await s.find('css selector', '.left');
	writeFileSync(join(artifacts, 'reversi-p1-board.png'), await s.elementScreenshot(wrap));

	await s.click(await s.find('css selector', `[data-sq="${legal}"]`));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"].can').length > 0, 'giliran pemain setelah bot');
	sameCells(base, await s.exec(readCells), 'setelah langkah dan balasan bot');
	const moves = await s.exec(() => document.querySelector('.moves')?.textContent ?? '');
	log(`langkah dimainkan: ${moves.trim()}`);
	writeFileSync(join(artifacts, 'reversi-p1-after-move.png'), await s.elementScreenshot(wrap));

	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		checkAlignment(await s.exec(readCells));
		writeFileSync(join(artifacts, `reversi-${name}-board.png`), await s.elementScreenshot(wrap));
	}
	log('tema P3 dan P4: tetap selaras');
}
