// Tes jendela asli layar profil dan statistik (SPEC §9 M3, §11):
//  - profil tunggal: nama diganti lewat kolom isian, tersimpan, dan tetap
//    ada setelah halaman dimuat ulang;
//  - statistik: baris per game yang sudah dimainkan (tes sebelumnya
//    menyelesaikan partai catur dan Reversi), rating lokal tampil, kolom
//    tabel selaras (tepi kiri dan lebar tiap kolom sama di semua baris),
//    tidak meluber; riwayat berisi perubahan rating dan membuka replay;
//  - tangkapan layar di tiga tema.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fail, resetToMenu, theme } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

async function openFromMenu(s, label) {
	await s.click(await s.find('xpath', `//button[normalize-space(.)='› ${label}']`));
}

/** Kolom tabel: tepi kiri dan lebar tiap `data-col` sama di semua baris. */
const tableShape = () => {
	const rows = [...document.querySelectorAll('.stats [role="row"]')];
	const cols = {};
	for (const row of rows) {
		for (const cell of row.querySelectorAll('[data-col]')) {
			const r = cell.getBoundingClientRect();
			(cols[cell.dataset.col] ??= []).push([Math.round(r.left * 10) / 10, Math.round(r.width * 10) / 10]);
		}
	}
	const table = document.querySelector('.stats [role="table"]');
	const frame = table.parentElement;
	return {
		rows: rows.length,
		cols,
		overflow: table.scrollWidth > table.clientWidth + 1 || frame.scrollWidth > frame.clientWidth + 1,
		games: rows.slice(1).map((r) => ({
			game: r.dataset.game,
			rating: r.querySelector('[data-col="rating"]').textContent.trim()
		}))
	};
};

export async function run(s, artifacts, log) {
	await resetToMenu(s);
	await openFromMenu(s, 'Profil');
	await s.waitFor(() => !!document.getElementById('profile-name'), 'layar profil');
	const before = await s.exec(() => document.querySelector('.profile .name').textContent.trim());
	const input = await s.find('css selector', '#profile-name');
	await s.cmd('POST', `/element/${input}/clear`, {});
	await s.type(input, `Mufuyu${KEYS.enter}`);
	await s.waitFor(() => document.querySelector('.profile .name')?.textContent.trim() === 'Mufuyu', 'nama tersimpan');
	writeFileSync(join(artifacts, 'profile-p1.png'), await s.screenshot());
	await resetToMenu(s);
	await openFromMenu(s, 'Profil');
	await s.waitFor(() => document.querySelector('.profile .name')?.textContent.trim() === 'Mufuyu', 'nama tetap setelah muat ulang');
	log(`profil: nama "${before}" → "Mufuyu", tetap setelah muat ulang`);

	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ STATISTIK ]']"));
	await s.waitFor(() => document.querySelectorAll('.stats [role="row"]').length >= 3, 'tabel statistik');
	const shape = await s.exec(tableShape);
	if (shape.overflow) fail('tabel statistik meluber');
	for (const [col, cells] of Object.entries(shape.cols)) {
		const [x0, w0] = cells[0];
		for (const [x, w] of cells) {
			if (Math.abs(x - x0) > 0.5 || Math.abs(w - w0) > 0.5) fail(`kolom ${col} tidak selaras: ${JSON.stringify(cells)}`);
		}
	}
	for (const g of ['catur', 'reversi']) {
		const row = shape.games.find((r) => r.game === g);
		if (!row) fail(`statistik tanpa baris ${g}`);
		if (!/^\d+ ±\s*\d+$/.test(row.rating)) fail(`rating lokal ${g} tidak tampil: "${row.rating}"`);
	}
	log(`statistik: ${shape.games.map((g) => `${g.game} ${g.rating}`).join(', ')}; ${Object.keys(shape.cols).length} kolom selaras, tidak meluber`);

	const hist = await s.exec(() => [...document.querySelectorAll('.stats .hist')].map((h) => h.textContent.trim()));
	if (hist.length < 3) fail(`riwayat hanya ${hist.length} baris`);
	if (!hist.some((h) => h.includes('→'))) fail('riwayat tanpa perubahan rating');
	log(`riwayat: ${hist.length} baris, contoh "${hist[0]}"`);
	const wrap = await s.find('css selector', '.stats');
	writeFileSync(join(artifacts, 'stats-p1.png'), await s.elementScreenshot(wrap));

	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		const again = await s.exec(tableShape);
		if (again.overflow) fail(`tabel meluber di tema ${name}`);
		writeFileSync(join(artifacts, `stats-${name}.png`), await s.elementScreenshot(await s.find('css selector', '.stats')));
	}
	log('tema P3 dan P4: tabel tetap muat');

	await s.click(await s.find('css selector', '.stats .hist button'));
	await s.waitFor(() => document.querySelector('.crumbs')?.textContent.includes('replay'), 'replay dari riwayat');
	log('riwayat membuka replay');
}
