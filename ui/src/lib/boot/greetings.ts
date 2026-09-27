/**
 * Kumpulan sapaan untuk mode boot "Sapaan" (SPEC §4, D-031).
 *
 * Setiap sapaan punya syarat atas data lokal. Data yang belum ada (profil,
 * game terakhir, chip: M3/M4) bernilai `undefined`, dan sapaan yang
 * membutuhkannya dilewati otomatis; begitu backend mengirim datanya,
 * sapaan itu ikut aktif tanpa perubahan di sini. Tanpa AI; tidak ada data
 * yang keluar dari perangkat. Teksnya ada di berkas terjemahan
 * (`greet.<id>`), jadi modul ini tidak bergantung pada Svelte dan bisa
 * diuji dengan `node --test`.
 */

export interface GreetingContext {
	/** Jam sistem, 0–23. */
	hour: number;
	/** Hari dalam minggu, 0 = Minggu … 6 = Sabtu. */
	weekday: number;
	/** Jumlah hari kalender berturut-turut dengan sesi, termasuk hari ini. */
	streak: number;
	/** Jeda sejak sesi terakhir dalam milidetik; `null` = sesi pertama. */
	gapMs: number | null;
	/** Nama profil (M3). */
	profile?: string;
	/** Nama tampilan game terakhir (M3). */
	lastGame?: string;
	/** Saldo chip profil (M4). */
	chips?: number;
}

type Data = 'profile' | 'lastGame' | 'chips';

interface Greeting {
	id: string;
	/** Makin tinggi makin didahulukan; cadangan umum = 0. */
	priority: number;
	/** Data yang wajib ada; tanpa data itu sapaan dilewati. */
	needs?: Data[];
	when?: (c: GreetingContext) => boolean;
	params?: (c: GreetingContext) => Record<string, string | number>;
}

const HOUR = 3_600_000;
const DAY = 24 * HOUR;

export const GREETINGS: Greeting[] = [
	{ id: 'first', priority: 4, when: (c) => c.gapMs === null },
	{
		id: 'long_absence',
		priority: 3,
		when: (c) => c.gapMs !== null && c.gapMs >= 7 * DAY,
		params: (c) => ({ days: Math.floor((c.gapMs ?? 0) / DAY) })
	},
	{ id: 'back_quick', priority: 3, when: (c) => c.gapMs !== null && c.gapMs < 10 * 60_000 },
	{ id: 'late_night', priority: 2, when: (c) => c.hour >= 0 && c.hour < 4 },
	{ id: 'early', priority: 2, when: (c) => c.hour >= 4 && c.hour < 7 },
	{ id: 'lunch', priority: 1, when: (c) => c.hour >= 12 && c.hour < 14 },
	{ id: 'evening', priority: 1, when: (c) => c.hour >= 18 && c.hour < 22 },
	{ id: 'weekend', priority: 1, when: (c) => c.weekday === 0 || c.weekday === 6 },
	{ id: 'streak', priority: 2, when: (c) => c.streak >= 3, params: (c) => ({ n: c.streak }) },
	{
		id: 'chips_low',
		priority: 2,
		needs: ['chips'],
		when: (c) => (c.chips ?? 0) < 1000,
		params: (c) => ({ chips: c.chips ?? 0 })
	},
	{
		id: 'chips_high',
		priority: 2,
		needs: ['chips'],
		when: (c) => (c.chips ?? 0) >= 50_000,
		params: (c) => ({ chips: c.chips ?? 0 })
	},
	{
		id: 'last_game',
		priority: 1,
		needs: ['lastGame'],
		params: (c) => ({ game: c.lastGame ?? '' })
	},
	{ id: 'name', priority: 1, needs: ['profile'], params: (c) => ({ name: c.profile ?? '' }) },
	{ id: 'generic_1', priority: 0 },
	{ id: 'generic_2', priority: 0 },
	{ id: 'generic_3', priority: 0 },
	{ id: 'generic_4', priority: 0 },
	{ id: 'generic_5', priority: 0 },
	{ id: 'generic_6', priority: 0 },
	{ id: 'generic_7', priority: 0 }
];

export function greetingKey(id: string): `greet.${string}` {
	return `greet.${id}`;
}

export interface PickedGreeting {
	id: string;
	key: `greet.${string}`;
	params: Record<string, string | number>;
}

function available(g: Greeting, c: GreetingContext): boolean {
	const has: Record<Data, boolean> = {
		profile: c.profile !== undefined && c.profile !== '',
		lastGame: c.lastGame !== undefined && c.lastGame !== '',
		chips: c.chips !== undefined
	};
	return (g.needs ?? []).every((d) => has[d]) && (g.when?.(c) ?? true);
}

/**
 * Memilih sapaan: yang syaratnya terpenuhi, bukan sapaan sesi sebelumnya,
 * dengan prioritas tertinggi; bila beberapa setara, dipilih acak.
 */
export function pickGreeting(
	c: GreetingContext,
	previousId: string | null,
	random: () => number = Math.random
): PickedGreeting {
	const eligible = GREETINGS.filter((g) => g.id !== previousId && available(g, c));
	const top = Math.max(...eligible.map((g) => g.priority));
	const best = eligible.filter((g) => g.priority === top);
	const g = best[Math.min(best.length - 1, Math.floor(random() * best.length))];
	return { id: g.id, key: greetingKey(g.id), params: g.params?.(c) ?? {} };
}
