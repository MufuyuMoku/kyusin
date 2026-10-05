// Tes jendela asli papan catur (SPEC §4, §11; D-041):
//  1. keselarasan papan;
//  2. kursor keyboard dan hover ke setiap sel tanpa menggeser sel;
//  3. seret-lepas: selama bidak diseret melintasi papan, tidak ada sel yang
//     bergeser; bidak bayangan tampil; lepas di tujuan = langkah;
//  4. klik-pilih-lalu-klik-tujuan: petak terpilih dan titik tujuan tampil;
//  5. satu partai pendek melawan bot sampai selesai (menyerah), lalu hasil
//     dan verify otomatis;
//  6. tangkapan layar di tiga tema.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { checkAlignment, inView, fail, readCells, resetToMenu, sameCells, sweepHover, sweepKeyboard, theme } from './helpers.mjs';
import { ELEMENT } from './webdriver.mjs';

const yourTurn = () =>
	document.querySelectorAll('[role="gridcell"].can').length > 0 && !document.querySelector('.layer.selected');

async function waitYourTurn(s, what) {
	await s.waitFor(yourTurn, what, 20000);
}

const movesText = () => document.querySelector('.moves')?.textContent ?? '';

/** Pilih petak asal lalu tujuan sah pertamanya lewat klik. */
async function clickMove(s) {
	const from = await s.exec(() => document.querySelector('[role="gridcell"].can').dataset.sq);
	// Perekam event untuk diagnosis bila pemilihan gagal.
	await s.exec(() => {
		const log = (window.__e2eEvents = []);
		const t0 = performance.now();
		const where = (el) => (el instanceof Element ? el.closest('[data-sq]')?.dataset.sq ?? el.tagName : String(el));
		for (const type of ['pointerdown', 'pointerup', 'pointermove', 'click'])
			document.addEventListener(
				type,
				(e) => log.push(`${Math.round(performance.now() - t0)} ${type} ${where(e.target)} (${e.clientX},${e.clientY})`),
				true
			);
		new MutationObserver(() =>
			log.push(`${Math.round(performance.now() - t0)} selected=${document.querySelector('.layer.selected')?.closest('[data-sq]')?.dataset.sq ?? null}`)
		).observe(document.querySelector('[role="grid"]'), { childList: true, subtree: true });
		window.addEventListener('unhandledrejection', (e) =>
			log.push(`${Math.round(performance.now() - t0)} reject ${JSON.stringify(e.reason) ?? String(e.reason)}`)
		);
	});
	await s.click(await s.find('css selector', `[data-sq="${from}"]`));
	await s.waitFor(() => !!document.querySelector('.layer.selected'), 'petak terpilih', 2000).catch(() => {});
	const picked = await s.exec(() => ({
		selected: document.querySelector('.layer.selected')?.closest('[data-sq]')?.dataset.sq,
		dots: [...document.querySelectorAll('.layer.dot')].map((d) => d.closest('[data-sq]').dataset.sq),
		events: window.__e2eEvents.filter((l) => !l.includes('pointermove')).slice(0, 30),
		moves: window.__e2eEvents.filter((l) => l.includes('pointermove')).length
	}));
	if (picked.selected !== from) {
		console.log(`  event: ${picked.moves} pointermove\n    ${picked.events.join('\n    ')}`);
		fail(`petak terpilih ${picked.selected}, seharusnya ${from}`);
	}
	if (picked.dots.length === 0) fail(`tidak ada titik tujuan untuk ${from}`);
	const before = await s.exec(movesText);
	await s.click(await s.find('css selector', `[data-sq="${picked.dots[0]}"]`));
	try {
		await s.waitFor((b) => (document.querySelector('.moves')?.textContent ?? '') !== b, 'langkah tercatat', 10000, before);
	} catch (e) {
		const state = await s.exec(() => ({
			events: window.__e2eEvents.filter((l) => !l.includes('pointermove')).slice(-20),
			selected: document.querySelector('.layer.selected')?.closest('[data-sq]')?.dataset.sq ?? null,
			can: [...document.querySelectorAll('[role="gridcell"].can')].map((c) => c.dataset.sq),
			status: document.querySelector('.match [role="status"]')?.textContent.trim() ?? null,
			alert: document.querySelector('[role="alert"]')?.textContent.trim() ?? null
		}));
		console.log(`  klik ${from} lalu ${picked.dots[0]} (titik: ${picked.dots.join(' ')}); ${JSON.stringify(state, null, 1)}`);
		throw e;
	}
	return `${from}-${picked.dots[0]}`;
}

export async function run(s, artifacts, log) {
	await resetToMenu(s);
	await s.click(await s.find('xpath', "//button[contains(., 'Catur')]"));
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64, 'papan 64 sel');
	await waitYourTurn(s, 'giliran putih');

	const base = await s.exec(readCells);
	const shape = checkAlignment(base);
	log(`selaras: sel ${shape.cell}, jarak ${shape.pitch}px`);
	log(`muat tanpa gulir: papan dan kontrol, sisa ${await inView(s, ['[role="grid"]', '.left .controls button'], 'papan')} px`);
	const wrap = await s.find('css selector', '.left');
	writeFileSync(join(artifacts, 'chess-p1-start.png'), await s.elementScreenshot(wrap));
	log(`kursor keyboard: ${await sweepKeyboard(s, base)} sel, tidak ada sel yang bergeser`);
	log(`hover: ${await sweepHover(s, base)} sel, tidak ada sel yang bergeser`);

	// Seret e2 → e4, melintasi beberapa petak; periksa sel di setiap titik.
	const cell = async (sq) => ({ [ELEMENT]: await s.find('css selector', `[data-sq="${sq}"]`) });
	const path = ['e3', 'd3', 'd4', 'e4'];
	const actions = [
		{ type: 'pointerMove', duration: 0, origin: await cell('e2'), x: 0, y: 0 },
		{ type: 'pointerDown', button: 0 }
	];
	await s.cmd('POST', '/actions', {
		actions: [{ type: 'pointer', id: 'mouse', parameters: { pointerType: 'mouse' }, actions }]
	});
	for (const sq of path) {
		await s.cmd('POST', '/actions', {
			actions: [
				{
					type: 'pointer',
					id: 'mouse',
					parameters: { pointerType: 'mouse' },
					actions: [{ type: 'pointerMove', duration: 50, origin: await cell(sq), x: 0, y: 0 }]
				}
			]
		});
		const drag = await s.exec(() => ({
			ghost: !!document.querySelector('.ghost svg'),
			over: document.querySelector('.layer.drop')?.closest('[data-sq]')?.dataset.sq
		}));
		if (!drag.ghost) fail(`bidak bayangan tidak tampil saat di atas ${sq}`);
		if (drag.over !== sq) fail(`sasaran seret ${drag.over}, seharusnya ${sq}`);
		sameCells(base, await s.exec(readCells), `menyeret di atas ${sq}`);
		if (sq === 'd4') writeFileSync(join(artifacts, 'chess-p1-dragging.png'), await s.elementScreenshot(wrap));
	}
	await s.cmd('POST', '/actions', {
		actions: [{ type: 'pointer', id: 'mouse', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerUp', button: 0 }] }]
	});
	await s.cmd('DELETE', '/actions');
	await s.waitFor(() => (document.querySelector('.moves')?.textContent ?? '').includes('1. e4'), 'langkah e4 dari seret');
	log('seret-lepas e2→e4: tidak ada sel yang bergeser selama diseret');
	await waitYourTurn(s, 'giliran setelah balasan bot');
	sameCells(base, await s.exec(readCells), 'setelah seret dan balasan bot');

	// Klik-pilih-lalu-klik-tujuan, beberapa langkah.
	const played = [];
	for (let i = 0; i < 4; i++) {
		played.push(await clickMove(s));
		if (i === 1) writeFileSync(join(artifacts, 'chess-p1-midgame.png'), await s.elementScreenshot(wrap));
		const over = await s.exec(() => !!document.querySelector('.match .result'));
		if (over) break;
		await waitYourTurn(s, 'giliran setelah bot');
		sameCells(base, await s.exec(readCells), `setelah langkah ${i + 2}`);
	}
	log(`klik-pilih-lalu-tujuan: ${played.join(', ')}`);

	// Selesaikan partai dengan menyerah, lalu periksa hasil dan verify.
	if (!(await s.exec(() => !!document.querySelector('.match .result')))) {
		await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MENYERAH ]']"));
		await s.click(await s.find('xpath', "//button[normalize-space(.)='[ YA, MENYERAH ]']"));
	}
	await s.waitFor(() => !!document.querySelector('.match .result'), 'hasil partai');
	const end = await s.exec(() => ({
		result: document.querySelector('.match .result')?.textContent.trim(),
		verify: document.body.innerText.includes('verify: semua cocok'),
		saved: document.body.innerText.includes('Replay tersimpan')
	}));
	if (!end.verify) fail(`verify tidak cocok atau tidak tampil (hasil: ${end.result})`);
	if (!end.saved) fail('replay tidak tersimpan');
	log(`partai selesai: ${end.result}; verify cocok; replay tersimpan`);
	writeFileSync(join(artifacts, 'chess-p1-window-end.png'), await s.screenshot());

	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		checkAlignment(await s.exec(readCells));
		writeFileSync(join(artifacts, `chess-${name}-board.png`), await s.elementScreenshot(wrap));
	}
	log('tema P3 dan P4: tetap selaras');
}
