/**
 * Pengaturan tampilan (SPEC §4). Disimpan di localStorage jendela sampai
 * profil SQLite ada di M3 (D-018).
 */

export type Theme = 'p1' | 'p3' | 'p4';
export type Effect = 'scanline' | 'glow' | 'curve' | 'flicker';
export type Lang = 'id' | 'en';

export const THEMES: Theme[] = ['p1', 'p3', 'p4'];
export const EFFECTS: Effect[] = ['scanline', 'glow', 'curve', 'flicker'];
export const LANGS: Lang[] = ['id', 'en'];

/** Efek yang bergerak; hanya ini yang dimatikan reduced motion (D-026). */
export const MOVING_EFFECTS: Effect[] = ['flicker'];

export const INTENSITY_STEP = 10;

export interface Settings {
	theme: Theme;
	fx: Record<Effect, boolean>;
	/** Intensitas efek CRT 0–100 (D-025). */
	intensity: number;
	/** Konsol perintah selalu tampil (bawaan: mati). */
	consoleAlways: boolean;
	/** Urutan boot saat aplikasi dibuka. */
	boot: boolean;
	/** `null` = ikuti bahasa sistem (D-027). */
	lang: Lang | null;
}

const KEY = 'kyusin.settings.v2';

export const DEFAULTS: Settings = {
	theme: 'p1',
	fx: { scanline: true, glow: true, curve: true, flicker: false },
	intensity: 30,
	consoleAlways: false,
	boot: true,
	lang: null
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

export function setIntensity(value: number) {
	settings.intensity = Math.max(0, Math.min(100, Math.round(value)));
	save();
}

/** Sistem meminta reduced motion. */
export const motion = $state({ reduced: false });

if (typeof window !== 'undefined') {
	const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
	motion.reduced = mq.matches;
	mq.addEventListener('change', (e) => (motion.reduced = e.matches));
}

/** Efek ini dipaksa mati oleh reduced motion, apa pun pengaturannya. */
export function forcedOff(effect: Effect): boolean {
	return motion.reduced && MOVING_EFFECTS.includes(effect);
}

/** Keadaan efek yang sebenarnya berlaku di layar. */
export function effectOn(effect: Effect): boolean {
	return settings.fx[effect] && !forcedOff(effect);
}
