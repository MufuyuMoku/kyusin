// Pemeriksaan bersama tes jendela asli papan (SPEC §4, §11; D-038).

import { KEYS } from './webdriver.mjs';

export const SQUARES = [];
for (let r = 1; r <= 8; r++) for (const c of 'abcdefgh') SQUARES.push(`${c}${r}`);

export function fail(msg) {
	throw new Error(msg);
}

/**
 * Tanda keadaan meja untuk menunggu perubahan setelah aksi (D-067): teks,
 * nilai setiap kartu (`data-card`), dan fase. Teks saja tidak cukup: kartu
 * berupa sprite SVG, jadi kartu baru setelah semua pemain check bisa
 * membuat teks meja persis sama (log tiga aksi terakhir "check · check ·
 * check", pot dan tumpukan tetap).
 */
export const tableSig = () => {
	const t = document.querySelector('.table');
	const cards = [...(t?.querySelectorAll('[data-card]') ?? [])].map((c) => c.dataset.card).join(',');
	return `${t?.dataset.fase}|${cards}|${t?.innerText ?? ''}`;
};

/** Syarat tunggu: tanda meja berbeda dari `before` (mandiri, dijalankan di halaman). */
export const changedSince = (before) => {
	const t = document.querySelector('.table');
	const cards = [...(t?.querySelectorAll('[data-card]') ?? [])].map((c) => c.dataset.card).join(',');
	return `${t?.dataset.fase}|${cards}|${t?.innerText ?? ''}` !== before;
};

/**
 * Keterangan kursi tidak boleh terpotong (`text-overflow: ellipsis`): teks
 * yang lebih lebar dari kotaknya berarti nilai (misalnya nilai pasangan
 * QiuQiu atau nama baris Capsa) tidak terbaca.
 */
export async function untruncated(s, selector, when) {
	const cut = await s.exec(
		(q) => [...document.querySelectorAll(q)].filter((el) => el.scrollWidth > el.clientWidth + 1).map((el) => el.textContent.trim()),
		selector
	);
	if (cut.length) fail(`${when}: keterangan terpotong: ${cut.join(' | ')}`);
}

/**
 * SPEC §4 Rev. 13: meja/papan dan kontrol aksi muat di jendela bawaan tanpa
 * gulir. Setiap elemen yang cocok dengan `selectors` harus seluruhnya berada
 * di area tampilan `main` yang belum digulir. Mengembalikan sisa ruang di
 * bawah elemen terbawah (px).
 */
export async function inView(s, selectors, when) {
	const r = await s.exec((sels) => {
		const main = document.querySelector('main');
		const m = main.getBoundingClientRect();
		const els = sels.flatMap((q) => [...document.querySelectorAll(q)].map((el) => [q, el]));
		const out = els
			.map(([q, el]) => [q, el.textContent.trim().slice(0, 24), el.getBoundingClientRect()])
			.filter(([, , b]) => b.top < m.top - 0.5 || b.bottom > m.bottom + 0.5 || b.left < m.left - 0.5 || b.right > m.right + 0.5)
			.map(([q, t, b]) => `${q} "${t}" [${Math.round(b.left)},${Math.round(b.top)}–${Math.round(b.right)},${Math.round(b.bottom)}]`);
		const bottom = Math.max(...els.map(([, el]) => el.getBoundingClientRect().bottom));
		return { count: els.length, out, scroll: main.scrollTop, room: Math.round(m.bottom - bottom), area: `${Math.round(m.width)}×${Math.round(m.height)}` };
	}, selectors);
	if (r.count === 0) fail(`${when}: tidak ada elemen ${selectors.join(', ')}`);
	if (r.scroll !== 0) fail(`${when}: halaman tergulir ${r.scroll} px`);
	if (r.out.length) fail(`${when}: di luar area tampilan ${r.area} tanpa gulir: ${r.out.join('; ')}`);
	return r.room;
}

/** Posisi dan ukuran semua sel papan, per nama petak (dijalankan di halaman). */
export const readCells = () => {
	const out = {};
	for (const el of document.querySelectorAll('[role="gridcell"][data-sq]')) {
		const r = el.getBoundingClientRect();
		out[el.dataset.sq] = [r.x, r.y, r.width, r.height];
	}
	return out;
};

export function sameCells(base, now, when) {
	for (const sq of SQUARES) {
		const a = base[sq];
		const b = now[sq];
		if (!b) fail(`${when}: sel ${sq} hilang`);
		if (a.some((v, i) => v !== b[i])) fail(`${when}: sel ${sq} berubah dari [${a}] menjadi [${b}]`);
	}
}

/**
 * Semua sel satu kolom punya x dan lebar sama, satu baris punya y dan
 * tinggi sama, semua sel berukuran sama, dan jarak antarsel rata. Berlaku
 * untuk papan terbalik juga (urutan diambil dari posisi di layar).
 */
export function checkAlignment(cells) {
	const all = SQUARES.map((sq) => cells[sq]);
	const [w, h] = [all[0][2], all[0][3]];
	for (const [i, c] of all.entries()) {
		if (c[2] !== w || c[3] !== h) fail(`ukuran sel ${SQUARES[i]} ${c[2]}×${c[3]}, seharusnya ${w}×${h}`);
	}
	for (const f of 'abcdefgh') {
		const xs = new Set([1, 2, 3, 4, 5, 6, 7, 8].map((r) => cells[`${f}${r}`][0]));
		if (xs.size !== 1) fail(`kolom ${f} tidak lurus: x = ${[...xs]}`);
	}
	for (let r = 1; r <= 8; r++) {
		const ys = new Set([...'abcdefgh'].map((f) => cells[`${f}${r}`][1]));
		if (ys.size !== 1) fail(`baris ${r} tidak lurus: y = ${[...ys]}`);
	}
	const xs = [...new Set(all.map((c) => c[0]))].sort((a, b) => a - b);
	const ys = [...new Set(all.map((c) => c[1]))].sort((a, b) => a - b);
	const steps = new Set([...xs.slice(1).map((x, i) => x - xs[i]), ...ys.slice(1).map((y, i) => y - ys[i])]);
	if (steps.size !== 1) fail(`jarak antarsel tidak rata: ${[...steps]}`);
	return { cell: `${w}×${h}`, pitch: [...steps][0] };
}

/** Petak di bawah kursor keyboard, dan apakah kursornya garis tepi tanpa latar. */
const cursorAt = () => {
	const g = document.querySelector('[role="grid"]');
	const cell = document.getElementById(g.getAttribute('aria-activedescendant'));
	const ring = cell?.querySelector('.layer.cursor-ring');
	const st = ring && getComputedStyle(ring);
	const outline = !!st && st.borderTopWidth === '2px' && st.backgroundColor === 'rgba(0, 0, 0, 0)';
	return { sq: cell?.dataset.sq, drawn: outline, x: cell?.getBoundingClientRect().x, y: cell?.getBoundingClientRect().y };
};

/**
 * Kursor keyboard ke setiap sel (ular dari pojok kiri atas layar) tanpa
 * menggeser sel mana pun. Urutan petak dibaca dari posisi di layar, jadi
 * berlaku juga untuk papan terbalik.
 */
export async function sweepKeyboard(s, base) {
	const grid = await s.find('css selector', '[role="grid"]');
	await s.exec(() => document.querySelector('[role="grid"]').focus());
	for (let i = 0; i < 8; i++) await s.type(grid, KEYS.up + KEYS.left);
	// Urutan tampilan: baris dari atas, kolom dari kiri.
	const byY = [...new Set(Object.values(base).map((c) => c[1]))].sort((a, b) => a - b);
	const byX = [...new Set(Object.values(base).map((c) => c[0]))].sort((a, b) => a - b);
	const at = (row, col) => SQUARES.find((sq) => base[sq][1] === byY[row] && base[sq][0] === byX[col]);
	let visited = 0;
	for (let r = 0; r < 8; r++) {
		for (let k = 0; k < 8; k++) {
			const col = r % 2 === 0 ? k : 7 - k;
			const want = at(r, col);
			const now = await s.exec(cursorAt);
			if (now.sq !== want) fail(`kursor di ${now.sq}, seharusnya ${want}`);
			if (!now.drawn) fail(`kursor di ${want} tidak tergambar sebagai garis tepi`);
			sameCells(base, await s.exec(readCells), `kursor di ${want}`);
			visited++;
			if (k < 7) await s.type(grid, r % 2 === 0 ? KEYS.right : KEYS.left);
		}
		if (r < 7) await s.type(grid, KEYS.down);
	}
	return visited;
}

/** Hover setiap sel tanpa menggeser sel mana pun. */
export async function sweepHover(s, base) {
	const hovered = () =>
		[...document.querySelectorAll('[role="gridcell"]')].filter((c) => c.matches(':hover')).map((c) => c.dataset.sq);
	for (const sq of SQUARES) {
		await s.hover(await s.find('css selector', `[data-sq="${sq}"]`));
		const h = await s.exec(hovered);
		if (h.length !== 1 || h[0] !== sq) fail(`hover di ${sq} terbaca ${JSON.stringify(h)}`);
		sameCells(base, await s.exec(readCells), `hover di ${sq}`);
	}
	return SQUARES.length;
}

/** Kembali ke menu utama dengan pengaturan uji (tanpa boot, bahasa Indonesia). */
export async function resetToMenu(s) {
	await s.exec(() => {
		localStorage.setItem(
			'kyusin.settings.v2',
			JSON.stringify({ bootMode: 'off', lang: 'id', theme: 'p1', intensity: 30 })
		);
		// Penanda di dokumen lama: hilang setelah muat ulang benar-benar terjadi.
		window.__kyusinReset = true;
		location.reload();
	});
	// Tunggu dokumen baru dan daftar game (dimuat asinkron dari backend),
	// bukan sekadar kerangka menu.
	await s.waitFor(
		() =>
			!window.__kyusinReset &&
			!!document.querySelector('.shell main .menu') &&
			[...document.querySelectorAll('.shell main .menu button')].some((b) => b.textContent.includes('Catur')),
		'menu utama',
		15000
	);
}

/** Ganti tema lewat konsol (perintah teks yang sama dengan pemain). */
export async function theme(s, name) {
	const body = await s.find('css selector', 'body');
	await s.type(body, ':');
	const input = await s.find('css selector', '.console input');
	await s.type(input, `theme ${name}${KEYS.enter}${KEYS.escape}`);
	await s.waitFor((t) => document.documentElement.dataset.theme === t, `tema ${name}`, 5000, name);
}
