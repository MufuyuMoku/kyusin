/**
 * Papan taruhan casino dadu dan roda (SPEC §6.4; M6a): satu komponen
 * (`PapanTable`) untuk Roulette Eropa/Amerika, Sic Bo, Chuck-a-luck, Big
 * Six, Fan-Tan, dan Craps, digerakkan model per game dari `view_data`.
 * Aturan dan bayaran tetap di Rust; UI hanya membaca view dan mengirim
 * perintah teks (`bet <tempat> <n>`, `clear`, `spin`/`roll`/`open`,
 * `take <tempat>`).
 *
 * Setiap tempat taruhan adalah sel berukuran tetap: label di kiri, jumlah
 * taruhan di kanan, jadi taruhan baru tidak menggeser apa pun (SPEC §4).
 */

import { t, type Key } from '$lib/i18n.svelte';
import type { MejaView } from './meja';
import { RED, range } from './roulette.ts';

export { RED, ROULETTE_ROWS, combined, rouletteCoverage, rouletteSpots } from './roulette.ts';

export interface Cell {
	spot: string;
	label: string;
}

export interface Group {
	id: string;
	label: string;
	/** Jumlah kolom sel. */
	cols: number;
	/** Lebar sel (ch). */
	width: number;
	cells: Cell[];
}

export interface PapanModel {
	/** Kata kerja ronde: `spin`, `roll`, `open`. */
	verb: string;
	/** Dadu hasil terakhir (Sic Bo, Chuck-a-luck, Craps). */
	dice: number[] | null;
	/** Jumlah slot dadu tetap (0 untuk roda dan Fan-Tan). */
	dieSlots: number;
	/** Hasil terakhir dalam satu baris (kantong, simbol, kancing, total). */
	face: string;
	groups: Group[];
	/** Label tiap tempat (juga yang tidak punya sel, misalnya split). */
	label: (spot: string) => string;
	roulette: { american: boolean } | null;
	craps: boolean;
}

type V = MejaView & Record<string, any>;


// ---------------------------------------------------------------- roulette

function rouletteLabel(spot: string): string {
	const [kind, ...rest] = spot.split('-');
	switch (kind) {
		case 'straight':
			return rest[0];
		case 'split':
		case 'street':
		case 'trio':
		case 'corner':
			return `${t(`pp.r.${kind}` as Key)} ${rest.join('/')}`;
		case 'line':
			return `${t('pp.r.line')} ${rest[0]}–${rest[1]}`;
		case 'dozen':
			return `${12 * (Number(rest[0]) - 1) + 1}–${12 * Number(rest[0])}`;
		case 'column':
			return t('pp.r.column', { n: rest[0] });
		case 'low':
			return '1–18';
		case 'high':
			return '19–36';
		default:
			return t(`pp.r.${kind}` as Key);
	}
}

function pocketText(p: string): string {
	const colour = p === '0' || p === '00' ? 'green' : RED.includes(Number(p)) ? 'red' : 'black';
	return t('pp.r.pocket', { n: p, colour: t(`pp.r.colour.${colour}` as Key) });
}

function roulette(american: boolean) {
	return (v: V): PapanModel => ({
		verb: 'spin',
		dice: null,
		dieSlots: 0,
		face: typeof v.hasil === 'string' ? pocketText(v.hasil) : '',
		groups: [
			{
				id: 'luar',
				label: '',
				cols: 3,
				width: 18,
				cells: ['dozen-1', 'dozen-2', 'dozen-3'].map((spot) => ({ spot, label: rouletteLabel(spot) }))
			},
			{
				id: 'genap',
				label: '',
				cols: american ? 4 : 3,
				width: american ? 13 : 18,
				cells: ['low', 'even', 'odd', ...(american ? ['topline'] : [])].map((spot) => ({ spot, label: rouletteLabel(spot) }))
			},
			{
				id: 'warna',
				label: '',
				cols: 3,
				width: 18,
				cells: ['red', 'black', 'high'].map((spot) => ({ spot, label: rouletteLabel(spot) }))
			}
		],
		label: rouletteLabel,
		roulette: { american },
		craps: false
	});
}

// ------------------------------------------------------------- dadu tiga

function dice3(v: V): number[] | null {
	return Array.isArray(v.hasil) ? (v.hasil as number[]) : null;
}

function sicBoLabel(spot: string): string {
	const [kind, ...rest] = spot.split('-');
	if (kind === 'total') return rest[0];
	if (kind === 'combo') return rest.join('-');
	if (kind === 'double') return `${rest[0]}-${rest[0]}`;
	if (kind === 'triple') return `${rest[0]}-${rest[0]}-${rest[0]}`;
	if (kind === 'single') return rest[0];
	return t(`pp.sb.${spot.replace('-', '_')}` as Key);
}

function totalText(d: number[] | null): string {
	return d ? t('pp.dice_total', { n: d.reduce((a, b) => a + b, 0) }) : '';
}

const sicBo = (v: V): PapanModel => {
	const cells = (spots: string[]) => spots.map((spot) => ({ spot, label: sicBoLabel(spot) }));
	const combos: string[] = [];
	for (const a of range(1, 6)) for (const b of range(a + 1, 6)) combos.push(`combo-${a}-${b}`);
	return {
		verb: 'roll',
		dice: dice3(v),
		dieSlots: 3,
		face: totalText(dice3(v)),
		groups: [
			{ id: 'utama', label: '', cols: 5, width: 12, cells: cells(['small', 'big', 'odd', 'even', 'any-triple']) },
			{ id: 'total', label: t('pp.g.total'), cols: 7, width: 8, cells: cells(range(4, 17).map((n) => `total-${n}`)) },
			{ id: 'kombinasi', label: t('pp.g.combo'), cols: 5, width: 9, cells: cells(combos) },
			{ id: 'double', label: t('pp.g.double'), cols: 6, width: 9, cells: cells(range(1, 6).map((n) => `double-${n}`)) },
			{ id: 'triple', label: t('pp.g.triple'), cols: 3, width: 11, cells: cells(range(1, 6).map((n) => `triple-${n}`)) },
			{ id: 'angka', label: t('pp.g.single'), cols: 6, width: 8, cells: cells(range(1, 6).map((n) => `single-${n}`)) }
		],
		label: sicBoLabel,
		roulette: null,
		craps: false
	};
};

const chuck = (v: V): PapanModel => {
	const cells = (spots: string[]) => spots.map((spot) => ({ spot, label: sicBoLabel(spot) }));
	return {
		verb: 'roll',
		dice: dice3(v),
		dieSlots: 3,
		face: totalText(dice3(v)),
		groups: [
			{ id: 'angka', label: t('pp.g.single'), cols: 6, width: 8, cells: cells(range(1, 6).map((n) => `single-${n}`)) },
			{ id: 'utama', label: '', cols: 3, width: 16, cells: cells(['small', 'big', 'any-triple']) }
		],
		label: sicBoLabel,
		roulette: null,
		craps: false
	};
};

// ---------------------------------------------------------------- Big Six

const SYMBOLS = ['1', '2', '5', '10', '20', 'joker', 'logo'];
const bigSixLabel = (s: string) => (/^\d+$/.test(s) ? s : t(`pp.b6.${s}` as Key));

const bigSix = (v: V): PapanModel => ({
	verb: 'spin',
	dice: null,
	dieSlots: 0,
	face: typeof v.hasil === 'string' ? t('pp.b6.stop', { s: bigSixLabel(v.hasil) }) : '',
	groups: [
		{ id: 'angka', label: '', cols: 5, width: 9, cells: SYMBOLS.slice(0, 5).map((spot) => ({ spot, label: bigSixLabel(spot) })) },
		{ id: 'bonus', label: '', cols: 2, width: 13, cells: SYMBOLS.slice(5).map((spot) => ({ spot, label: bigSixLabel(spot) })) }
	],
	label: bigSixLabel,
	roulette: null,
	craps: false
});

// ---------------------------------------------------------------- Fan-Tan

function fanTanLabel(spot: string): string {
	const [kind, ...n] = spot.split('-');
	if (kind === 'fan') return n[0];
	if (kind === 'nim') return `${n[0]}/${n[1]}`;
	if (kind === 'tan') return `${n[0]}-${n[1]}/${n[2]}`;
	return n.join('-');
}

const fanTan = (v: V): PapanModel => {
	const nim: string[] = [];
	for (const a of range(1, 4)) for (const b of range(1, 4)) if (a !== b) nim.push(`nim-${a}-${b}`);
	const tan: string[] = [];
	for (const a of range(1, 4)) for (const b of range(a + 1, 4)) for (const c of range(1, 4)) if (c !== a && c !== b) tan.push(`tan-${a}-${b}-${c}`);
	const ssh = [4, 3, 2, 1].map((skip) => `ssh-${range(1, 4).filter((n) => n !== skip).join('-')}`);
	const cells = (spots: string[]) => spots.map((spot) => ({ spot, label: fanTanLabel(spot) }));
	return {
		verb: 'open',
		dice: null,
		dieSlots: 0,
		face: typeof v.hasil === 'number' ? t('pp.ft.result', { beans: v.hasil, n: v.angka ?? '' }) : '',
		groups: [
			{ id: 'fan', label: t('pp.ft.fan'), cols: 4, width: 9, cells: cells(range(1, 4).map((n) => `fan-${n}`)) },
			{ id: 'kwok', label: t('pp.ft.kwok'), cols: 4, width: 9, cells: cells(['kwok-1-2', 'kwok-2-3', 'kwok-3-4', 'kwok-1-4']) },
			{ id: 'ssh', label: t('pp.ft.ssh'), cols: 4, width: 11, cells: cells(ssh) },
			{ id: 'nim', label: t('pp.ft.nim'), cols: 6, width: 9, cells: cells(nim) },
			{ id: 'tan', label: t('pp.ft.tan'), cols: 4, width: 11, cells: cells(tan) }
		],
		label: fanTanLabel,
		roulette: null,
		craps: false
	};
};

// ------------------------------------------------------------------ Craps

export const POINTS = [4, 5, 6, 8, 9, 10];

function crapsLabel(spot: string): string {
	const m = /^(.*?)-?(\d+)?$/.exec(spot) ?? [];
	const base = m[1] ?? spot;
	const n = m[2] ?? '';
	if (n && base !== '') return t(`pp.cr.${base.replace(/-/g, '_')}_n` as Key, { n });
	return t(`pp.cr.${spot.replace(/-/g, '_')}` as Key);
}

const craps = (v: V): PapanModel => {
	const cells = (spots: string[]) => spots.map((spot) => ({ spot, label: crapsLabel(spot) }));
	const dice = Array.isArray(v.dadu) ? (v.dadu as number[]) : null;
	return {
		verb: 'roll',
		dice,
		dieSlots: 2,
		face: [dice ? totalText(dice) : '', v.titik ? t('pp.cr.point_on', { p: v.titik }) : t('pp.cr.comeout')].filter(Boolean).join(' · '),
		groups: [
			{ id: 'garis', label: '', cols: 3, width: 16, cells: cells(['pass', 'dont-pass', 'field']) },
			{ id: 'come', label: '', cols: 2, width: 16, cells: cells(['come', 'dont-come']) },
			{ id: 'odds', label: '', cols: 2, width: 16, cells: cells(['odds-pass', 'odds-dont-pass']) },
			{ id: 'place', label: t('pp.g.place'), cols: 6, width: 8, cells: POINTS.map((n) => ({ spot: `place-${n}`, label: String(n) })) },
			{ id: 'hard', label: t('pp.g.hard'), cols: 4, width: 8, cells: [4, 6, 8, 10].map((n) => ({ spot: `hard-${n}`, label: String(n) })) },
			{ id: 'come-n', label: t('pp.g.come'), cols: 6, width: 8, cells: POINTS.map((n) => ({ spot: `come-${n}`, label: String(n) })) },
			{ id: 'odds-come', label: t('pp.g.odds_come'), cols: 6, width: 8, cells: POINTS.map((n) => ({ spot: `odds-come-${n}`, label: String(n) })) },
			{ id: 'dont-come-n', label: t('pp.g.dont_come'), cols: 6, width: 8, cells: POINTS.map((n) => ({ spot: `dont-come-${n}`, label: String(n) })) },
			{ id: 'odds-dont-come', label: t('pp.g.lay_come'), cols: 6, width: 8, cells: POINTS.map((n) => ({ spot: `odds-dont-come-${n}`, label: String(n) })) }
		],
		label: crapsLabel,
		roulette: null,
		craps: true
	};
};

const MODELS: Record<string, (v: V) => PapanModel> = {
	'roulette-eropa': roulette(false),
	'roulette-amerika': roulette(true),
	'sic-bo': sicBo,
	'chuck-a-luck': chuck,
	'big-six': bigSix,
	'fan-tan': fanTan,
	craps
};

export function papanModel(game: string, v: MejaView): PapanModel | null {
	const make = MODELS[game];
	return make ? make(v as V) : null;
}

/** Tempat taruhan Craps yang boleh dipasang di lemparan come-out (ronde baru). */
export function crapsComeOut(spot: string): boolean {
	return /^(pass|dont-pass|field|place-\d+|hard-\d+)$/.test(spot);
}
