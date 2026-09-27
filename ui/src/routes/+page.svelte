<script lang="ts">
	import { onMount } from 'svelte';
	import { app, back, current, load } from '$lib/app.svelte';
	import { effectOn, settings } from '$lib/settings.svelte';
	import { errorText, lang, t, type Key } from '$lib/i18n.svelte';
	import Boot from '$lib/components/Boot.svelte';
	import Console from '$lib/components/Console.svelte';
	import GameScreen from '$lib/components/GameScreen.svelte';
	import HelpScreen from '$lib/components/HelpScreen.svelte';
	import MenuScreen from '$lib/components/MenuScreen.svelte';
	import SettingsScreen from '$lib/components/SettingsScreen.svelte';
	import TutorialScreen from '$lib/components/TutorialScreen.svelte';

	// Saat reduced motion, boot tetap tampil tetapi tanpa animasi ketik (D-026).
	let booting = $state(settings.boot);
	let loaded = $state(false);
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

	onMount(() => {
		load()
			.catch((e) => (app.error = e))
			.finally(() => (loaded = true));
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
			// Kontrol yang disorot tutorial didahulukan.
			const first =
				main?.querySelector<HTMLElement>('[data-nav].sorot') ??
				main?.querySelector<HTMLElement>('[data-nav]');
			// focusVisible: sorotan tampil walau fokus dipasang lewat skrip.
			first?.focus({ focusVisible: true } as FocusOptions);
		});
	});

	function navItems(): HTMLElement[] {
		return Array.from(main?.querySelectorAll<HTMLElement>('[data-nav]') ?? []);
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
			const i = items.indexOf(document.activeElement as HTMLElement);
			const step = e.key === 'ArrowDown' || e.key === 'ArrowRight' ? 1 : -1;
			const next = i < 0 ? 0 : (i + step + items.length) % items.length;
			e.preventDefault();
			items[next].focus();
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
		<Boot
			cartridges={loaded ? app.catalog.reduce((n, c) => n + c.games.length, 0) : 0}
			version={app.info?.version ?? ''}
			ondone={() => (booting = false)}
		/>
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
