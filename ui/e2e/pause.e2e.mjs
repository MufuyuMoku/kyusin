// Tes jendela asli menu jeda, penundaan, dan kursor (SPEC §4 Rev. 9, §11):
//  a. `Esc` di tengah permainan membuka menu jeda dengan fokus di
//     Lanjutkan; jam berhenti selama jeda; Tunda & keluar lalu Lanjutkan
//     mengembalikan posisi, daftar langkah, dan jam yang sama (catur 5+0);
//     Reversi lewat komponen yang sama;
//  b. urutan campuran klik – pindah pilihan – langkah – panah: kursor
//     tersembunyi selama mouse dipakai, lalu muncul di petak tujuan langkah
//     terakhir (langkah bot tidak memindahkannya). Catur dan Reversi.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fail, resetToMenu } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

/**
 * Isi setiap petak dari label aksesibelnya, tanpa penanda yang bergantung
 * pada giliran (langkah sah, tujuan): catur "e4, putih pion", Reversi
 * "d3: hitam".
 */
const pieces = () =>
	Object.fromEntries(
		[...document.querySelectorAll('[role="gridcell"][data-sq]')].map((c) => {
			const parts = c.getAttribute('aria-label').split(',');
			return [c.dataset.sq, parts[0].includes(':') ? parts[0] : parts.slice(0, 2).join(',')];
		})
	);
const movesText = () => document.querySelector('.moves')?.textContent.trim() ?? '';
const timers = () => [...document.querySelectorAll('[role="timer"]')].map((t) => t.textContent.trim());
const seconds = (mmss) => {
	const [m, s] = mmss.split(':').map(Number);
	return m * 60 + s;
};
const focused = () => document.activeElement?.textContent.trim() ?? null;
const cursor = () => {
	const g = document.querySelector('[role="grid"]');
	const cell = document.getElementById(g.getAttribute('aria-activedescendant'));
	return { sq: cell?.dataset.sq, ring: document.querySelector('.layer.cursor-ring')?.closest('[data-sq]')?.dataset.sq ?? null };
};

async function button(s, text) {
	return s.find('xpath', `//button[normalize-space(.)='${text}']`);
}

async function openGame(s, name) {
	await s.click(await s.find('xpath', `//button[contains(., '${name}')]`));
	await s.waitFor(() => !!document.querySelector('.game'), `layar game ${name}`);
}

/** [ MAIN ]; bila masih ada pertandingan tertunda, konfirmasi main baru. */
async function play(s) {
	await s.click(await button(s, '[ MAIN ]'));
	const confirm = await s.exec(() => [...document.querySelectorAll('button')].some((b) => b.textContent.trim() === '[ YA, MAIN BARU ]'));
	if (confirm) await s.click(await button(s, '[ YA, MAIN BARU ]'));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64, 'papan 64 sel');
	await waitYourTurn(s);
}

async function waitYourTurn(s) {
	await s.waitFor(
		() => document.querySelectorAll('[role="gridcell"].can').length > 0 && !document.querySelector('.layer.selected'),
		'giliran pemain',
		20000
	);
}

async function clickSq(s, sq) {
	await s.click(await s.find('css selector', `[data-sq="${sq}"]`));
}

/** Esc → menu jeda dengan fokus di Lanjutkan. */
async function escToPause(s) {
	await s.type(await s.find('css selector', '[role="grid"]'), KEYS.escape);
	await s.waitFor(() => !!document.querySelector('[role="dialog"]'), 'menu jeda');
	await s.waitFor(() => document.activeElement?.textContent.trim() === '[ LANJUTKAN ]', 'fokus di Lanjutkan', 3000).catch(() => {});
	const f = await s.exec(focused);
	if (f !== '[ LANJUTKAN ]') fail(`fokus awal menu jeda di ${JSON.stringify(f)}, seharusnya [ LANJUTKAN ]`);
}

async function suspendAndResume(s, game) {
	await s.click(await button(s, '[ TUNDA & KELUAR ]'));
	await s.waitFor(() => !!document.querySelector('.game') && document.body.innerText.includes('pertandingan tertunda'), 'layar game dengan pertandingan tertunda');
	await s.click(await button(s, '[ LANJUTKAN ]'));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64, `papan ${game} dilanjutkan`);
	await waitYourTurn(s);
}

async function chessPause(s, artifacts, log) {
	await resetToMenu(s);
	await openGame(s, 'Catur');
	await s.click(await s.find('xpath', "//button[contains(., '5 menit + 0 detik')]"));
	await play(s);
	await clickSq(s, 'e2');
	await clickSq(s, 'e4');
	await s.waitFor((b) => document.querySelector('.moves')?.textContent.includes('1. e4 '), 'balasan bot');
	await waitYourTurn(s);

	await escToPause(s);
	const atPause = await s.exec(() => ({ t: [...document.querySelectorAll('[role="timer"]')].map((x) => x.textContent.trim()) }));
	writeFileSync(join(artifacts, 'pause-chess-menu.png'), await s.screenshot());
	await new Promise((r) => setTimeout(r, 2500));
	const still = await s.exec(timers);
	if (JSON.stringify(still) !== JSON.stringify(atPause.t)) fail(`jam berjalan saat jeda: ${atPause.t} → ${still}`);
	log(`Esc: menu jeda, fokus di Lanjutkan; jam berhenti (${still.join(' / ')})`);

	const before = { pieces: await s.exec(pieces), moves: await s.exec(movesText), timers: still };
	await s.click(await button(s, '[ TUNDA & KELUAR ]'));
	await s.waitFor(() => document.body.innerText.includes('pertandingan tertunda'), 'pertandingan tertunda di layar game');
	writeFileSync(join(artifacts, 'pause-chess-suspended.png'), await s.screenshot());
	await new Promise((r) => setTimeout(r, 2000));
	await s.click(await button(s, '[ LANJUTKAN ]'));
	await s.waitFor(() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64, 'papan catur dilanjutkan');
	await waitYourTurn(s);
	const after = { pieces: await s.exec(pieces), moves: await s.exec(movesText), timers: await s.exec(timers) };
	if (JSON.stringify(after.pieces) !== JSON.stringify(before.pieces)) fail('posisi berbeda setelah dilanjutkan');
	if (after.moves !== before.moves) fail(`langkah berbeda: ${before.moves} → ${after.moves}`);
	for (let i = 0; i < 2; i++) {
		const d = seconds(before.timers[i]) - seconds(after.timers[i]);
		if (d < 0 || d > 1) fail(`jam ${i} berubah selama ditunda: ${before.timers[i]} → ${after.timers[i]}`);
	}
	log(`tunda & lanjutkan: posisi sama, langkah "${after.moves}", jam ${after.timers.join(' / ')}`);

	// Lanjutkan lewat Enter pada fokus awal: menu tertutup, permainan jalan.
	await escToPause(s);
	await s.type(await s.find('css selector', '[role="dialog"] button'), KEYS.enter);
	await s.waitFor(() => !document.querySelector('[role="dialog"]'), 'menu jeda tertutup');
	return after;
}

/** (b) Pilih A, pindah pilih B tanpa membatalkan, melangkah, lalu panah. */
async function chessCursor(s, artifacts, log) {
	await clickSq(s, 'b1');
	await clickSq(s, 'g1');
	const sel = await s.exec(() => document.querySelector('.layer.selected')?.closest('[data-sq]')?.dataset.sq);
	if (sel !== 'g1') fail(`pilihan ${sel}, seharusnya g1 (pilihan b1 tertinggal?)`);
	const dots = await s.exec(() => [...document.querySelectorAll('.layer.dot')].map((d) => d.closest('[data-sq]').dataset.sq).sort());
	if (dots.includes('a3') || dots.includes('c3')) fail(`titik tujuan b1 tertinggal: ${dots}`);
	await clickSq(s, 'f3');
	await s.waitFor(() => /Nf3 \S+/.test(document.querySelector('.moves')?.textContent ?? ''), 'balasan bot setelah Nf3');
	await waitYourTurn(s);
	const hidden = await s.exec(cursor);
	if (hidden.ring !== null) fail(`kursor tampil di ${hidden.ring} selama mouse dipakai`);
	await s.type(await s.find('css selector', '[role="grid"]'), KEYS.up);
	const shown = await s.exec(cursor);
	if (shown.sq !== 'f3' || shown.ring !== 'f3') fail(`setelah panah kursor di ${JSON.stringify(shown)}, seharusnya f3`);
	writeFileSync(join(artifacts, 'cursor-chess-after-move.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
	await s.type(await s.find('css selector', '[role="grid"]'), KEYS.up);
	const moved = await s.exec(cursor);
	if (moved.sq !== 'f4') fail(`panah kedua: kursor di ${moved.sq}, seharusnya f4`);
	log('kursor catur: pilih b1 → pilih g1 → f3 → panah: kursor di f3 (bukan pilihan lama), panah berikutnya ke f4');

	// Selesaikan partai supaya tidak tertinggal sebagai pertandingan tertunda.
	await escToPause(s);
	await s.click(await button(s, '[ MENYERAH ]'));
	await s.click(await button(s, '[ YA, MENYERAH ]'));
	await s.waitFor(() => !!document.querySelector('.match .result'), 'hasil setelah menyerah');
	const verify = await s.exec(() => document.body.innerText.includes('verify: semua cocok'));
	if (!verify) fail('verify tidak cocok setelah menyerah dari menu jeda');
	log('menyerah dari menu jeda: hasil tercatat, verify cocok');
}

async function reversi(s, artifacts, log) {
	await resetToMenu(s);
	await openGame(s, 'Reversi');
	await play(s);
	const first = await s.exec(() => document.querySelector('[role="gridcell"].can').dataset.sq);
	await clickSq(s, first);
	await s.waitFor(() => (document.querySelector('.moves')?.textContent.trim().split(/\s+/).length ?? 0) >= 3, 'balasan bot Reversi');
	await waitYourTurn(s);
	const hidden = await s.exec(cursor);
	if (hidden.ring !== null) fail(`kursor Reversi tampil di ${hidden.ring} selama mouse dipakai`);
	await s.type(await s.find('css selector', '[role="grid"]'), KEYS.up);
	const shown = await s.exec(cursor);
	if (shown.ring !== first) fail(`kursor Reversi di ${JSON.stringify(shown)}, seharusnya ${first}`);
	log(`kursor Reversi: klik ${first} → bot → panah: kursor di ${first}`);

	await escToPause(s);
	writeFileSync(join(artifacts, 'pause-reversi-menu.png'), await s.screenshot());
	const before = { pieces: await s.exec(pieces), moves: await s.exec(movesText) };
	await suspendAndResume(s, 'Reversi');
	const after = { pieces: await s.exec(pieces), moves: await s.exec(movesText) };
	if (JSON.stringify(after) !== JSON.stringify(before)) fail('Reversi: posisi atau langkah berbeda setelah dilanjutkan');
	log(`Reversi: Esc → menu jeda (fokus Lanjutkan), tunda & lanjutkan: posisi sama ("${after.moves}")`);
	await escToPause(s);
	await s.click(await button(s, '[ MENYERAH ]'));
	await s.click(await button(s, '[ YA, MENYERAH ]'));
	await s.waitFor(() => !!document.querySelector('.match .result'), 'hasil Reversi setelah menyerah');
}

export async function run(s, artifacts, log) {
	await chessPause(s, artifacts, log);
	await chessCursor(s, artifacts, log);
	await reversi(s, artifacts, log);
}
