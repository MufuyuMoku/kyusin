// Tes jendela asli casino lotere dan instan M6b (SPEC §4, §7, §11): Keno,
// Bingo, kartu gosok, Hi-Lo, dan Video Poker (Jacks or Better, Deuces
// Wild, Joker Poker). Untuk setiap game:
//  1. keselarasan: sel/kartu berukuran sama dan sejajar; keterangan tidak
//     terpotong (`untruncated`); meja dan kontrol muat tanpa gulir (`inView`);
//  2. hover dan kursor keyboard ke setiap kontrol tidak menggeser apa pun;
//  3. satu ronde lewat kontrol visual; taruhan yang berjalan lebih dari satu
//     aksi (kartu gosok, Hi-Lo, Video Poker) dipotong saat dipasang (D-059);
//     saldo akhir = saldo awal + hasil ronde; seed ronde dibuka, verify cocok;
//  4. tangkapan layar.
// Ditambah: memilih angka Keno dan menahan kartu Video Poker tidak menggeser
// apa pun; angka Keno di meja yang selesai memulai ronde baru.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { changedSince, fail, inView, resetToMenu, tableSig, untruncated } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

const GAMES = [
	{ id: 'keno', name: 'Keno' },
	{ id: 'bingo', name: 'Bingo' },
	{ id: 'kartu-gosok', name: 'Kartu gosok' },
	{ id: 'hi-lo', name: 'Hi-Lo' },
	{ id: 'jacks-or-better', name: 'Jacks or Better' },
	{ id: 'deuces-wild', name: 'Deuces Wild' },
	{ id: 'joker-poker', name: 'Joker Poker' }
];

/** Posisi semua sel, kartu, baris, dan tombol meja (dipantau saat hover, kursor, dan pilihan). */
const layout = () => {
	const out = {};
	document
		.querySelectorAll('.table [data-num], .table [data-card], .table [data-cell], .table [data-sq], .table [data-ball], .table [data-row], .table .held, .table .controls button')
		.forEach((el, i) => {
			const b = el.getBoundingClientRect();
			out[`${i}:${el.dataset.num ?? el.dataset.card ?? el.dataset.cell ?? el.dataset.row ?? el.textContent.trim().slice(0, 12)}`] = [
				Math.round(b.x * 10) / 10,
				Math.round(b.y * 10) / 10,
				Math.round(b.width * 10) / 10,
				Math.round(b.height * 10) / 10
			];
		});
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

const button = (label) => `//div[contains(@class,'controls')]//button[normalize-space(.)="${label}"]`;

async function press(s, label) {
	const before = await s.exec(tableSig);
	await s.click(await s.find('xpath', button(label)));
	await s.waitFor(changedSince, `meja berubah setelah ${label}`, 10000, before);
}

/** Tombol pertama yang labelnya diawali `prefix`. */
async function pressPrefix(s, prefix) {
	const st = await s.exec(state);
	const b = st.buttons.find((x) => x.label.startsWith(prefix));
	if (!b?.on) fail(`tombol ${prefix}… tidak aktif: ${st.buttons.map((x) => x.label).join(' | ')}`);
	await press(s, b.label);
	return b.label;
}

async function clickSel(s, css) {
	const before = await s.exec(tableSig);
	await s.click(await s.find('css selector', css));
	await s.waitFor(changedSince, `meja berubah setelah ${css}`, 10000, before);
}

async function checkAlignment(s, id) {
	const a = await s.exec(() => {
		const size = (q) => [...new Set([...document.querySelectorAll(q)].map((e) => {
			const b = e.getBoundingClientRect();
			return `${Math.round(b.width)}x${Math.round(b.height)}`;
		}))];
		const axis = (q, f) => new Set([...document.querySelectorAll(q)].map((e) => Math.round(e.getBoundingClientRect()[f]))).size;
		return {
			keno: size('.table [data-num]'),
			kenoCols: axis('.table [data-num]', 'left'),
			kenoRows: axis('.table [data-num]', 'top'),
			sq: size('.table [data-sq]'),
			cells: size('.table [data-cell]'),
			cards: size('.table [data-card]'),
			cardTops: axis('.table [data-card]', 'top')
		};
	});
	for (const [k, v] of Object.entries(a)) if (Array.isArray(v) && v.length > 1) fail(`${id}: ukuran ${k} berbeda: ${v}`);
	if (id === 'keno' && (a.kenoCols !== 10 || a.kenoRows !== 8)) fail(`keno: papan ${a.kenoCols} × ${a.kenoRows}`);
	if (a.cards.length && a.cards[0] !== '44x60') fail(`${id}: ukuran kartu ${a.cards}`);
	if (a.cardTops > 1) fail(`${id}: kartu tidak sejajar`);
	await untruncated(s, '.table .outcome, .table .line, .table .pay, .table .prize, .table .held, .table .info span, .table .face', id);
	await inView(s, ['.table', '.table .controls button'], `${id}: meja`);
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
	const confirm = await s.exec(() => [...document.querySelectorAll('button')].some((b) => b.textContent.trim() === '[ YA, MAIN BARU ]'));
	if (confirm) await s.click(await s.find('xpath', "//button[normalize-space(.)='[ YA, MAIN BARU ]']"));
	await s.waitFor((id) => document.querySelector(`.table[data-game="${id}"]`)?.dataset.fase === 'taruhan', `meja ${g.id}`, 10000, g.id);
}

/** Taruhan yang berjalan: dipotong dari saldo saat dipasang. */
function deducted(id, before, after) {
	if (!(after.risk > 0) || after.chips !== before.chips - after.risk)
		fail(`${id}: taruhan tidak dipotong saat dipasang (${before.chips} → ${after.chips}, di meja ${after.risk})`);
}

async function playRound(s, g) {
	const start = await s.exec(state);
	let extra = '';
	if (g.id === 'keno') {
		const base = await s.exec(layout);
		for (const n of [5, 23, 47]) await clickSel(s, `.table [data-num="${n}"] button`);
		same(base, await s.exec(layout), 'memilih angka keno');
		const picked = await s.exec(() => [...document.querySelectorAll('.table [data-num] button[aria-pressed="true"]')].length);
		if (picked !== 3) fail(`keno: ${picked} angka terpilih`);
		await pressPrefix(s, '[ TARIK · ');
		extra = ', 3 angka dipilih tanpa geser';
	} else if (g.id === 'bingo') {
		await press(s, '[ 2 KARTU ]');
		const on = await s.exec(() => document.querySelectorAll('.table .card:not(.off)').length);
		if (on !== 2) fail(`bingo: ${on} kartu aktif`);
		await pressPrefix(s, '[ TARIK · ');
		extra = ', 2 kartu';
	} else if (g.id === 'kartu-gosok') {
		await pressPrefix(s, '[ BELI · ');
		deducted(g.id, start, await s.exec(state));
		const base = await s.exec(layout);
		await clickSel(s, '.table [data-cell="5"] button');
		same(base, await s.exec(layout), 'menggosok satu sel');
		const v = await s.exec(() => document.querySelector('.table [data-cell="5"]')?.dataset.value);
		if (!v) fail('kartu gosok: sel 5 tidak terbuka');
		await press(s, '[ GOSOK SEMUA ]');
		extra = ', satu sel digosok tanpa geser lalu GOSOK SEMUA';
	} else if (g.id === 'hi-lo') {
		await pressPrefix(s, '[ BUKA · ');
		deducted(g.id, start, await s.exec(state));
		// Tebakan yang lebih aman (pengali lebih kecil), lalu cash out bila menang.
		const pick = await s.exec(() => {
			const bs = [...document.querySelectorAll('.table .controls button')].map((b) => b.textContent.trim());
			const m = (p) => Number((bs.find((b) => b.startsWith(p)) ?? '').replace(/[^\d,]/g, '').replace(',', '.'));
			return m('[ HIGHER') <= m('[ LOWER') ? 'HIGHER' : 'LOWER';
		});
		await pressPrefix(s, `[ ${pick} · `);
		if ((await s.exec(state)).fase === 'tebak') await pressPrefix(s, '[ CASH OUT · ');
		extra = `, ${pick}`;
	} else {
		await pressPrefix(s, '[ BAGI · ');
		deducted(g.id, start, await s.exec(state));
		const base = await s.exec(layout);
		await s.click(await s.find('css selector', '.table .pick button'));
		if (!(await s.exec(() => document.querySelector('.table .pick button')?.getAttribute('aria-pressed') === 'true')))
			fail(`${g.id}: kartu tidak tertahan`);
		same(base, await s.exec(layout), 'menahan kartu');
		const label = await s.exec(() => document.querySelector('.table .held')?.textContent.trim());
		if (label !== 'TAHAN') fail(`${g.id}: label tahan "${label}"`);
		await press(s, '[ DRAW ]');
		extra = ', kartu 1 ditahan tanpa geser';
	}
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'selesai', `${g.id}: ronde selesai`, 10000);
	const end = await s.exec(state);
	if (!Number.isFinite(end.last)) fail(`${g.id}: hasil ronde tidak tampil: ${end.text}`);
	if (end.chips !== start.chips + end.last) fail(`${g.id}: saldo ${start.chips} → ${end.chips}, padahal hasil ronde ${end.last}`);
	return { start: start.chips, end, extra };
}

export async function run(s, artifacts, log) {
	for (const g of GAMES) {
		await open(s, g);
		await checkAlignment(s, g.id);
		const controls = await sweepControls(s, `${g.id} taruhan`);
		const r = await playRound(s, g);
		await checkAlignment(s, g.id);
		await s.waitFor(() => document.body.innerText.includes('verify: semua cocok'), `${g.id}: verify`, 10000);
		writeFileSync(join(artifacts, `lotere-${g.id}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
		log(
			`${g.name}: selaras, ${controls} kontrol tanpa geser${r.extra}; saldo ${r.start} → ${r.end.chips} (${r.end.last >= 0 ? '+' : ''}${r.end.last}), seed ronde dibuka, verify cocok`
		);
	}

	// Keno: angka di meja yang selesai memulai ronde baru (komitmen seed baru).
	const keno = GAMES[0];
	await open(s, keno);
	await clickSel(s, '.table [data-num="9"] button');
	await pressPrefix(s, '[ TARIK · ');
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'selesai', 'keno selesai', 10000);
	const first = await s.exec(() => document.querySelector('.right')?.innerText ?? '');
	await s.click(await s.find('css selector', '.table [data-num="10"] button'));
	await s.waitFor((f) => (document.querySelector('.right')?.innerText ?? '') !== f, 'komitmen ronde baru', 10000, first);
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'taruhan', 'ronde keno baru', 10000);
	const picks = await s.exec(() => [...document.querySelectorAll('.table [data-num] button[aria-pressed="true"]')].map((b) => b.textContent.trim()));
	if (picks.join() !== '10') fail(`ronde baru keno: pilihan ${picks}`);
	log('ronde berikutnya: angka keno di meja yang selesai memulai pertandingan baru dengan komitmen seed baru');
}
