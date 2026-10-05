// Tes jendela asli meja Blackjack (SPEC §4, §11; D-056):
//  1. keselarasan: kotak tangan sama lebar dan sebaris, kartu 44×60 dengan
//     geser tetap 18 px, kartu bandar sebaris dengan kartu pemain;
//  2. taruhan lewat tombol chip, lalu BAGI;
//  3. hit, stand, double, dan split masing-masing dimainkan (ronde diulang
//     sampai kartunya memungkinkan split; shoe diacak, jadi jumlah ronde
//     berbeda-beda);
//  4. hover dan kursor keyboard ke setiap kontrol meja tidak menggeser
//     elemen mana pun; kartu baru juga tidak menggeser kotak tangan;
//  5. saldo chip berubah sebesar hasil ronde;
//  6. Berhenti lewat menu jeda: shoe selesai, seed dibuka, verify cocok;
//     ringkasan melawan bandar tampil di statistik;
//  7. tangkapan layar di tiga tema.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fail, inView, resetToMenu, tableSig, theme } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

/** Posisi dan ukuran elemen meja yang harus diam. */
const layout = () => {
	const r = (el) => {
		const b = el.getBoundingClientRect();
		return [Math.round(b.x * 10) / 10, Math.round(b.y * 10) / 10, Math.round(b.width * 10) / 10, Math.round(b.height * 10) / 10];
	};
	const out = {};
	document.querySelectorAll('.table [data-hand]').forEach((h) => (out[`hand${h.dataset.hand}`] = r(h)));
	const dealer = document.querySelector('.table .dealer .hand');
	if (dealer) out.dealer = r(dealer);
	document.querySelectorAll('.table .controls button').forEach((b, i) => (out[`btn${i}:${b.textContent.trim()}`] = r(b)));
	return out;
};

function same(a, b, when) {
	for (const k of Object.keys(a)) {
		if (!(k in b)) continue;
		if (a[k].some((v, i) => Math.abs(v - b[k][i]) > 0.5)) fail(`${k} bergeser saat ${when}: ${a[k]} → ${b[k]}`);
	}
}

const state = () => {
	const t = document.querySelector('.table');
	const btn = (label) => {
		const b = [...document.querySelectorAll('.table .controls button')].find((x) => x.textContent.trim() === label);
		return b ? b.getAttribute('aria-disabled') !== 'true' : false;
	};
	return {
		fase: t?.dataset.fase,
		text: t?.innerText ?? '',
		hands: document.querySelectorAll('.table [data-hand] .cards').length,
		can: {
			hit: btn('[ HIT ]'),
			stand: btn('[ STAND ]'),
			double: btn('[ DOUBLE ]'),
			split: btn('[ SPLIT ]'),
			decline: btn('[ TOLAK ]')
		},
		chips: Number((/Chip: (\d+)/.exec(t?.innerText ?? '') ?? [])[1] ?? NaN),
		last: Number((/Ronde terakhir: ([+-]?\d+)/.exec(t?.innerText ?? '') ?? [])[1] ?? NaN)
	};
};

async function press(s, label) {
	const before = await s.exec(tableSig);
	await s.click(await s.find('xpath', `//div[contains(@class,'controls')]//button[normalize-space(.)='${label}']`));
	await s.waitFor((b) => tableSig() !== b, `meja berubah setelah ${label}`, 10000, before);
}

/** Keselarasan statis meja. */
async function checkAlignment(s) {
	const a = await s.exec(() => {
		const hands = [...document.querySelectorAll('.table [data-hand]')].map((h) => h.getBoundingClientRect());
		const cards = [...document.querySelectorAll('.table .card')].map((c) => {
			const b = c.getBoundingClientRect();
			return { x: b.x, y: b.y, w: b.width, h: b.height, parent: c.parentElement };
		});
		const offsets = [];
		for (let i = 1; i < cards.length; i++) if (cards[i].parent === cards[i - 1].parent) offsets.push(cards[i].x - cards[i - 1].x);
		return {
			widths: hands.map((h) => Math.round(h.width)),
			tops: hands.map((h) => Math.round(h.top)),
			lefts: hands.map((h) => Math.round(h.left)),
			sizes: cards.map((c) => `${Math.round(c.w)}x${Math.round(c.h)}`),
			offsets: offsets.map((o) => Math.round(o))
		};
	});
	if (new Set(a.widths).size !== 1) fail(`kotak tangan tidak sama lebar: ${a.widths}`);
	if (new Set(a.tops).size !== 1) fail(`kotak tangan tidak sebaris: ${a.tops}`);
	for (let i = 1; i < a.lefts.length; i++) if (a.lefts[i] <= a.lefts[i - 1]) fail(`urutan kotak tangan: ${a.lefts}`);
	if (a.sizes.some((z) => z !== '44x60')) fail(`ukuran kartu: ${a.sizes}`);
	if (a.offsets.some((o) => o !== 18)) fail(`geser kartu: ${a.offsets}`);
	await inView(s, ['.table', '.table .controls button'], 'blackjack: meja');
	return a;
}

/** Hover dan kursor keyboard ke setiap tombol meja tanpa menggeser apa pun. */
async function sweepControls(s, when) {
	const base = await s.exec(layout);
	const buttons = await s.exec(() => document.querySelectorAll('.table .controls button').length);
	for (let i = 0; i < buttons; i++) {
		await s.exec((i) => document.querySelectorAll('.table .controls button')[i].setAttribute('data-e2e', 'x'), i);
		const target = await s.find('css selector', '[data-e2e="x"]');
		await s.hover(target);
		same(base, await s.exec(layout), `hover tombol ${i} (${when})`);
		await s.exec(() => document.querySelector('[data-e2e="x"]')?.removeAttribute('data-e2e'));
	}
	// Kursor keyboard: fokus berpindah dengan panah ke setiap kontrol.
	await s.exec(() => document.querySelector('.table .controls button')?.focus());
	for (let i = 0; i < buttons; i++) {
		const active = await s.find('css selector', ':focus');
		await s.type(active, KEYS.right);
		same(base, await s.exec(layout), `kursor keyboard langkah ${i + 1} (${when})`);
	}
	return buttons;
}

export async function run(s, artifacts, log) {
	await resetToMenu(s);
	await s.click(await s.find('xpath', "//button[normalize-space(.)='› Blackjack']"));
	await s.waitFor(() => !!document.querySelector('.game'), 'layar Blackjack');
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'taruhan', 'meja siap taruhan');
	const start = await s.exec(state);
	if (!(start.chips > 0)) fail(`saldo chip tidak tampil: ${start.text}`);
	const wrap = await s.find('css selector', '.left');
	writeFileSync(join(artifacts, 'blackjack-p1-bet.png'), await s.elementScreenshot(wrap));
	log(`meja siap: saldo ${start.chips}; ${await sweepControls(s, 'taruhan')} kontrol taruhan, hover/kursor tidak menggeser`);

	// Taruhan lewat tombol chip: 10 + 10 = 20.
	await s.click(await s.find('xpath', "//div[contains(@class,'controls')]//button[normalize-space(.)='[ +10 ]']"));
	await s.waitFor(() => document.querySelector('.table').innerText.includes('Taruhan: 20'), 'taruhan 20');

	const used = { hit: false, stand: false, double: false, split: false };
	let rounds = 0;
	let splitShot = false;
	while (!(used.hit && used.stand && used.double && used.split) && rounds < 150) {
		// Shoe habis sebelum semua aksi sempat dimainkan: mulai shoe baru.
		if (await s.exec(() => !!document.querySelector('.match .result'))) {
			await s.click(await s.find('xpath', "//button[normalize-space(.)='[ SHOE BARU ]']"));
			await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'taruhan', 'shoe baru');
			log('shoe habis sebelum split; shoe baru dimulai');
		}
		if (await s.exec(() => document.querySelector('.table').innerText.includes('Taruhan: 10'))) {
			await s.click(await s.find('xpath', "//div[contains(@class,'controls')]//button[normalize-space(.)='[ +10 ]']"));
			await s.waitFor(() => document.querySelector('.table').innerText.includes('Taruhan: 20'), 'taruhan 20');
		}
		rounds++;
		const before = await s.exec(state);
		await press(s, '[ BAGI · 20 ]');
		// Taruhan tercatat saat dipasang: saldo langsung turun 20 (D-059).
		const dealt = await s.exec(state);
		if ((dealt.fase === 'giliran' || dealt.fase === 'asuransi') && dealt.chips !== before.chips - 20)
			fail(`taruhan belum dipotong saat dipasang: ${before.chips} → ${dealt.chips}`);
		let guard = 0;
		for (;;) {
			const st = await s.exec(state);
			if (st.fase === 'taruhan' || st.fase === 'selesai' || guard++ > 20) break;
			if (st.fase === 'asuransi') {
				await press(s, '[ TOLAK ]');
				continue;
			}
			const layoutBefore = await s.exec(layout);
			let pick = 'stand';
			if (st.can.split && !used.split) pick = 'split';
			else if (st.can.double && !used.double) pick = 'double';
			else if (st.can.hit && !used.hit) pick = 'hit';
			used[pick] = true;
			await press(s, `[ ${pick.toUpperCase()} ]`);
			// Kartu baru / tangan baru tidak menggeser kotak yang sudah ada.
			same(layoutBefore, await s.exec(layout), `aksi ${pick}`);
			if (pick === 'split' && !splitShot) {
				splitShot = true;
				await checkAlignment(s);
				writeFileSync(join(artifacts, 'blackjack-p1-split.png'), await s.elementScreenshot(wrap));
				log(`split: ${await sweepControls(s, 'setelah split')} kontrol, hover/kursor tidak menggeser`);
			}
		}
		const after = await s.exec(state);
		if (after.fase === 'selesai') continue;
		if (Number.isFinite(after.last) && Number.isFinite(before.chips) && after.chips !== before.chips + after.last)
			fail(`saldo ${before.chips} → ${after.chips}, padahal hasil ronde ${after.last}`);
	}
	await checkAlignment(s);
	if (!(used.hit && used.stand && used.double && used.split)) fail(`aksi belum lengkap setelah ${rounds} ronde: ${JSON.stringify(used)}`);
	const now = await s.exec(state);
	log(`${rounds} ronde: hit, stand, double, split dimainkan; saldo ${start.chips} → ${now.chips} sesuai hasil tiap ronde`);
	writeFileSync(join(artifacts, 'blackjack-p1-round.png'), await s.elementScreenshot(wrap));

	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		await checkAlignment(s);
		writeFileSync(join(artifacts, `blackjack-${name}-table.png`), await s.elementScreenshot(wrap));
	}
	await theme(s, 'p1');
	log('tema P3 dan P4: meja tetap selaras');

	// Berhenti lewat menu jeda (di antara ronde).
	if ((await s.exec(state)).fase === 'taruhan') {
		await s.type(await s.find('css selector', '.table .controls button'), KEYS.escape);
		await s.waitFor(() => !!document.querySelector('[role="dialog"]'), 'menu jeda');
		const f = await s.exec(() => document.activeElement?.textContent.trim());
		if (f !== '[ LANJUTKAN ]') fail(`fokus menu jeda: ${f}`);
		await s.click(await s.find('xpath', "//div[@role='dialog']//button[normalize-space(.)='[ BERHENTI ]']"));
	}
	await s.waitFor(() => !!document.querySelector('.match .result'), 'shoe selesai');
	const end = await s.exec(() => ({
		result: document.querySelector('.match .result')?.textContent.trim(),
		verify: document.body.innerText.includes('verify: semua cocok'),
		saved: document.body.innerText.includes('Replay tersimpan')
	}));
	if (!end.verify) fail(`verify tidak cocok (${end.result})`);
	if (!end.saved) fail('replay shoe tidak tersimpan');
	writeFileSync(join(artifacts, 'blackjack-p1-window-end.png'), await s.screenshot());
	log(`berhenti: ${end.result}; seed dibuka, verify cocok, replay tersimpan`);

	// Ringkasan melawan bandar di statistik.
	await resetToMenu(s);
	await s.click(await s.find('xpath', "//button[normalize-space(.)='› Statistik']"));
	await s.waitFor(() => !!document.querySelector('.casino [data-game="blackjack"]'), 'ringkasan bandar');
	const row = await s.exec(() =>
		[...document.querySelectorAll('.casino [data-game="blackjack"] [data-col]')].map((c) => c.textContent.trim())
	);
	if (Number(row[1]) < rounds) fail(`ronde di statistik ${row[1]} < ${rounds}`);
	writeFileSync(join(artifacts, 'blackjack-stats.png'), await s.elementScreenshot(await s.find('css selector', '.stats')));
	log(`statistik melawan bandar: ${row.join(' | ')}`);
}
