// Tes jendela asli meja antar-pemain M5b (SPEC §4, §7, §11; D-063, D-065):
// Texas Hold'em, Omaha, Teen Patti, Domino QiuQiu melawan bot level 1. Untuk setiap game:
//  1. duduk: buy-in 2.000 dipindahkan dari saldo (D-059);
//  2. keselarasan: kotak kartu kursi lawan sejajar per kolom kisi (paling
//     banyak dua kolom), kotak kartu kursimu sejajar dengan kartu meja, kartu
//     44×60, geser kartu tetap (18 px; kartu meja poker 48 px);
//  3. meja dan semua kontrol aksi muat di jendela bawaan tanpa gulir (SPEC §4
//     Rev. 13); hover dan kursor keyboard ke setiap kontrol tidak menggeser
//     apa pun;
//  4. satu tangan dimainkan sampai selesai lewat kontrol visual; kartu baru
//     tidak menggeser kotak kursi;
//  5. berdiri: tumpukan kembali ke saldo, verify cocok, rating lokal tampil;
//  6. tangkapan layar.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { changedSince, fail, inView, resetToMenu, tableSig, theme } from './helpers.mjs';
import { KEYS } from './webdriver.mjs';

const GAMES = [
	{ id: 'texas-holdem', name: "Texas Hold'em", passive: ['[ CHECK ]', '[ CALL · '] },
	{ id: 'omaha', name: 'Omaha (Pot-Limit)', passive: ['[ CHECK ]', '[ CALL · '] },
	{ id: 'teen-patti', name: 'Teen Patti', passive: ['[ SHOW · ', '[ CHAAL · '] },
	{ id: 'domino-qiuqiu', name: 'Domino QiuQiu', passive: ['[ CHECK ]', '[ CALL · '] }
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
	return {
		fase: t?.dataset.fase,
		text: t?.innerText ?? '',
		chips: Number((/Chip: (\d+)/.exec(t?.innerText ?? '') ?? [])[1] ?? NaN),
		buttons: [...document.querySelectorAll('.table .controls button')].map((b) => ({
			label: b.textContent.trim(),
			on: b.getAttribute('aria-disabled') !== 'true'
		}))
	};
};

async function checkAlignment(s, id) {
	const a = await s.exec(() => {
		const left = (q) => [...document.querySelectorAll(q)].map((h) => Math.round(h.getBoundingClientRect().left));
		const seats = left('.table .others [data-row^="kursi"]');
		const mine = left('.table .me [data-row], .table [data-row="meja"]');
		const cards = [...document.querySelectorAll('.table .card')].map((c) => {
			const b = c.getBoundingClientRect();
			return { x: b.x, w: b.width, h: b.height, parent: c.parentElement };
		});
		const offsets = [];
		for (let i = 1; i < cards.length; i++) if (cards[i].parent === cards[i - 1].parent) offsets.push(Math.round(cards[i].x - cards[i - 1].x));
		return { seats, mine, sizes: cards.map((c) => `${Math.round(c.w)}x${Math.round(c.h)}`), offsets };
	});
	if (new Set(a.seats).size > 2) fail(`${id}: kotak kursi lawan tidak sejajar per kolom: ${a.seats}`);
	if (new Set(a.mine).size > 1) fail(`${id}: kotak kursimu tidak sejajar dengan kartu meja: ${a.mine}`);
	if (a.sizes.some((z) => z !== '44x60')) fail(`${id}: ukuran kartu ${a.sizes}`);
	if (a.offsets.some((o) => o !== 18 && o !== 48)) fail(`${id}: geser kartu ${a.offsets}`);
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
	await s.exec(() => document.querySelector('.table .controls button')?.focus());
	for (let i = 0; i < buttons; i++) {
		await s.type(await s.find('css selector', ':focus'), KEYS.right);
		same(base, await s.exec(layout), `kursor keyboard langkah ${i + 1} (${when})`);
	}
	return buttons;
}

/**
 * Perekam diagnosis klik yang hilang (dipasang sekali per halaman): event
 * pointer/mouse/click dengan id node dan apakah node masih terpasang,
 * tombol kontrol yang ditambah/dilepas (MutationObserver), dan setiap
 * panggilan IPC ke host (mulai/selesai). Disimpan sebagai cincin 400 baris.
 */
const installRecorder = () => {
	if (window.__e2eRec) return;
	const rec = (window.__e2eRec = { lines: [], ids: new WeakMap(), next: 1, t0: performance.now() });
	const id = (n) => {
		if (!(n instanceof Element)) return String(n);
		if (!rec.ids.has(n)) rec.ids.set(n, rec.next++);
		return `#${rec.ids.get(n)}${n.isConnected ? '' : '(lepas)'}:${n.tagName}:${n.textContent.trim().slice(0, 14)}`;
	};
	const push = (line) => {
		rec.lines.push(`${Math.round(performance.now() - rec.t0)} ${line}`);
		if (rec.lines.length > 400) rec.lines.shift();
	};
	rec.push = push;
	rec.id = id;
	for (const type of ['pointerdown', 'pointerup', 'mousedown', 'mouseup', 'click'])
		document.addEventListener(type, (e) => push(`${type} ${id(e.target)} (${e.clientX},${e.clientY}) d${e.detail}`), true);
	new MutationObserver((list) => {
		for (const m of list) {
			for (const n of m.removedNodes)
				if (n instanceof Element && (n.matches('button') || n.querySelector?.('button')) && m.target.closest?.('.table'))
					push(`dilepas ${id(n.matches('button') ? n : n.querySelector('button'))} dari ${m.target.className}`);
			for (const n of m.addedNodes)
				if (n instanceof Element && (n.matches('button') || n.querySelector?.('button')) && m.target.closest?.('.table'))
					push(`ditambah ${id(n.matches('button') ? n : n.querySelector('button'))}`);
		}
	}).observe(document.body, { childList: true, subtree: true });
	// IPC Tauri 2 lewat fetch ke ipc.localhost (menimpa
	// `__TAURI_INTERNALS__.invoke` tidak berefek).
	const fetch0 = window.fetch.bind(window);
	window.fetch = (input, init) => {
		const url = String(input?.url ?? input);
		if (!url.includes('ipc.localhost') && !url.startsWith('ipc:')) return fetch0(input, init);
		const cmd = decodeURIComponent(url.split('?')[0].split('/').pop());
		push(`ipc mulai ${cmd}`);
		// Tes batas waktu (D-067): host "tidak menjawab".
		if (window.__e2eHang && cmd === 'match_act') return new Promise(() => {});
		return fetch0(input, init).then(
			(r) => (push(`ipc selesai ${cmd} ${r.status}`), r),
			(e) => (push(`ipc gagal ${cmd}`), Promise.reject(e))
		);
	};
};

let artifactsDir = null;
/** Aksi yang hanya terdeteksi lewat kartu/fase (teks meja tidak berubah). */
let textBlind = 0;

async function press(s, label) {
	await s.exec(installRecorder);
	const before = await s.exec(tableSig);
	const target = await s.find('xpath', `//div[contains(@class,'controls')]//button[normalize-space(.)="${label}"]`);
	await s.exec(
		(label) => {
			const b = [...document.querySelectorAll('.table .controls button')].find((x) => x.textContent.trim() === label);
			window.__e2eTarget = b;
			window.__e2eRec.push(`tes: klik ${window.__e2eRec.id(b)}`);
		},
		label
	);
	await s.click(target);
	try {
		await s.waitFor(changedSince, `meja berubah setelah ${label}`, 15000, before);
		// Bukti D-067: perubahan yang hanya terlihat dari kartu/fase, bukan teks.
		const after = await s.exec(tableSig);
		const text = (sig) => sig.split('|').slice(2).join('|');
		if (text(after) === text(before)) {
			textBlind++;
			console.log(`  bukti D-067: setelah ${label} teks meja sama persis, hanya kartu/fase yang berubah (${before.split('|')[1]} → ${after.split('|')[1]})`);
		}
	} catch (e) {
		const why = await s.exec(() => ({
			target: window.__e2eRec.id(window.__e2eTarget),
			buttons: [...document.querySelectorAll('.table .controls button')].map((b) => `${window.__e2eRec.id(b)}${b.getAttribute('aria-disabled') ? ' (nonaktif)' : ''}`),
			active: window.__e2eRec.id(document.activeElement),
			sending: document.querySelector('.match')?.dataset.act ?? null,
			alert: [...document.querySelectorAll('[role="alert"]')].map((a) => a.textContent.trim()),
			status: document.querySelector('.match [role="status"]')?.textContent.trim(),
			text: document.querySelector('.table')?.innerText,
			events: window.__e2eRec.lines.slice(-120)
		}));
		console.log(`  diagnosis ${label}: ${JSON.stringify(why, null, 1)}`);
		if (artifactsDir) {
			writeFileSync(join(artifactsDir, 'klik-hilang.json'), JSON.stringify(why, null, 1));
			writeFileSync(join(artifactsDir, 'klik-hilang.png'), await s.screenshot());
		}
		throw e;
	}
}

/** Menunggu giliran pemain (kontrol muncul) atau tangan selesai. */
async function waitControls(s) {
	await s.waitFor(() => document.querySelectorAll('.table .controls button').length > 0 || !!document.querySelector('.match .result'), 'giliran pemain', 60000);
}

export async function run(s, artifacts, log) {
	artifactsDir = artifacts;
	// KYUSIN_E2E_GAMES=texas-holdem,omaha: sebagian game saja (workflow ulang).
	const only = process.env.KYUSIN_E2E_GAMES?.split(',').filter(Boolean);
	for (const g of GAMES.filter((x) => !only || only.includes(x.id))) {
		await resetToMenu(s);
		const before = await s.exec(async () => {
			const p = await window.__TAURI_INTERNALS__.invoke('profile_get');
			return p.chips;
		});
		await s.click(await s.find('xpath', `//button[normalize-space(.)="› ${g.name}"]`));
		await s.waitFor(() => !!document.querySelector('.game'), `layar ${g.name}`);
		await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
		const confirm = await s.exec(() => [...document.querySelectorAll('button')].some((b) => b.textContent.trim() === '[ YA, MAIN BARU ]'));
		if (confirm) await s.click(await s.find('xpath', "//button[normalize-space(.)='[ YA, MAIN BARU ]']"));
		await s.waitFor((id) => !!document.querySelector(`.table[data-game="${id}"]`), `meja ${g.id}`, 15000, g.id);
		await waitControls(s);
		const sat = await s.exec(state);
		if (sat.chips !== before - 2000) fail(`${g.id}: buy-in tidak dipotong (${before} → ${sat.chips})`);
		await checkAlignment(s, g.id);
		const controls = await sweepControls(s, `${g.id} giliran`);
		writeFileSync(join(artifacts, `pvp-${g.id}-turn.png`), await s.elementScreenshot(await s.find('css selector', '.left')));

		// Mainkan sampai tangan selesai: pilihan pasif (check/call/show/chaal).
		for (let guard = 0; guard < 40; guard++) {
			const st = await s.exec(state);
			if (st.fase !== 'main') break;
			if (st.buttons.length === 0) {
				await waitControls(s);
				continue;
			}
			const base = await s.exec(layout);
			const pick = st.buttons.find((b) => b.on && g.passive.some((p) => b.label.startsWith(p))) ?? st.buttons.find((b) => b.on);
			await press(s, pick.label);
			same(
				Object.fromEntries(Object.entries(base).filter(([k]) => k.startsWith('row:'))),
				await s.exec(layout),
				`aksi ${pick.label}`
			);
			await waitControls(s);
		}
		await s.waitFor(() => document.querySelector('.table')?.dataset.fase !== 'main', `${g.id}: tangan selesai`, 60000);
		await checkAlignment(s, g.id);
		writeFileSync(join(artifacts, `pvp-${g.id}-hand.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
		const stack = await s.exec(() => {
			const row = document.querySelector('.table [data-seat="0"] .line')?.textContent ?? '';
			return Number((/tumpukan (\d+)/.exec(row) ?? [])[1] ?? NaN);
		});

		// Berdiri: tumpukan kembali ke saldo; verify dan rating.
		if ((await s.exec(state)).fase === 'antara') await press(s, '[ BERDIRI ]');
		await s.waitFor(() => !!document.querySelector('.match .result'), `${g.id}: sesi selesai`, 15000);
		const end = await s.exec(async () => ({
			chips: (await window.__TAURI_INTERNALS__.invoke('profile_get')).chips,
			verify: document.body.innerText.includes('verify: semua cocok'),
			rating: document.querySelector('.match .rating')?.textContent.trim() ?? ''
		}));
		if (end.chips !== before - 2000 + stack) fail(`${g.id}: saldo ${before} → ${end.chips}, padahal tumpukan akhir ${stack}`);
		if (!end.verify) fail(`${g.id}: verify tidak cocok`);
		if (!end.rating) fail(`${g.id}: rating lokal tidak tampil`);
		writeFileSync(join(artifacts, `pvp-${g.id}-end.png`), await s.screenshot());
		log(`${g.name}: buy-in 2000 dipotong, selaras, ${controls} kontrol tanpa geser, satu tangan dimainkan, berdiri dengan ${stack} (saldo ${before} → ${end.chips}), verify cocok, ${end.rating}`);
	}
	if (only) return;
	await resetToMenu(s);
	await s.click(await s.find('xpath', `//button[normalize-space(.)="› Teen Patti"]`));
	await s.waitFor(() => !!document.querySelector('.game'), 'layar Teen Patti');
	await s.click(await s.find('xpath', "//button[normalize-space(.)='[ MAIN ]']"));
	await s.waitFor(() => !!document.querySelector('.table[data-game="teen-patti"]'), 'meja Teen Patti');
	await waitControls(s);
	for (const name of ['p3', 'p4']) {
		await theme(s, name);
		await checkAlignment(s, 'teen-patti');
		writeFileSync(join(artifacts, `pvp-teen-patti-${name}.png`), await s.elementScreenshot(await s.find('css selector', '.left')));
	}
	await theme(s, 'p1');
	log('tema P3 dan P4: meja antar-pemain tetap selaras');
}
