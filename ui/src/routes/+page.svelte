<script lang="ts">
	import { app, back, current, load } from '$lib/app.svelte';
	import { effectOn, settings } from '$lib/settings.svelte';
	import { errorText, lang, t, type Key } from '$lib/i18n.svelte';
	import { startSession } from '$lib/boot/session';
	import { NAV_SELECTOR, moveFocus } from '$lib/nav';
	import Boot from '$lib/components/Boot.svelte';
	import Console from '$lib/components/Console.svelte';
	import GameScreen from '$lib/components/GameScreen.svelte';
	import HelpScreen from '$lib/components/HelpScreen.svelte';
	import MatchScreen from '$lib/components/MatchScreen.svelte';
	import MenuScreen from '$lib/components/MenuScreen.svelte';
	import ReplayScreen from '$lib/components/ReplayScreen.svelte';
	import SettingsScreen from '$lib/components/SettingsScreen.svelte';
	import TutorialScreen from '$lib/components/TutorialScreen.svelte';

	// Sesi sebelumnya dibaca sekali saat aplikasi dibuka (untuk sapaan dan
	// boot Verbose), lalu sesi ini dicatat.
	const previous = startSession();
	const sessionAt = performance.now();
	const bootMode = settings.bootMode;
	let booting = $state(bootMode !== 'off');
	// Dimulai saat inisialisasi (bukan onMount) supaya Boot, yang terpasang
	// lebih dulu, menunggu janji yang sama.
	const ready = load().catch((e) => {
		app.error = e;
	});
	let main: HTMLElement | undefined = $state();

	const screen = $derived(current());
	const crumbs = $derived(
		app.stack
			.map((s) => {
				const name = t(`crumb.${s.name}` as Key);
				return 'id' in s ? `${name} ${s.id}` : name;
			})
			.join(' / ')
	);

	$effect(() => {
		document.documentElement.dataset.theme = settings.theme;
		document.documentElement.lang = lang();
	});

	// Fokus pindah ke kontrol pertama tiap kali layar berganti, supaya menu
	// bisa dipakai penuh dengan keyboard (SPEC §4).
	$effect(() => {
		void screen;
		void booting;
		void app.catalog;
		void main;
		void app.tutorial?.index;
		queueMicrotask(() => {
			if (app.consoleOpen) return;
			// Papan game didahulukan, lalu kontrol yang disorot tutorial.
			const first =
				main?.querySelector<HTMLElement>(`[role='grid']${NAV_SELECTOR}`) ??
				main?.querySelector<HTMLElement>(`${NAV_SELECTOR}.sorot`) ??
				main?.querySelector<HTMLElement>(NAV_SELECTOR);
			// focusVisible: sorotan tampil walau fokus dipasang lewat skrip.
			first?.focus({ focusVisible: true } as FocusOptions);
		});
	});

	function navItems(): HTMLElement[] {
		return Array.from(main?.querySelectorAll<HTMLElement>(NAV_SELECTOR) ?? []);
	}

	function onkeydown(e: KeyboardEvent) {
		if (booting) return;
		const inInput = e.target instanceof HTMLInputElement;
		if (inInput) return;
		if (e.key === ':' || e.key === '`') {
			e.preventDefault();
			app.consoleOpen = true;
			return;
		}
		if (e.key === 'Escape') {
			e.preventDefault();
			back();
			return;
		}
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
			const items = navItems();
			if (items.length === 0) return;
			e.preventDefault();
			moveFocus(items, document.activeElement, e.key === 'ArrowDown' || e.key === 'ArrowRight' ? 1 : -1);
		}
	}
</script>

<svelte:window {onkeydown} />

<div
	class="crt"
	style:--fx={settings.intensity / 100}
	class:fx-scanline={effectOn('scanline')}
	class:fx-glow={effectOn('glow')}
	class:fx-curve={effectOn('curve')}
	class:fx-flicker={effectOn('flicker')}
>
	{#if booting}
		{#if bootMode !== 'off'}
			<Boot mode={bootMode} {ready} {previous} {sessionAt} ondone={() => (booting = false)} />
		{/if}
	{:else}
		<div class="shell">
			<header>
				<span class="display brand">KyuSin</span>
				<span class="dim crumbs">{crumbs}</span>
				<span class="dim keys">{t('keys.header')}</span>
			</header>

			<main bind:this={main}>
				{#if app.error}
					<p role="alert">{t('error.load', { error: errorText(app.error) })}</p>
				{:else if screen.name === 'menu'}
					<MenuScreen />
				{:else if screen.name === 'game'}
					<GameScreen id={screen.id} />
				{:else if screen.name === 'tutorial'}
					<TutorialScreen />
				{:else if screen.name === 'match'}
					<MatchScreen />
				{:else if screen.name === 'replay'}
					<ReplayScreen />
				{:else if screen.name === 'settings'}
					<SettingsScreen />
				{:else if screen.name === 'help'}
					<HelpScreen />
				{/if}
			</main>

			<Console />
		</div>
	{/if}
</div>

<style>
	.shell {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	header {
		display: flex;
		align-items: baseline;
		gap: 3ch;
		padding: 0.75rem 3ch 0.25rem;
	}
	.brand {
		font-size: 2.25rem;
		line-height: 1;
	}
	.keys {
		margin-left: auto;
	}
	main {
		flex: 1;
		min-height: 0;
		overflow: auto;
		padding: 1rem 3ch;
	}
</style>
