// Tes jendela asli meja Capsa Susun M5b-2 (SPEC §4, §7, §11; D-065):
//  1. duduk: buy-in 2.000 dipindahkan dari saldo (D-059);
//  2. keselarasan: kotak kartu kursi lawan sejajar, tangan dan susunan
//     sejajar, kartu 44×60; meja dan semua kontrol aksi muat di jendela
//     bawaan tanpa gulir (SPEC §4 Rev. 13);
//  3. hover dan kursor keyboard ke setiap kontrol dan kartu tidak menggeser
//     apa pun; memilih kartu, memindahkannya ke baris, dan
//     mengembalikannya tidak menggeser kotak mana pun;
//  4. SARAN lalu KIRIM: bot menyusun, semua susunan dibuka;
//  5. tema P3 dan P4 tetap selaras;
//  6. berdiri: tumpukan kembali ke saldo, verify cocok, rating lokal tampil.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fail, inView, resetToMenu, theme } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

const layout = () => {
	const r = (el) => {
		const b = el.getBoundingClientRect();
		return [Math.round(b.x * 10) / 10, Math.round(b.y * 10) / 10, Math.round(b.width * 10) / 10, Math.round(b.height * 10) / 10];
	};
	const out = {};
	document.querySelectorAll('.table [data-row]').forEach((h) => (out[`row:${h.dataset.row}`] = r(h)));
	document.querySelectorAll('.table .controls button').forEach((b, i) => (out[`btn${i}:${b.textContent.trim()}`] = r(b)));
	return out;
};

function same(a, b, when) {
	for (const k of Object.keys(a)) {
		if (!(k in b)) continue;
		if (a[k].some((v, i) => Math.abs(v - b[k][i]) > 0.5)) fail(`${k} bergeser saat ${when}: ${a[k]} → ${b[k]}`);
	}
}

const rowsOnly = (l) => Object.fromEntries(Object.entries(l).filter(([k]) => k.startsWith('row:')));

async function checkAlignment(s, when) {
	const a = await s.exec(() => {
		const left = (q) => [...document.querySelectorAll(q)].map((h) => Math.round(h.getBoundingClientRect().left));
		return {
			seats: left('.table [data-row^="kursi"]'),
			mine: left('.table [data-row="tangan"], .table [data-row="susun"]'),
			sizes: [...document.querySelectorAll('.table .card')].map((c) => {
				const b = c.getBoundingClientRect();
				return `${Math.round(b.width)}x${Math.round(b.height)}`;
			})
		};
	});
	if (new Set(a.seats).size !== 1) fail(`capsa ${when}: kotak kursi lawan tidak sejajar: ${a.seats}`);
	if (new Set(a.mine).size !== 1) fail(`capsa ${when}: tangan dan susunan tidak sejajar: ${a.mine}`);
	if (a.sizes.some((z) => z !== '44x60')) fail(`capsa ${when}: ukuran kartu ${a.sizes}`);
	await inView(s, ['.table', '.table .controls button'], `capsa ${when}`);
}

async function button(s, label) {
	await s.click(await s.find('xpath', `//div[contains(@class,'table')]//button[normalize-space(.)="${label}" or starts-with(normalize-space(.), "${label}")]`));
}

const chipsText = () => Number((/Chip: (\d+)/.exec(document.querySelector('.table')?.innerText ?? '') ?? [])[1] ?? NaN);

export async function run(s, artifacts, log) {
	await resetToMenu(s);
	const before = await s.exec(async () => (await window.__TAURI_INTERNALS__.invoke('profile_get')).chips);
	await s.click(await s.find('xpath', '//button[normalize-space(.)="› Capsa Susun"]'));
	await s.waitFor(() => !!document.querySelector('.game'), 'layar Capsa Susun');
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	const confirm = await s.exec(() => [...document.querySelectorAll('button')].some((b) => b.textContent.trim() === '[ YA, MAIN BARU ]'));
	if (confirm) await s.click(await s.find('xpath', "//button[normalize-space(.)='[ YA, MAIN BARU ]']"));
	await s.waitFor(() => !!document.querySelector('.table[data-game="capsa-susun"]'), 'meja Capsa Susun', 15000);
	await s.waitFor(
		() => document.querySelectorAll('.table .controls button').length > 0 || document.querySelector('.table')?.dataset.fase !== 'susun',
		'giliran menyusun',
		30000
	);
	const sat = await s.exec(chipsText);
	if (sat !== before - 2000) fail(`capsa: buy-in tidak dipotong (${before} → ${sat})`);

	let controls = 0;
	if ((await s.exec(() => document.querySelector('.table')?.dataset.fase)) === 'susun') {
		await checkAlignment(s, 'menyusun');
		// Hover dan kursor keyboard ke setiap tombol meja (kartu dan kontrol).
		const base = await s.exec(layout);
		controls = await s.exec(() => document.querySelectorAll('.table button').length);
		for (let i = 0; i < controls; i++) {
			await s.exec((i) => document.querySelectorAll('.table button')[i].setAttribute('data-e2e', 'x'), i);
			await s.hover(await s.find('css selector', '[data-e2e="x"]'));
			same(base, await s.exec(layout), `hover tombol ${i}`);
			await s.exec(() => document.querySelector('[data-e2e="x"]')?.removeAttribute('data-e2e'));
		}
		await s.exec(() => document.querySelector('.table button')?.focus());
		for (let i = 0; i < controls; i++) {
			await s.type(await s.find('css selector', ':focus'), KEYS.right);
			same(base, await s.exec(layout), `kursor keyboard langkah ${i + 1}`);
		}

		// Pilih satu kartu, pindahkan ke depan, kembalikan.
		const first = await s.exec(() => document.querySelector('.table [data-row="tangan"] .card')?.dataset.card);
		await s.click(await s.find('css selector', `.table [data-row="tangan"] [data-card="${first}"] button`));
		if (!(await s.exec((c) => document.querySelector(`.table [data-row="tangan"] [data-card="${c}"] button`)?.getAttribute('aria-pressed') === 'true', first)))
			fail(`capsa: kartu ${first} tidak terpilih`);
		await button(s, '[ KE DEPAN');
		if (!(await s.exec((c) => !!document.querySelector(`.table [data-row="susun"] [data-card="${c}"]`), first)))
			fail(`capsa: kartu ${first} tidak pindah ke baris depan`);
		same(rowsOnly(base), await s.exec(layout), 'memindahkan kartu');
		await s.click(await s.find('css selector', `.table [data-row="susun"] [data-card="${first}"] button`));
		if (!(await s.exec((c) => !!document.querySelector(`.table [data-row="tangan"] [data-card="${c}"]`), first)))
			fail(`capsa: kartu ${first} tidak kembali ke tangan`);
		same(rowsOnly(base), await s.exec(layout), 'mengembalikan kartu');

		await button(s, '[ SARAN ]');
		const placed = await s.exec(() => document.querySelectorAll('.table [data-row="susun"] .card').length);
		if (placed !== 13) fail(`capsa: saran mengisi ${placed} kartu`);
		await checkAlignment(s, 'saran');
		writeFileSync(join(artifacts, 'capsa-arrange.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
		await button(s, '[ KIRIM ]');
	}
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'antara', 'capsa: susunan dibuka', 30000);
	const opened = await s.exec(() => [...document.querySelectorAll('.table [data-row^="kursi"] .card')].filter((c) => c.dataset.card !== '??').length);
	if (opened === 0) fail('capsa: kartu lawan tidak dibuka');
	await s.waitFor(() => document.querySelectorAll('.table .controls button').length > 0, 'capsa: kontrol antara tangan', 15000);
	await checkAlignment(s, 'dibuka');
	writeFileSync(join(artifacts, 'capsa-hand.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		await checkAlignment(s, `tema ${name}`);
		writeFileSync(join(artifacts, `capsa-${name}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
	}
	await theme(s, 'p1');

	const stack = await s.exec(() => {
		const line = document.querySelector('.table [data-row="susun"]')?.parentElement?.querySelector('.line')?.textContent ?? '';
		return Number((/tumpukan (\d+)/.exec(line) ?? [])[1] ?? NaN);
	});
	await button(s, '[ BERDIRI ]');
	await s.waitFor(() => !!document.querySelector('.match .result'), 'capsa: sesi selesai', 15000);
	const end = await s.exec(async () => ({
		chips: (await window.__TAURI_INTERNALS__.invoke('profile_get')).chips,
		verify: document.body.innerText.includes('verify: semua cocok'),
		rating: document.querySelector('.match .rating')?.textContent.trim() ?? ''
	}));
	if (end.chips !== before - 2000 + stack) fail(`capsa: saldo ${before} → ${end.chips}, padahal tumpukan akhir ${stack}`);
	if (!end.verify) fail('capsa: verify tidak cocok');
	if (!end.rating) fail('capsa: rating lokal tidak tampil');
	writeFileSync(join(artifacts, 'capsa-end.png'), await s.screenshot());
	log(
		`Capsa Susun: buy-in 2000 dipotong, selaras dan muat tanpa gulir, ${controls} tombol tanpa geser, pindah/kembalikan kartu tanpa geser, susunan dibuka, tema P3/P4 selaras, berdiri dengan ${stack} (saldo ${before} → ${end.chips}), verify cocok, ${end.rating}`
	);
}
