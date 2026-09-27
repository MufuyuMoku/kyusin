/**
 * Naskah tiga mode urutan boot (SPEC §4, D-031). Setiap naskah adalah
 * daftar baris; pemutarnya (Boot.svelte) yang mengatur ketikan dan jeda.
 *
 * Verbose hanya berisi hal yang benar-benar terjadi saat startup, dengan
 * stempel waktu saat langkah itu selesai. Nama produk dan label proyek
 * tidak diterjemahkan; status `[  OK  ]` bergaya log dan juga tidak.
 */

import { isTauri } from '@tauri-apps/api/core';
import { app, fonts, marks } from '$lib/app.svelte';
import { L, lang, t, type Key } from '$lib/i18n.svelte';
import { EFFECTS, effectOn, motion, settings, settingsReadAt } from '$lib/settings.svelte';
import { pickGreeting } from './greetings';
import type { SessionRecord } from './session';

export const PROJECT_LABEL = 'Project Sinners';
export const PRODUCT = 'KyuSin';

export type Status = 'ok' | 'fail' | 'info';

export interface Line {
	text: string;
	/** Diketik huruf demi huruf (kecuali saat reduced motion). */
	typed?: boolean;
	/** Jeda sebelum baris ini, ms. */
	pause?: number;
	/** Gaya tampilan. */
	style?: 'title' | 'logo' | 'log' | 'plain';
	/** Stempel waktu (ms sejak jendela dibuka), untuk Verbose. */
	stamp?: number;
	status?: Status;
}

export interface Script {
	lines: Line[];
	/** Kecepatan ketik, ms per huruf. */
	charMs: number;
	/** Jeda setelah baris terakhir sebelum menu, ms. */
	hold: number;
}

export function stamp(ms: number): string {
	const secs = Math.floor(ms / 1000);
	const micros = Math.floor((ms % 1000) * 1000);
	return `[${String(secs).padStart(5)}.${String(micros).padStart(6, '0')}]`;
}

export function statusTag(s: Status): string {
	return s === 'ok' ? '[  OK  ]' : s === 'fail' ? '[FAILED]' : '[ INFO ]';
}

function cartridgeCount(): number {
	return app.catalog.reduce((n, c) => n + c.games.length, 0);
}

function ago(ms: number): string {
	const min = Math.floor(ms / 60_000);
	if (min < 1) return t('ago.now');
	if (min < 60) return t('ago.minutes', { n: min });
	const h = Math.floor(min / 60);
	if (h < 48) return t('ago.hours', { n: h });
	return t('ago.days', { n: Math.floor(h / 24) });
}

export function verbose(previous: SessionRecord, sessionAt: number): Script {
	const lines: Line[] = [];
	const log = (stampMs: number, status: Status, text: string) =>
		lines.push({ stamp: stampMs, status, text, style: 'log' });

	log(0, 'info', t('verbose.start', { version: app.info?.version ?? '?', label: PROJECT_LABEL }));
	log(settingsReadAt, 'ok', t('verbose.settings'));
	log(settingsReadAt, 'info', t('verbose.theme', { theme: t(`settings.theme.${settings.theme}` as Key) }));
	const on = EFFECTS.filter(effectOn).map((fx) => t(`settings.fx.${fx}` as Key));
	log(
		settingsReadAt,
		'info',
		on.length ? t('verbose.fx', { list: on.join(', '), n: settings.intensity }) : t('verbose.fx.none')
	);
	log(settingsReadAt, 'info', t(motion.reduced ? 'verbose.reduced.on' : 'verbose.reduced.off'));
	const langName = t(`settings.lang.${lang()}` as Key);
	log(
		settingsReadAt,
		'info',
		t(settings.lang ? 'verbose.lang.chosen' : 'verbose.lang.system', { lang: langName })
	);
	log(marks.mount, 'ok', t('verbose.ui'));
	if (marks.info !== null) {
		log(marks.info, 'ok', t(isTauri() ? 'verbose.backend.tauri' : 'verbose.backend.mock'));
		log(
			marks.info,
			'info',
			app.info?.data_dir ? t('verbose.data', { path: app.info.data_dir }) : t('verbose.data.none')
		);
	}
	log(
		sessionAt,
		'info',
		previous.last === null
			? t('verbose.session.first')
			: t('verbose.session.ago', { ago: ago(Date.now() - previous.last) })
	);
	if (marks.catalog !== null) {
		log(marks.catalog, 'ok', t('verbose.registry', { n: cartridgeCount() }));
		for (const g of app.catalog.flatMap((c) => c.games)) {
			log(marks.catalog, 'ok', t('verbose.cartridge', { id: g.id, name: L(g.nama) }));
		}
	} else {
		log(performance.now(), 'fail', t('verbose.registry.fail'));
	}
	for (const f of fonts) log(marks.fonts ?? performance.now(), f.ok ? 'ok' : 'fail', t('verbose.font', { font: f.name }));
	log(performance.now(), 'ok', t('verbose.done'));

	// Urut menurut waktu kejadian yang sebenarnya.
	lines.sort((a, b) => (a.stamp ?? 0) - (b.stamp ?? 0));
	return { lines: lines.map((l) => ({ ...l, pause: 35 })), charMs: 0, hold: 500 };
}

export function cinematic(): Script {
	return {
		charMs: 55,
		hold: 900,
		lines: [
			{ text: PROJECT_LABEL.toUpperCase(), style: 'title', pause: 400 },
			{ text: t('cine.load', { n: cartridgeCount() }), typed: true, pause: 700 },
			{
				text: t('cine.phosphor', {
					theme: t(`settings.theme.${settings.theme}` as Key),
					lang: t(`settings.lang.${lang()}` as Key)
				}),
				typed: true,
				pause: 600
			},
			{ text: t('cine.ready'), typed: true, pause: 800 },
			{ text: PRODUCT, style: 'logo', pause: 900 }
		]
	};
}

export function greeting(previous: SessionRecord): { script: Script; id: string } {
	const profile = app.info?.profile;
	const g = pickGreeting(
		{
			hour: new Date().getHours(),
			gapMs: previous.last === null ? null : Date.now() - previous.last,
			profile: profile?.name,
			lastGame: profile?.last_game,
			chips: profile?.chips
		},
		previous.greeting
	);
	return {
		id: g.id,
		script: {
			charMs: 38,
			hold: 1600,
			lines: [{ text: t(g.key as Key, g.params), typed: true, pause: 500, style: 'plain' }]
		}
	};
}
