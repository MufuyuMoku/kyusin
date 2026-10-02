// Tes jendela asli sepuluh meja casino M5a (SPEC §4, §7, §11). Untuk
// setiap game:
//  1. keselarasan: semua kotak kartu berawal di x yang sama, kartu 44×60,
//     geser kartu tetap (18 px bertumpuk, 48 px terpisah, +16 px di antara
//     tangan depan dan belakang Pai Gow);
//  2. hover dan kursor keyboard ke setiap kontrol tidak menggeser apa pun;
//  3. taruhan lewat kontrol visual (tombol tempat atau BAGI) langsung
//     dipotong dari saldo (D-059);
//  4. satu ronde dimainkan sampai selesai dengan tombol pilihan (Pai Gow:
//     memilih kartu lalu HOUSE WAY); kartu baru tidak menggeser kotak;
//  5. saldo berubah sebesar hasil ronde; game satu ronde per sesi
//     menyelesaikan pertandingannya dengan verify cocok;
//  6. tangkapan layar.
// Ditambah: ronde berikutnya di meja satu ronde per sesi, tema P3/P4,
// berhenti lewat menu jeda di meja ber-shoe, dan statistik melawan bandar.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fail, resetToMenu, theme } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

const GAMES = [
	{ id: 'dragon-tiger', name: 'Dragon Tiger', spot: 'NAGA' },
	{ id: 'casino-war', name: 'Casino War', spot: 'ANTE' },
	{ id: 'baccarat', name: 'Baccarat (Punto Banco)', spot: 'PEMAIN' },
	{ id: 'red-dog', name: 'Red Dog' },
	{ id: 'andar-bahar', name: 'Andar Bahar', spot: 'BAHAR', perRound: true },
	{ id: 'three-card-poker', name: 'Three Card Poker', spot: 'ANTE', perRound: true },
	{ id: 'caribbean-stud', name: 'Caribbean Stud Poker', perRound: true },
	{ id: 'casino-holdem', name: "Casino Hold'em", perRound: true },
	{ id: 'let-it-ride', name: 'Let It Ride', perRound: true },
	{ id: 'pai-gow', name: 'Pai Gow Poker', perRound: true }
];

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

const state = () => {
	const t = document.querySelector('.table');
	const text = t?.innerText ?? '';
	return {
		fase: t?.dataset.fase,
		text,
		chips: Number((/Chip: (\d+)/.exec(text) ?? [])[1] ?? NaN),
		risk: Number((/Di meja: (\d+)/.exec(text) ?? [])[1] ?? NaN),
		last: Number((/Ronde terakhir: ([+-]?\d+)/.exec(text) ?? [])[1] ?? NaN),
		buttons: [...document.querySelectorAll('.table .controls button')].map((b) => ({
			label: b.textContent.trim(),
			on: b.getAttribute('aria-disabled') !== 'true'
		}))
	};
};

async function press(s, label) {
	const before = await s.exec(() => document.querySelector('.table')?.innerText ?? '');
	await s.click(await s.find('xpath', `//div[contains(@class,'controls')]//button[normalize-space(.)="${label}"]`));
	await s.waitFor((b) => (document.querySelector('.table')?.innerText ?? '') !== b, `meja berubah setelah ${label}`, 10000, before);
}

async function checkAlignment(s, id) {
	const a = await s.exec(() => {
		const hands = [...document.querySelectorAll('.table [data-row]')].map((h) => Math.round(h.getBoundingClientRect().left));
		const cards = [...document.querySelectorAll('.table .card')].map((c) => {
			const b = c.getBoundingClientRect();
			return { x: b.x, w: b.width, h: b.height, parent: c.parentElement };
		});
		const offsets = [];
		for (let i = 1; i < cards.length; i++) if (cards[i].parent === cards[i - 1].parent) offsets.push(Math.round(cards[i].x - cards[i - 1].x));
		return { hands, sizes: cards.map((c) => `${Math.round(c.w)}x${Math.round(c.h)}`), offsets };
	});
	if (new Set(a.hands).size > 1) fail(`${id}: kotak kartu tidak sejajar: ${a.hands}`);
	if (a.sizes.some((z) => z !== '44x60')) fail(`${id}: ukuran kartu ${a.sizes}`);
	if (a.offsets.some((o) => o !== 18 && o !== 48 && o !== 64)) fail(`${id}: geser kartu ${a.offsets}`);
	return a;
}

async function sweepControls(s, when) {
	const base = await s.exec(layout);
	const buttons = await s.exec(() => document.querySelectorAll('.table .controls button').length);
	for (let i = 0; i < buttons; i++) {
		await s.exec((i) => document.querySelectorAll('.table .controls button')[i].setAttribute('data-e2e', 'x'), i);
		await s.hover(await s.find('css selector', '[data-e2e="x"]'));
		same(base, await s.exec(layout), `hover tombol ${i} (${when})`);
		await s.exec(() => document.querySelector('[data-e2e="x"]')?.removeAttribute('data-e2e'));
	}
	await s.exec(() => document.querySelector('.table .controls button')?.focus());
	for (let i = 0; i < buttons; i++) {
		await s.type(await s.find('css selector', ':focus'), KEYS.right);
		same(base, await s.exec(layout), `kursor keyboard langkah ${i + 1} (${when})`);
	}
	return buttons;
}

async function open(s, g) {
	await resetToMenu(s);
	await s.click(await s.find('xpath', `//button[normalize-space(.)="› ${g.name}"]`));
	await s.waitFor(() => !!document.querySelector('.game'), `layar ${g.name}`);
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	// Shoe dari putaran sebelumnya masih tertunda: konfirmasi main baru.
	const confirm = await s.exec(() => [...document.querySelectorAll('button')].some((b) => b.textContent.trim() === '[ YA, MAIN BARU ]'));
	if (confirm) await s.click(await s.find('xpath', "//button[normalize-space(.)='[ YA, MAIN BARU ]']"));
	await s.waitFor((id) => document.querySelector(`.table[data-game="${id}"]`)?.dataset.fase === 'taruhan', `meja ${g.id}`, 10000, g.id);
}

/** Pasang taruhan terkecil lewat kontrol visual sampai kartu dibagi. */
async function bet(s, g) {
	const before = await s.exec(state);
	if (g.spot) {
		await press(s, `[ + ${g.spot} ]`);
		const placed = await s.exec(state);
		if (placed.chips !== before.chips - placed.risk || !(placed.risk > 0))
			fail(`${g.id}: taruhan tidak dipotong saat dipasang (${before.chips} → ${placed.chips}, di meja ${placed.risk})`);
		await press(s, '[ BAGI ]');
	} else {
		const deal = before.buttons.find((b) => b.label.startsWith('[ BAGI · '));
		if (!deal?.on) fail(`${g.id}: tombol BAGI tidak aktif`);
		await press(s, deal.label);
		const dealt = await s.exec(state);
		if (dealt.fase !== 'taruhan' && dealt.fase !== 'selesai' && dealt.chips !== before.chips - dealt.risk)
			fail(`${g.id}: taruhan tidak dipotong saat dipasang (${before.chips} → ${dealt.chips}, di meja ${dealt.risk})`);
	}
	return before.chips;
}

/** Mainkan pilihan sampai ronde selesai; kotak kartu tidak bergeser. */
async function finishRound(s, g) {
	for (let guard = 0; guard < 6; guard++) {
		const st = await s.exec(state);
		if (st.fase === 'taruhan' || st.fase === 'selesai') return;
		const base = await s.exec(layout);
		if (g.id === 'pai-gow') {
			// Pilih lalu batalkan satu kartu: tidak menggeser apa pun.
			const card = await s.find('css selector', '.table .pick button');
			await s.click(card);
			if (!(await s.exec(() => document.querySelector('.table .pick button')?.getAttribute('aria-pressed') === 'true')))
				fail('pai-gow: kartu tidak terpilih');
			same(base, await s.exec(layout), 'memilih kartu');
			await s.click(card);
			await press(s, '[ HOUSE WAY ]');
		} else {
			const choice = st.buttons.find((b) => b.on);
			if (!choice) fail(`${g.id}: tidak ada pilihan aktif di fase ${st.fase}`);
			await press(s, choice.label);
		}
		same(base, await s.exec(layout), `pilihan ${g.id}`);
	}
	fail(`${g.id}: ronde tidak selesai`);
}

export async function run(s, artifacts, log) {
	for (const g of GAMES) {
		await open(s, g);
		await checkAlignment(s, g.id);
		const controls = await sweepControls(s, `${g.id} taruhan`);
		const chips = await bet(s, g);
		await checkAlignment(s, g.id);
		await finishRound(s, g);
		await checkAlignment(s, g.id);
		const end = await s.exec(state);
		if (!Number.isFinite(end.last)) fail(`${g.id}: hasil ronde tidak tampil: ${end.text}`);
		if (end.chips !== chips + end.last) fail(`${g.id}: saldo ${chips} → ${end.chips}, padahal hasil ronde ${end.last}`);
		let fair = '';
		if (g.perRound) {
			await s.waitFor(() => document.body.innerText.includes('verify: semua cocok'), `${g.id}: verify`, 10000);
			fair = ', seed ronde dibuka, verify cocok';
		}
		writeFileSync(join(artifacts, `meja-${g.id}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
		log(`${g.name}: selaras, ${controls} kontrol tanpa geser, taruhan dipotong saat dipasang, saldo ${chips} → ${end.chips} (${end.last >= 0 ? '+' : ''}${end.last})${fair}`);
	}

	// Ronde berikutnya di meja satu ronde per sesi: komitmen baru.
	const ab = GAMES.find((g) => g.id === 'three-card-poker');
	await open(s, ab);
	await bet(s, ab);
	await finishRound(s, ab);
	const commit = () => document.querySelector('.right')?.innerText ?? '';
	const first = await s.exec(commit);
	await press(s, '[ + ANTE ]');
	await s.waitFor((f) => (document.querySelector('.right')?.innerText ?? '') !== f, 'komitmen ronde baru', 10000, first);
	const st = await s.exec(state);
	if (st.fase !== 'taruhan' || !(st.risk > 0)) fail(`ronde berikutnya: fase ${st.fase}, di meja ${st.risk}`);
	log('ronde berikutnya: taruhan di meja yang selesai memulai pertandingan baru dengan komitmen seed baru');
	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		await checkAlignment(s, ab.id);
		writeFileSync(join(artifacts, `meja-${ab.id}-${name}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
	}
	await theme(s, 'p1');
	log('tema P3 dan P4: meja tetap selaras');

	// Berhenti lewat menu jeda di meja ber-shoe.
	const bac = GAMES.find((g) => g.id === 'baccarat');
	await open(s, bac);
	await bet(s, bac);
	await s.type(await s.find('css selector', '.table .controls button'), KEYS.escape);
	await s.waitFor(() => !!document.querySelector('[role="dialog"]'), 'menu jeda');
	await s.click(await s.find('xpath', "//div[@role='dialog']//button[normalize-space(.)='[ BERHENTI ]']"));
	await s.waitFor(() => !!document.querySelector('.match .result'), 'shoe selesai');
	if (!(await s.exec(() => document.body.innerText.includes('verify: semua cocok')))) fail('baccarat: verify tidak cocok');
	writeFileSync(join(artifacts, 'meja-baccarat-end.png'), await s.screenshot());
	log('baccarat: berhenti lewat menu jeda, seed shoe dibuka, verify cocok');

	await resetToMenu(s);
	await s.click(await s.find('xpath', "//button[normalize-space(.)='› Statistik']"));
	await s.waitFor(() => document.querySelectorAll('.casino [data-game]').length >= 11, 'ringkasan bandar semua meja');
	writeFileSync(join(artifacts, 'meja-stats.png'), await s.elementScreenshot(await s.find('css selector', '.stats')));
	const rows = await s.exec(() => document.querySelectorAll('.casino [data-game]').length);
	log(`statistik melawan bandar: ${rows} game`);
}
