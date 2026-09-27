/**
 * Pengaturan tampilan (SPEC §4). Disimpan di localStorage jendela sampai
 * profil SQLite ada di M3 (D-016).
 */

export type Theme = 'p1' | 'p3' | 'p4';
export type Effect = 'scanline' | 'glow' | 'curve' | 'flicker';

export const THEMES: { id: Theme; label: string }[] = [
	{ id: 'p1', label: 'Hijau P1' },
	{ id: 'p3', label: 'Amber P3' },
	{ id: 'p4', label: 'Putih P4' }
];

export const EFFECTS: { id: Effect; label: string }[] = [
	{ id: 'scanline', label: 'Scanline' },
	{ id: 'glow', label: 'Glow' },
	{ id: 'curve', label: 'Lengkungan layar' },
	{ id: 'flicker', label: 'Flicker' }
];

export interface Settings {
	theme: Theme;
	fx: Record<Effect, boolean>;
	/** Konsol perintah selalu tampil (bawaan: mati). */
	consoleAlways: boolean;
	/** Urutan boot saat aplikasi dibuka. */
	boot: boolean;
}

const KEY = 'kyusin.settings.v1';

export const DEFAULTS: Settings = {
	theme: 'p1',
	fx: { scanline: true, glow: true, curve: true, flicker: false },
	consoleAlways: false,
	boot: true
};

function load(): Settings {
	try {
		const raw = localStorage.getItem(KEY);
		if (!raw) return structuredClone(DEFAULTS);
		const saved = JSON.parse(raw) as Partial<Settings>;
		return {
			...DEFAULTS,
			...saved,
			fx: { ...DEFAULTS.fx, ...(saved.fx ?? {}) }
		};
	} catch {
		return structuredClone(DEFAULTS);
	}
}

export const settings: Settings = $state(load());

export function save() {
	try {
		localStorage.setItem(KEY, JSON.stringify(settings));
	} catch {
		// Penyimpanan tidak tersedia; pengaturan berlaku untuk sesi ini saja.
	}
}

/** Sistem meminta reduced motion: semua efek CRT mati otomatis. */
export const motion = $state({ reduced: false });

if (typeof window !== 'undefined') {
	const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
	motion.reduced = mq.matches;
	mq.addEventListener('change', (e) => (motion.reduced = e.matches));
}

export function effectOn(effect: Effect): boolean {
	return !motion.reduced && settings.fx[effect];
}
