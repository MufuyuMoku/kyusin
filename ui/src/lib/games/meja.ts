/**
 * Meja casino M5a (SPEC §6.3): satu komponen meja (`MejaTable`) untuk
 * sepuluh game melawan bandar, digerakkan model per game dari `view_data`.
 * Aturan tetap di Rust; UI hanya membaca view dan mengirim perintah teks.
 *
 * Setiap baris kartu punya jumlah slot tetap, jadi kartu baru tidak
 * menggeser apa pun (SPEC §4, D-038). Tombol tempat taruhan berlabel tetap;
 * jumlahnya tampil di baris taruhan dengan lebar tetap.
 */

import { t, type Key } from '$lib/i18n.svelte';

/** Bagian view yang sama untuk semua meja (`meja::Umum` di Rust). */
export interface MejaView {
	fase: string;
	bersih: number;
	ronde: number;
	taruhan_meja: number;
	dipertaruhkan: number;
	biaya: Record<string, number>;
	pengali: Record<string, number>;
	min_taruhan: number;
	maks_taruhan: number;
	langkah_taruhan: number;
	selesai: boolean;
	alasan: string | null;
	netral: string | null;
	kartu_terpakai: number | null;
	sisa: number | null;
	potong: number | null;
	// Bidang per game dibaca model masing-masing.
	[key: string]: unknown;
}

export function isMejaView(v: unknown): v is MejaView {
	return !!v && typeof v === 'object' && 'langkah_taruhan' in v && 'fase' in v;
}

export interface CardRow {
	id: string;
	label: string;
	cards: string[];
	/** Jumlah slot kartu tetap untuk baris ini. */
	slots: number;
	/** Kartu tidak bertumpuk (bisa dipilih satu per satu). */
	spread?: boolean;
	line?: string;
	/** Kartu bisa dipilih (Pai Gow: tangan depan). */
	selectable?: boolean;
	/** Jarak ekstra setelah kartu ke-n (Pai Gow: depan | belakang). */
	gapAfter?: number;
}

export interface Spot {
	spot: string;
	label: string;
}

export interface Choice {
	cmd: string;
	label: string;
}

export interface TableModel {
	rows: CardRow[];
	/** `null` = satu taruhan (`bet <jumlah>` langsung membagi). */
	spots: Spot[] | null;
	/** Pilihan di tengah ronde. */
	choices: Choice[];
	/** Ringkasan ronde terakhir. */
	outcome: string;
	/** Pai Gow: jumlah kartu yang dipilih untuk `set`. */
	pick?: number;
	/** Baris berisi ubin domino (Pai Gow ubin), bukan kartu remi. */
	tiles?: boolean;
}

type V = MejaView & Record<string, any>;

const signed = (n: number) => (n > 0 ? `+${n}` : String(n));

/** Hasil bersih ronde terakhir dari peta `bayar`. */
function sum(map: unknown): number | null {
	if (!map || typeof map !== 'object') return null;
	const values = Object.values(map as Record<string, number>);
	return values.length ? values.reduce((a, b) => a + b, 0) : null;
}

function hand(key: unknown): string {
	return typeof key === 'string' ? t(`mj.hand.${key}` as Key) : '';
}

function payLine(map: unknown, labels: Record<string, string>): string {
	if (!map || typeof map !== 'object') return '';
	return Object.entries(map as Record<string, number>)
		.map(([k, v]) => `${labels[k] ?? k} ${signed(v)}`)
		.join(' · ');
}

function outcomeOf(v: V, detail: string): string {
	if (v.ronde === 0) return '';
	const net = sum(v.bayar);
	if (net === null && typeof v.bayar !== 'number') return '';
	const total = typeof v.bayar === 'number' ? v.bayar : (net ?? 0);
	return [detail, t('mj.round_net', { n: signed(total) })].filter(Boolean).join(' · ');
}

function choice(cmd: string): Choice {
	return { cmd, label: t(`mj.cmd.${cmd}` as Key) };
}

const MODELS: Record<string, (v: V) => TableModel> = {
	'dragon-tiger': (v) => {
		const labels = { dragon: t('mj.dt.dragon'), tiger: t('mj.dt.tiger'), tie: t('mj.dt.tie') };
		const win = v.pemenang as string | null;
		return {
			rows: [
				{ id: 'naga', label: labels.dragon, cards: v.naga ? [v.naga] : [], slots: 1, line: win === 'naga' ? t('mj.wins') : '' },
				{ id: 'macan', label: labels.tiger, cards: v.macan ? [v.macan] : [], slots: 1, line: win === 'macan' ? t('mj.wins') : '' }
			],
			spots: [
				{ spot: 'dragon', label: labels.dragon },
				{ spot: 'tiger', label: labels.tiger },
				{ spot: 'tie', label: labels.tie }
			],
			choices: [],
			outcome: outcomeOf(v, [win === 'seri' ? t('mj.tie_result') : '', payLine(v.bayar, labels)].filter(Boolean).join(' · '))
		};
	},
	'casino-war': (v) => {
		const labels = { ante: t('mj.spot.ante'), tie: t('mj.war.tie'), raise: t('mj.spot.raise') };
		return {
			rows: [
				{ id: 'bandar', label: t('mj.dealer'), cards: v.bandar, slots: 2 },
				{ id: 'pemain', label: t('mj.you'), cards: v.pemain, slots: 2, line: v.dibuang ? t('mj.war.burned', { n: v.dibuang }) : '' }
			],
			spots: [
				{ spot: 'ante', label: labels.ante },
				{ spot: 'tie', label: labels.tie }
			],
			choices: v.fase === 'perang' ? [choice('war'), choice('surrender')] : [],
			outcome: v.fase === 'perang' ? '' : outcomeOf(v, payLine(v.bayar, labels))
		};
	},
	'andar-bahar': (v) => {
		const labels = { andar: t('mj.ab.andar'), bahar: t('mj.ab.bahar') };
		const pile = (cards: string[], side: string) => ({
			id: side,
			label: labels[side as 'andar' | 'bahar'],
			cards: cards.slice(-7),
			slots: 7,
			line:
				(cards.length ? t('mj.ab.count', { n: cards.length }) : '') +
				(v.pemenang === side ? ` · ${t('mj.wins')}` : '')
		});
		return {
			rows: [
				{ id: 'tengah', label: t('mj.ab.middle'), cards: v.tengah ? [v.tengah] : [], slots: 1 },
				pile(v.andar, 'andar'),
				pile(v.bahar, 'bahar')
			],
			spots: [
				{ spot: 'andar', label: labels.andar },
				{ spot: 'bahar', label: labels.bahar }
			],
			choices: [],
			outcome: outcomeOf(v, payLine(v.bayar, labels))
		};
	},
	baccarat: (v) => {
		const labels = { player: t('mj.bac.player'), banker: t('mj.bac.banker'), tie: t('mj.bac.tie') };
		const total = (n: unknown, side: string) =>
			typeof n === 'number' ? `${t('mj.total', { n })}${v.pemenang === side ? ` · ${t('mj.wins')}` : ''}` : '';
		return {
			rows: [
				{ id: 'pemain', label: labels.player, cards: v.pemain, slots: 3, line: total(v.nilai_pemain, 'player') },
				{ id: 'bankir', label: labels.banker, cards: v.bankir, slots: 3, line: total(v.nilai_bankir, 'banker') }
			],
			spots: [
				{ spot: 'player', label: labels.player },
				{ spot: 'banker', label: labels.banker },
				{ spot: 'tie', label: labels.tie }
			],
			choices: [],
			outcome: outcomeOf(v, [v.pemenang === 'tie' ? t('mj.tie_result') : '', payLine(v.bayar, labels)].filter(Boolean).join(' · '))
		};
	},
	'red-dog': (v) => ({
		rows: [
			{
				id: 'kartu',
				label: t('mj.cards'),
				cards: v.kartu,
				slots: 3,
				line: typeof v.jarak === 'number' ? t('mj.rd.spread', { n: v.jarak, k: v.kali }) : ''
			}
		],
		spots: null,
		choices: v.fase === 'naikkan' ? [choice('raise'), choice('call')] : [],
		outcome: v.fase === 'naikkan' || !v.hasil ? '' : outcomeOf(v, t(`mj.rd.${v.hasil}` as Key))
	}),
	'three-card-poker': (v) => {
		const labels = {
			ante: t('mj.spot.ante'),
			play: t('mj.spot.play'),
			bonus: t('mj.tcp.bonus'),
			pairplus: t('mj.tcp.pairplus')
		};
		return {
			rows: [
				{
					id: 'bandar',
					label: t('mj.dealer'),
					cards: v.bandar,
					slots: 3,
					line: [hand(v.tangan_bandar), v.memenuhi === false ? t('mj.not_qualified') : ''].filter(Boolean).join(' · ')
				},
				{ id: 'pemain', label: t('mj.you'), cards: v.pemain, slots: 3, line: hand(v.tangan_pemain) }
			],
			spots: [
				{ spot: 'ante', label: labels.ante },
				{ spot: 'pairplus', label: labels.pairplus }
			],
			choices: v.fase === 'keputusan' ? [choice('play'), choice('fold')] : [],
			outcome: v.fase === 'keputusan' ? '' : outcomeOf(v, payLine(v.bayar, labels))
		};
	},
	'caribbean-stud': (v) => {
		const labels = { ante: t('mj.spot.ante'), raise: t('mj.spot.raise') };
		return {
			rows: [
				{
					id: 'bandar',
					label: t('mj.dealer'),
					cards: v.bandar,
					slots: 5,
					line: [hand(v.tangan_bandar), v.memenuhi === false ? t('mj.not_qualified') : ''].filter(Boolean).join(' · ')
				},
				{ id: 'pemain', label: t('mj.you'), cards: v.pemain, slots: 5, line: hand(v.tangan_pemain) }
			],
			spots: null,
			choices: v.fase === 'keputusan' ? [choice('raise'), choice('fold')] : [],
			outcome: v.fase === 'keputusan' ? '' : outcomeOf(v, payLine(v.bayar, labels))
		};
	},
	'casino-holdem': (v) => {
		const labels = { ante: t('mj.spot.ante'), call: t('mj.spot.call') };
		return {
			rows: [
				{
					id: 'bandar',
					label: t('mj.dealer'),
					cards: v.bandar,
					slots: 2,
					line: [hand(v.tangan_bandar), v.memenuhi === false ? t('mj.not_qualified') : ''].filter(Boolean).join(' · ')
				},
				{ id: 'meja', label: t('mj.board'), cards: v.meja_kartu, slots: 5 },
				{ id: 'pemain', label: t('mj.you'), cards: v.pemain, slots: 2, line: hand(v.tangan_pemain) }
			],
			spots: null,
			choices: v.fase === 'keputusan' ? [choice('call'), choice('fold')] : [],
			outcome: v.fase === 'keputusan' ? '' : outcomeOf(v, payLine(v.bayar, labels))
		};
	},
	'let-it-ride': (v) => ({
		rows: [
			{ id: 'bersama', label: t('mj.lir.community'), cards: v.bersama, slots: 2 },
			{
				id: 'pemain',
				label: t('mj.you'),
				cards: v.pemain,
				slots: 3,
				line: v.tempat.length ? t('mj.lir.spots', { s: (v.tempat as number[]).join(' / ') }) : ''
			}
		],
		spots: null,
		choices:
			v.fase === 'keputusan'
				? [
						{ cmd: 'pull', label: t('mj.lir.pull', { n: v.keputusan ?? 1 }) },
						{ cmd: 'ride', label: t('mj.cmd.ride') }
					]
				: [],
		outcome: v.fase === 'keputusan' || v.bayar === null ? '' : outcomeOf(v, hand(v.tangan))
	}),
	'pai-gow': (v) => {
		const over = v.fase === 'selesai' && v.ronde > 0;
		const res = (r: unknown) => (typeof r === 'string' ? t(`mj.pg.${r}` as Key) : '');
		return {
			rows: [
				{
					id: 'bandar',
					label: t('mj.dealer'),
					cards: over ? [...v.bandar_depan, ...v.bandar_belakang] : v.bandar,
					slots: 7,
					spread: true,
					gapAfter: over ? 2 : undefined,
					line: over ? t('mj.pg.split', { front: v.bandar_depan.length, back: hand(v.tangan_bandar_belakang) }) : ''
				},
				{
					id: 'pemain',
					label: t('mj.you'),
					cards: over ? [...v.depan, ...v.belakang] : v.pemain,
					slots: 7,
					spread: true,
					selectable: v.fase === 'susun',
					gapAfter: over ? 2 : undefined,
					line: over
						? t('mj.pg.result', { front: res(v.hasil_depan), back: `${hand(v.tangan_belakang)} ${res(v.hasil_belakang)}` })
						: ''
				}
			],
			spots: null,
			choices: v.fase === 'susun' ? [choice('houseway')] : [],
			outcome: over ? outcomeOf(v, '') : '',
			pick: v.fase === 'susun' ? 2 : 0
		};
	}
};

/** Nama tangan Pai Gow ubin: `pair`, `wong`, `gong`, `gee_joon`, atau nilai 0–9. */
function tileHand(key: unknown): string {
	if (typeof key !== 'string') return '';
	return /^\d$/.test(key) ? t('mj.pgu.points', { n: key }) : t(`mj.pgu.${key}` as Key);
}

MODELS['pai-gow-ubin'] = (v) => {
	const over = v.fase === 'selesai' && v.ronde > 0;
	const nama = (v.nama ?? []) as string[];
	const res = (r: unknown) => (typeof r === 'string' ? t(`mj.pg.${r}` as Key) : '');
	const hasil = (v.hasil ?? []) as string[];
	return {
		rows: [
			{
				id: 'bandar',
				label: t('mj.dealer'),
				cards: over ? [...v.bandar_tinggi, ...v.bandar_rendah] : v.bandar,
				slots: 4,
				spread: true,
				gapAfter: over ? 2 : undefined,
				line: over ? t('mj.pgu.hands', { hi: tileHand(nama[2]), lo: tileHand(nama[3]) }) : ''
			},
			{
				id: 'pemain',
				label: t('mj.you'),
				cards: over ? [...v.tinggi, ...v.rendah] : v.pemain,
				slots: 4,
				spread: true,
				selectable: v.fase === 'susun',
				gapAfter: over ? 2 : undefined,
				line: over
					? t('mj.pgu.hands', { hi: `${tileHand(nama[0])} ${res(hasil[0])}`, lo: `${tileHand(nama[1])} ${res(hasil[1])}` })
					: v.fase === 'susun' && Array.isArray(v.saran) && v.saran.length
						? t('mj.pgu.advice', { tiles: (v.saran as string[]).join(' ') })
						: ''
			}
		],
		spots: null,
		choices: v.fase === 'susun' ? [choice('houseway')] : [],
		outcome: over ? outcomeOf(v, '') : '',
		pick: v.fase === 'susun' ? 2 : 0,
		tiles: true
	};
};

export function tableModel(game: string, v: MejaView): TableModel | null {
	const make = MODELS[game];
	return make ? make(v as V) : null;
}

/** Pilihan chip untuk menyusun taruhan, menurut kelipatan meja. */
export function chipSteps(step: number): number[] {
	if (step === 20) return [20, 100, 500];
	if (step === 120) return [120, 600];
	return [10, 50, 100, 500];
}

/** Lebar kartu bertumpuk / terpisah (px). */
export const OFFSET_STACK = 18;
export const OFFSET_SPREAD = 48;
/** Jarak antara tangan depan dan belakang Pai Gow (px); selalu disediakan. */
export const SPLIT_GAP = 16;

/** Posisi kiri kartu ke-`i` di sebuah baris. */
export function cardLeft(row: CardRow, i: number): number {
	const offset = row.spread ? OFFSET_SPREAD : OFFSET_STACK;
	return i * offset + (row.gapAfter !== undefined && i >= row.gapAfter ? SPLIT_GAP : 0);
}

/** Lebar kotak kartu sebuah baris (px, tanpa lebar kartu terakhir). */
export function rowSpan(row: CardRow): number {
	const offset = row.spread ? OFFSET_SPREAD : OFFSET_STACK;
	return (row.slots - 1) * offset + (row.spread && row.slots > 2 ? SPLIT_GAP : 0);
}
