// Tes jendela asli papan Reversi (SPEC §4, §11; D-038).
//
// Menjalankan aplikasi KyuSin sungguhan lewat tauri-driver, membuka
// pertandingan Reversi, lalu:
//  1. memeriksa keselarasan: semua sel satu kolom punya x dan lebar yang
//     sama, semua sel satu baris punya y dan tinggi yang sama, semua sel
//     berukuran sama;
//  2. menggerakkan kursor keyboard ke setiap sel (ular dari a1) dan
//     memastikan posisi serta ukuran semua sel tidak berubah sama sekali;
//  3. meng-hover setiap sel dengan penunjuk dan memeriksa hal yang sama;
//  4. meletakkan bidak, menunggu bot, dan memeriksa lagi;
//  5. menyimpan tangkapan layar (tiga tema) sebagai artefak.

import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { KEYS, Session } from './webdriver.mjs';

const SQUARES = [];
for (let r = 1; r <= 8; r++) for (const c of 'abcdefgh') SQUARES.push(`${c}${r}`);

function fail(msg) {
	throw new Error(msg);
}

/** Posisi dan ukuran semua sel papan, per nama petak. */
const readCells = () => {
	const out = {};
	for (const el of document.querySelectorAll('[role="gridcell"][data-sq]')) {
		const r = el.getBoundingClientRect();
		out[el.dataset.sq] = [r.x, r.y, r.width, r.height];
	}
	return out;
};

function sameCells(base, now, when) {
	for (const sq of SQUARES) {
		const a = base[sq];
		const b = now[sq];
		if (!b) fail(`${when}: sel ${sq} hilang`);
		if (a.some((v, i) => v !== b[i])) {
			fail(`${when}: sel ${sq} berubah dari [${a}] menjadi [${b}]`);
		}
	}
}

function checkAlignment(cells) {
	const [w, h] = [cells.a1[2], cells.a1[3]];
	for (const sq of SQUARES) {
		const [x, y, cw, ch] = cells[sq];
		if (cw !== w || ch !== h) fail(`ukuran sel ${sq} ${cw}×${ch}, seharusnya ${w}×${h}`);
		const colHead = cells[`${sq[0]}1`];
		const rowHead = cells[`a${sq[1]}`];
		if (x !== colHead[0]) fail(`sel ${sq} tidak sejajar dengan kolomnya (x ${x} ≠ ${colHead[0]})`);
		if (y !== rowHead[1]) fail(`sel ${sq} tidak sejajar dengan barisnya (y ${y} ≠ ${rowHead[1]})`);
	}
	// Jarak antarsel sama (garis 1px merata, tidak putus).
	const xs = 'abcdefgh'.split('').map((c) => cells[`${c}1`][0]);
	const ys = [1, 2, 3, 4, 5, 6, 7, 8].map((r) => cells[`a${r}`][1]);
	const steps = new Set([...xs.slice(1).map((x, i) => x - xs[i]), ...ys.slice(1).map((y, i) => y - ys[i])]);
	if (steps.size !== 1) fail(`jarak antarsel tidak rata: ${[...steps]}`);
	return { cell: `${w}×${h}`, pitch: [...steps][0] };
}

export async function run({ base, application, artifacts }) {
	mkdirSync(artifacts, { recursive: true });
	const s = await Session.start(base, application);
	const log = (m) => console.log(`  ${m}`);
	let saved = null;
	try {
		await s.waitFor(() => !!document.querySelector('.crt'), 'jendela KyuSin');
		// Mulai bersih tanpa boot; pengaturan pemain dikembalikan di akhir.
		saved = await s.exec(() => localStorage.getItem('kyusin.settings.v2'));
		await s.exec(() => {
			localStorage.setItem(
				'kyusin.settings.v2',
				JSON.stringify({ bootMode: 'off', lang: 'id', theme: 'p1', intensity: 30 })
			);
			location.reload();
		});
		await s.waitFor(() => !!document.querySelector('.shell main'), 'menu utama');

		await s.click(await s.find('xpath', "//button[contains(., 'Reversi')]"));
		await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
		await s.waitFor(
			() => document.querySelectorAll('[role="gridcell"][data-sq]').length === 64,
			'papan 64 sel'
		);
		await s.waitFor(
			() => document.querySelectorAll('[role="gridcell"].can').length > 0,
			'giliran pemain'
		);

		const base0 = await s.exec(readCells);
		const shape = checkAlignment(base0);
		log(`selaras: sel ${shape.cell}, jarak ${shape.pitch}px`);

		// 1. Kursor keyboard ke setiap sel.
		const grid = await s.find('css selector', '[role="grid"]');
		await s.exec(() => document.querySelector('[role="grid"]').focus());
		for (let i = 0; i < 8; i++) await s.type(grid, KEYS.up + KEYS.left);
		const cursorAt = () => {
			const g = document.querySelector('[role="grid"]');
			const id = g.getAttribute('aria-activedescendant');
			const cell = document.getElementById(id);
			return { sq: cell?.dataset.sq, drawn: !!cell?.querySelector('.layer.cursor') };
		};
		let visited = 0;
		for (let r = 0; r < 8; r++) {
			for (let k = 0; k < 8; k++) {
				const col = r % 2 === 0 ? k : 7 - k;
				const want = `${'abcdefgh'[col]}${r + 1}`;
				const at = await s.exec(cursorAt);
				if (at.sq !== want) fail(`kursor di ${at.sq}, seharusnya ${want}`);
				if (!at.drawn) fail(`kursor di ${want} tidak tergambar`);
				sameCells(base0, await s.exec(readCells), `kursor di ${want}`);
				visited++;
				if (k < 7) await s.type(grid, r % 2 === 0 ? KEYS.right : KEYS.left);
			}
			if (r < 7) await s.type(grid, KEYS.down);
		}
		log(`kursor keyboard: ${visited} sel, tidak ada sel yang bergeser`);
		// Kursor di petak legal untuk tangkapan layar.
		const legal = await s.exec(() => document.querySelector('[role="gridcell"].can').dataset.sq);

		// 2. Hover setiap sel.
		const hovered = () => {
			const h = [...document.querySelectorAll('[role="gridcell"]')].filter((c) => c.matches(':hover'));
			return h.map((c) => c.dataset.sq);
		};
		for (const sq of SQUARES) {
			const el = await s.find('css selector', `[data-sq="${sq}"]`);
			await s.hover(el);
			const h = await s.exec(hovered);
			if (h.length !== 1 || h[0] !== sq) fail(`hover di ${sq} terbaca ${JSON.stringify(h)}`);
			sameCells(base0, await s.exec(readCells), `hover di ${sq}`);
		}
		log('hover: 64 sel, tidak ada sel yang bergeser');

		// Tangkapan layar: kursor dan hover di petak legal.
		await s.hover(await s.find('css selector', `[data-sq="${legal}"]`));
		writeFileSync(join(artifacts, 'reversi-p1-window.png'), await s.screenshot());
		const wrap = await s.find('css selector', '.left');
		writeFileSync(join(artifacts, 'reversi-p1-board.png'), await s.elementScreenshot(wrap));

		// 3. Main satu langkah, tunggu bot, periksa lagi.
		await s.click(await s.find('css selector', `[data-sq="${legal}"]`));
		await s.waitFor(
			() => document.querySelectorAll('[role="gridcell"].can').length > 0,
			'giliran pemain setelah bot'
		);
		sameCells(base0, await s.exec(readCells), 'setelah langkah dan balasan bot');
		const moves = await s.exec(() => document.querySelector('.moves')?.textContent ?? '');
		log(`langkah dimainkan: ${moves.trim()}`);
		writeFileSync(join(artifacts, 'reversi-p1-after-move.png'), await s.elementScreenshot(wrap));

		// 4. Tema lain lewat konsol (perintah teks yang sama dengan pemain).
		const body = await s.find('css selector', 'body');
		for (const theme of ['p3', 'p4']) {
			await s.type(body, ':');
			const input = await s.find('css selector', '.console input');
			await s.type(input, `theme ${theme}${KEYS.enter}${KEYS.escape}`);
			await s.waitFor((t) => document.documentElement.dataset.theme === t, `tema ${theme}`, 5000, theme);
			checkAlignment(await s.exec(readCells));
			writeFileSync(join(artifacts, `reversi-${theme}-board.png`), await s.elementScreenshot(wrap));
		}
		log('tema P3 dan P4: tetap selaras');
	} finally {
		try {
			await s.exec((v) => {
				if (v === null) localStorage.removeItem('kyusin.settings.v2');
				else localStorage.setItem('kyusin.settings.v2', v);
			}, saved);
		} catch {
			// Sesi mungkin sudah rusak; pengaturan lama tetap ada di berkasnya.
		}
		await s.end().catch(() => {});
	}
}
