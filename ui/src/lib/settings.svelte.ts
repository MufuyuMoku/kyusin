/**
 * Pengaturan tampilan (SPEC §4). Disimpan di localStorage jendela sampai
 * profil SQLite ada di M3 (D-018).
 */

export type Theme = 'p1' | 'p3' | 'p4';
export type Effect = 'scanline' | 'glow' | 'curve' | 'flicker';
export type Lang = 'id' | 'en';
export type BootMode = 'verbose' | 'cinematic' | 'greeting' | 'off';

export const THEMES: Theme[] = ['p1', 'p3', 'p4'];
export const EFFECTS: Effect[] = ['scanline', 'glow', 'curve', 'flicker'];
export const LANGS: Lang[] = ['id', 'en'];
export const BOOT_MODES: BootMode[] = ['verbose', 'cinematic', 'greeting', 'off'];

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
	/** Mode urutan boot saat aplikasi dibuka (D-031). */
	bootMode: BootMode;
	/** `null` = ikuti bahasa sistem (D-027). */
	lang: Lang | null;
}

const KEY = 'kyusin.settings.v2';

export const DEFAULTS: Settings = {
	theme: 'p1',
	fx: { scanline: true, glow: true, curve: true, flicker: false },
	intensity: 30,
	consoleAlways: false,
	bootMode: 'cinematic',
	lang: null
};

function load(): Settings {
	try {
		const raw = localStorage.getItem(KEY);
		if (!raw) return structuredClone(DEFAULTS);
		const saved = JSON.parse(raw) as Partial<Settings> & { boot?: boolean };
		// Pengaturan lama (M0/M0b) menyimpan toggle `boot`; mati tetap mati.
		const migrated: Partial<Settings> =
			saved.bootMode === undefined && saved.boot === false ? { bootMode: 'off' } : {};
		delete saved.boot;
		return {
			...DEFAULTS,
			...saved,
			...migrated,
			fx: { ...DEFAULTS.fx, ...(saved.fx ?? {}) }
		};
	} catch {
		return structuredClone(DEFAULTS);
	}
}

export const settings: Settings = $state(load());

/** Kapan pengaturan dibaca (untuk boot Verbose). */
export const settingsReadAt = typeof performance === 'undefined' ? 0 : performance.now();

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
