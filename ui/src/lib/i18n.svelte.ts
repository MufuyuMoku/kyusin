/**
 * Terjemahan antarmuka (SPEC §4, D-027). Semua teks UI lewat `t()`;
 * teks dari backend datang sebagai `{ id, en }` dan dipilih dengan `L()`.
 * Kata perintah (`help`, `take`) tidak diterjemahkan.
 */

import en from './i18n/en.json';
import id from './i18n/id.json';
import { settings, type Lang } from './settings.svelte';

export type Key = keyof typeof id;
export type Localized = { id: string; en: string };

const TABLES: Record<Lang, Record<string, string>> = { id, en };

/** Bahasa sistem: Indonesia bila sistem berbahasa Indonesia, selain itu Inggris. */
export function systemLang(): Lang {
	const langs = typeof navigator === 'undefined' ? [] : (navigator.languages ?? [navigator.language]);
	return langs[0]?.toLowerCase().startsWith('id') ? 'id' : 'en';
}

export function lang(): Lang {
	return settings.lang ?? systemLang();
}

export function t(key: Key, params: Record<string, string | number> = {}): string {
	let s = TABLES[lang()][key] ?? key;
	for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, String(v));
	return s;
}

/** Memilih teks backend sesuai bahasa aktif. */
export function L(text: Localized | null | undefined): string {
	return text ? text[lang()] : '';
}

/** Teks kesalahan: backend melempar `{ id, en }`, sisanya apa adanya. */
export function errorText(e: unknown): string {
	if (e && typeof e === 'object' && 'id' in e && 'en' in e) return L(e as Localized);
	return String(e);
}
