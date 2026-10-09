// Tes jendela asli papan taruhan casino dadu dan roda M6a (SPEC §4, §7,
// §11): Roulette Eropa/Amerika, Sic Bo, Chuck-a-luck, Big Six, Fan-Tan,
// Craps, dan Pai Gow ubin. Untuk setiap game:
//  1. keselarasan: sel satu kelompok berlebar sama dan sejajar per baris
//     dan kolom; label dan jumlah taruhan di sel tidak terpotong; papan dan
//     kontrol muat tanpa gulir (`inView`);
//  2. hover dan kursor keyboard ke setiap kontrol tidak menggeser apa pun;
//  3. taruhan lewat sel papan langsung dipotong dari saldo (D-059), dan
//     jumlahnya tampil di sel;
//  4. ronde dijalankan (PUTAR/LEMPAR/BUKA); hasil (dadu, kantong, kancing)
//     tampil utuh; saldo berubah sebesar hasil ronde; seed ronde dibuka dan
//     verify cocok;
//  5. tangkapan layar.
// Ditambah: Roulette mode GABUNG (split dari dua angka), Craps mode TARIK
// dan satu penembak sampai seven-out, Pai Gow ubin memilih ubin tanpa
// menggeser lalu HOUSE WAY.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { changedSince, fail, inView, resetToMenu, tableSig, untruncated } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

const GAMES = [
	{ id: 'roulette-eropa', name: 'Roulette Eropa', spot: 'red', verb: 'PUTAR' },
	{ id: 'roulette-amerika', name: 'Roulette Amerika', spot: 'topline', verb: 'PUTAR' },
	{ id: 'sic-bo', name: 'Sic Bo', spot: 'small', verb: 'LEMPAR' },
	{ id: 'chuck-a-luck', name: 'Chuck-a-luck', spot: 'single-3', verb: 'LEMPAR' },
	{ id: 'big-six', name: 'Big Six', spot: '1', verb: 'PUTAR' },
	{ id: 'fan-tan', name: 'Fan-Tan', spot: 'kwok-1-2', verb: 'BUKA' },
	{ id: 'craps', name: 'Craps', spot: 'pass', verb: 'LEMPAR' },
	{ id: 'pai-gow-ubin', name: 'Pai Gow (ubin)' }
];

const layout = () => {
	const r = (el) => {
		const b = el.getBoundingClientRect();
		return [Math.round(b.x * 10) / 10, Math.round(b.y * 10) / 10, Math.round(b.width * 10) / 10, Math.round(b.height * 10) / 10];
	};
	const out = {};
	document.querySelectorAll('.table [data-row], .table [data-group], .table [data-spot], .table [data-pocket]').forEach((h, i) => {
		out[`${i}:${h.dataset.row ?? h.dataset.group ?? h.dataset.spot ?? h.dataset.pocket}`] = r(h);
	});
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
		net: Number((/Bersih: ([+-]?\d+)/.exec(text) ?? [])[1] ?? NaN),
		buttons: [...document.querySelectorAll('.table .controls button')].map((b) => ({
			label: b.textContent.trim(),
			on: b.getAttribute('aria-disabled') !== 'true'
		}))
	};
};

async function waitChange(s, before, what) {
	await s.waitFor(changedSince, `meja berubah setelah ${what}`, 10000, before);
}

async function press(s, label) {
	const before = await s.exec(tableSig);
	await s.click(await s.find('xpath', `//div[contains(@class,'controls')]//button[normalize-space(.)="${label}"]`));
	await waitChange(s, before, label);
}

/** Klik tombol kontrol tanpa menunggu meja berubah (mode, chip). */
async function tap(s, label) {
	await s.click(await s.find('xpath', `//div[contains(@class,'controls')]//button[normalize-space(.)="${label}"]`));
}

async function clickSpot(s, spot) {
	const before = await s.exec(tableSig);
	await s.click(await s.find('css selector', `.table [data-spot="${spot}"] button`));
	await waitChange(s, before, `tempat ${spot}`);
}

async function checkAlignment(s, id) {
	const a = await s.exec(() => {
		const groups = [...document.querySelectorAll('.table [data-group]')].map((g) => {
			const cells = [...g.querySelectorAll('.cell')].map((c) => c.getBoundingClientRect());
			const widths = new Set(cells.map((b) => Math.round(b.width)));
			const heights = new Set(cells.map((b) => Math.round(b.height)));
			const tops = [...new Set(cells.map((b) => Math.round(b.top)))];
			const lefts = [...new Set(cells.map((b) => Math.round(b.left)))];
			const cols = Number(g.querySelector('.cells')?.style.gridTemplateColumns.match(/repeat\((\d+)/)?.[1] ?? 0);
			return { id: g.dataset.group, n: cells.length, widths: [...widths], heights: [...heights], rows: tops.length, cols: lefts.length, want: cols };
		});
		const pockets = [...document.querySelectorAll('.table .wheel [data-pocket]:not(.zero)')].map((c) => c.getBoundingClientRect());
		return {
			groups,
			pocketSizes: [...new Set(pockets.map((b) => `${Math.round(b.width)}x${Math.round(b.height)}`))],
			pocketCols: new Set(pockets.map((b) => Math.round(b.left))).size,
			pocketRows: new Set(pockets.map((b) => Math.round(b.top))).size,
			hands: [...document.querySelectorAll('.table .hand')].map((h) => Math.round(h.getBoundingClientRect().left))
		};
	});
	for (const g of a.groups) {
		if (g.widths.length > 1 || g.heights.length > 1) fail(`${id}/${g.id}: ukuran sel berbeda ${g.widths} × ${g.heights}`);
		if (g.cols !== Math.min(g.want, g.n) || g.rows !== Math.ceil(g.n / g.want)) fail(`${id}/${g.id}: ${g.cols} kolom × ${g.rows} baris, seharusnya ${g.want} kolom`);
	}
	if (a.pocketSizes.length > 1) fail(`${id}: kotak angka roulette berbeda ukuran ${a.pocketSizes}`);
	if (id.startsWith('roulette') && (a.pocketCols !== 12 || a.pocketRows !== 3)) fail(`${id}: angka roulette ${a.pocketCols} kolom × ${a.pocketRows} baris`);
	if (new Set(a.hands).size > 1) fail(`${id}: kotak ubin tidak sejajar: ${a.hands}`);
	await untruncated(s, '.table .cell .tbtn, .table .pocket .tbtn, .table .face, .table .hand .line', `${id}`);
	await inView(s, ['.table', '.table .controls button'], `${id}: meja`);
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
	// Hover sel papan (beberapa contoh) juga tidak menggeser apa pun.
	const cells = await s.exec(() => Math.min(6, document.querySelectorAll('.table .cell button, .table .pocket button').length));
	for (let i = 0; i < cells; i++) {
		await s.exec((i) => document.querySelectorAll('.table .cell button, .table .pocket button')[i * 3]?.setAttribute('data-e2e', 'x'), i);
		const el = await s.exec(() => !!document.querySelector('[data-e2e="x"]'));
		if (!el) continue;
		await s.hover(await s.find('css selector', '[data-e2e="x"]'));
		same(base, await s.exec(layout), `hover sel ${i} (${when})`);
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

/** Pasang taruhan terkecil di satu sel; dipotong dari saldo saat dipasang. */
async function betSpot(s, g, spot) {
	const before = await s.exec(state);
	await clickSpot(s, spot);
	const placed = await s.exec(state);
	if (placed.chips !== before.chips - (placed.risk - before.risk) || !(placed.risk > before.risk))
		fail(`${g.id}: taruhan ${spot} tidak dipotong saat dipasang (${before.chips} → ${placed.chips}, di meja ${placed.risk})`);
	const amt = await s.exec((sp) => document.querySelector(`.table [data-spot="${sp}"] .amt`)?.textContent.trim(), spot);
	if (Number(amt) !== placed.risk - before.risk) fail(`${g.id}: jumlah di sel ${spot} = "${amt}", seharusnya ${placed.risk - before.risk}`);
	return before.chips;
}

async function verify(s, id) {
	await s.waitFor(() => document.body.innerText.includes('verify: semua cocok'), `${id}: verify`, 10000);
}

async function boardGame(s, g, log, artifacts) {
	await open(s, g);
	await checkAlignment(s, g.id);
	const controls = await sweepControls(s, `${g.id} taruhan`);
	const chips = await betSpot(s, g, g.spot);
	let extra = '';
	if (g.id === 'roulette-eropa') {
		// Mode GABUNG: 17 dan 20 menjadi split 17/20; memilih tidak menggeser.
		const base = await s.exec(layout);
		await tap(s, '[ GABUNG ]');
		for (const p of ['17', '20']) await s.click(await s.find('css selector', `.table [data-pocket="${p}"] button`));
		same(base, await s.exec(layout), 'memilih angka mode GABUNG');
		const label = await s.exec(() => [...document.querySelectorAll('.table .controls button')].map((b) => b.textContent.trim()).find((t) => t.startsWith('[ + ')));
		if (label !== '[ + SPLIT 17/20 ]') fail(`roulette: tombol gabung "${label}"`);
		await press(s, label);
		const inside = await s.exec(() => document.querySelector('.table [data-inside]')?.textContent.trim());
		if (!inside?.includes('split 17/20 ·10')) fail(`roulette: taruhan gabungan tidak tampil: "${inside}"`);
		await tap(s, '[ LURUS ]');
		extra = ', split 17/20 lewat mode GABUNG';
	}
	await checkAlignment(s, g.id);
	await press(s, `[ ${g.verb} ]`);
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'selesai', `${g.id}: ronde selesai`, 10000);
	await checkAlignment(s, g.id);
	const end = await s.exec(state);
	if (!Number.isFinite(end.last)) fail(`${g.id}: hasil ronde tidak tampil: ${end.text}`);
	if (end.chips !== chips + end.last) fail(`${g.id}: saldo ${chips} → ${end.chips}, padahal hasil ronde ${end.last}`);
	const face = await s.exec(() => document.querySelector('.table .face')?.textContent.trim());
	if (!face) fail(`${g.id}: hasil putaran/lemparan tidak tampil`);
	const dice = await s.exec(() => [...document.querySelectorAll('.table .die')].map((d) => d.dataset.die));
	if ((g.id === 'sic-bo' || g.id === 'chuck-a-luck') && !(dice.length === 3 && dice.every((d) => /^[1-6]$/.test(d))))
		fail(`${g.id}: dadu ${dice}`);
	await verify(s, g.id);
	writeFileSync(join(artifacts, `papan-${g.id}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
	log(
		`${g.name}: selaras, ${controls} kontrol tanpa geser, taruhan dipotong saat dipasang${extra}; ${face}; saldo ${chips} → ${end.chips} (${end.last >= 0 ? '+' : ''}${end.last}), seed ronde dibuka, verify cocok`
	);
}

async function craps(s, g, log, artifacts) {
	await open(s, g);
	await checkAlignment(s, g.id);
	const controls = await sweepControls(s, 'craps taruhan');
	const start = await betSpot(s, g, 'pass');
	let rolls = 0;
	let took = false;
	for (; rolls < 300; rolls++) {
		const st = await s.exec(state);
		if (st.fase === 'selesai') break;
		const v = await s.exec(() => ({
			pass: Number(document.querySelector('.table [data-spot="pass"] .amt')?.textContent || 0),
			point: /titik \d+/.test(document.querySelector('.table .face')?.textContent ?? '')
		}));
		if (!v.pass && !v.point) await betSpot(s, g, 'pass');
		if (v.point && !took) {
			// Field lalu TARIK: taruhan kembali ke saldo, sel kosong lagi.
			await betSpot(s, g, 'field');
			const before = await s.exec(state);
			const base = await s.exec(layout);
			await tap(s, '[ TARIK ]');
			same(base, await s.exec(layout), 'mode TARIK');
			await clickSpot(s, 'field');
			const after = await s.exec(state);
			if (after.chips !== before.chips + 10 || after.risk !== before.risk - 10) fail(`craps: TARIK field ${before.chips}/${before.risk} → ${after.chips}/${after.risk}`);
			took = true;
			// Tanpa taruhan lain yang bisa ditarik, mode TARIK padam sendiri.
			if (await s.exec(() => [...document.querySelectorAll('.table .controls button')].some((b) => b.textContent.trim() === '[ TARIK ]' && b.getAttribute('aria-pressed') === 'true')))
				await tap(s, '[ TARIK ]');
			await checkAlignment(s, g.id);
			writeFileSync(join(artifacts, 'papan-craps-titik.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
		}
		await press(s, '[ LEMPAR ]');
	}
	const end = await s.exec(state);
	if (end.fase !== 'selesai') fail(`craps: belum seven-out setelah ${rolls} lemparan`);
	if (end.chips !== start + end.net) fail(`craps: saldo ${start} → ${end.chips}, padahal bersih ${end.net}`);
	await checkAlignment(s, g.id);
	await verify(s, g.id);
	writeFileSync(join(artifacts, 'papan-craps.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
	log(
		`Craps: selaras, ${controls} kontrol tanpa geser, ${rolls} lemparan sampai seven-out${took ? ', TARIK field mengembalikan 10' : ''}; saldo ${start} → ${end.chips} (${end.net >= 0 ? '+' : ''}${end.net}), seed dibuka, verify cocok`
	);
}

async function paiGowUbin(s, g, log, artifacts) {
	await open(s, g);
	await checkAlignment(s, g.id);
	const controls = await sweepControls(s, 'pai gow ubin taruhan');
	const before = await s.exec(state);
	const deal = before.buttons.find((b) => b.label.startsWith('[ BAGI · '));
	if (!deal?.on) fail('pai-gow-ubin: tombol BAGI tidak aktif');
	await press(s, deal.label);
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'susun', 'pai-gow-ubin: susun', 10000);
	await checkAlignment(s, g.id);
	const tiles = await s.exec(() => [...document.querySelectorAll('.table [data-row="pemain"] [data-card]')].map((c) => c.dataset.card));
	if (tiles.length !== 4 || !tiles.every((x) => /^[1-6]-[1-6]$/.test(x))) fail(`pai-gow-ubin: ubin ${tiles}`);
	const base = await s.exec(layout);
	const tile = await s.find('css selector', '.table .pick button');
	await s.click(tile);
	if (!(await s.exec(() => document.querySelector('.table .pick button')?.getAttribute('aria-pressed') === 'true')))
		fail('pai-gow-ubin: ubin tidak terpilih');
	same(base, await s.exec(layout), 'memilih ubin');
	await s.click(tile);
	await press(s, '[ HOUSE WAY ]');
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'selesai', 'pai-gow-ubin: selesai', 10000);
	await checkAlignment(s, g.id);
	const end = await s.exec(state);
	if (end.chips !== before.chips + end.last) fail(`pai-gow-ubin: saldo ${before.chips} → ${end.chips}, padahal hasil ${end.last}`);
	const lines = await s.exec(() => [...document.querySelectorAll('.table .hand .line')].map((l) => l.textContent.trim()));
	if (!lines.every((l) => /tinggi .+ · rendah .+/.test(l))) fail(`pai-gow-ubin: nama tangan ${lines}`);
	await verify(s, g.id);
	writeFileSync(join(artifacts, 'papan-pai-gow-ubin.png'), await s.elementScreenshot(await s.find('css selector', '.left')));
	log(`Pai Gow (ubin): selaras, ${controls} kontrol tanpa geser, ubin dipilih tanpa geser, HOUSE WAY; ${lines.join(' / ')}; saldo ${before.chips} → ${end.chips}, verify cocok`);
}

export async function run(s, artifacts, log) {
	for (const g of GAMES) {
		if (g.id === 'craps') await craps(s, g, log, artifacts);
		else if (g.id === 'pai-gow-ubin') await paiGowUbin(s, g, log, artifacts);
		else await boardGame(s, g, log, artifacts);
	}
	// Ronde berikutnya: taruhan di papan yang selesai memulai pertandingan baru.
	const sb = GAMES.find((g) => g.id === 'sic-bo');
	await open(s, sb);
	await betSpot(s, sb, 'big');
	await press(s, '[ LEMPAR ]');
	await s.waitFor(() => document.querySelector('.table')?.dataset.fase === 'selesai', 'sic bo selesai', 10000);
	const first = await s.exec(() => document.querySelector('.right')?.innerText ?? '');
	await s.click(await s.find('css selector', '.table [data-spot="small"] button'));
	await s.waitFor((f) => (document.querySelector('.right')?.innerText ?? '') !== f, 'komitmen ronde baru', 10000, first);
	await s.waitFor(() => Number((/Di meja: (\d+)/.exec(document.querySelector('.table')?.innerText ?? '') ?? [])[1] ?? 0) > 0, 'taruhan ronde baru', 10000);
	log('ronde berikutnya: taruhan di papan yang selesai memulai pertandingan baru dengan komitmen seed baru');
}
