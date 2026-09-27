<!--
  Urutan boot (SPEC §4, D-031): memutar naskah Verbose, Sinematik, atau
  Sapaan. Bisa dilewati dengan tombol apa saja atau klik. Saat reduced
  motion semua teks tampil langsung tanpa animasi ketik (D-026).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n.svelte';
	import { motion, type BootMode } from '$lib/settings.svelte';
	import { rememberGreeting, type SessionRecord } from '$lib/boot/session';
	import {
		cinematic,
		greeting,
		stamp,
		statusTag,
		verbose,
		type Line,
		type Script
	} from '$lib/boot/scripts';

	let {
		mode,
		ready,
		previous,
		sessionAt,
		ondone
	}: {
		mode: Exclude<BootMode, 'off'>;
		ready: Promise<unknown>;
		previous: SessionRecord;
		sessionAt: number;
		ondone: () => void;
	} = $props();

	/** Baris yang sudah tampil; baris terakhir bisa masih diketik. */
	let shown = $state<Line[]>([]);
	let typing = $state('');
	let finished = false;
	let cancelled = false;

	function done() {
		if (finished) return;
		finished = true;
		cancelled = true;
		ondone();
	}

	const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

	function build(): Script {
		if (mode === 'verbose') return verbose(previous, sessionAt);
		if (mode === 'cinematic') return cinematic();
		const g = greeting(previous);
		rememberGreeting(g.id);
		return g.script;
	}

	async function play() {
		// Verbose hanya melaporkan langkah yang sudah terjadi, jadi tunggu
		// startup selesai (paling lama 3 detik; sisanya dilaporkan gagal).
		await Promise.race([ready.catch(() => {}), sleep(3000)]);
		if (cancelled) return;
		const script = build();
		if (motion.reduced) {
			shown = script.lines;
			await sleep(Math.max(script.hold, 2500));
			done();
			return;
		}
		for (const line of script.lines) {
			await sleep(line.pause ?? 0);
			if (cancelled) return;
			if (line.typed && script.charMs > 0) {
				shown = [...shown, { ...line, text: '' }];
				for (let i = 1; i <= line.text.length; i++) {
					typing = line.text.slice(0, i);
					await sleep(script.charMs);
					if (cancelled) return;
				}
				shown = [...shown.slice(0, -1), line];
				typing = '';
			} else {
				shown = [...shown, line];
			}
		}
		await sleep(script.hold);
		done();
	}

	onMount(() => {
		play();
		// Tombol atau klik yang melewati boot hanya melewati boot: jangan
		// sampai ikut memilih tombol menu yang baru saja muncul dan difokuskan.
		const swallowClick = (e: MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();
		};
		const skipKey = (e: KeyboardEvent) => {
			e.preventDefault();
			e.stopPropagation();
			done();
		};
		const skipPointer = () => {
			window.addEventListener('click', swallowClick, { capture: true, once: true });
			setTimeout(() => window.removeEventListener('click', swallowClick, { capture: true }), 600);
			done();
		};
		window.addEventListener('keydown', skipKey, { capture: true, once: true });
		window.addEventListener('pointerdown', skipPointer, { once: true });
		return () => {
			cancelled = true;
			window.removeEventListener('keydown', skipKey, { capture: true });
			window.removeEventListener('pointerdown', skipPointer);
		};
	});
</script>

<div class="boot" class:verbose={mode === 'verbose'} role="status" aria-live="polite">
	{#each shown as line, i (i)}
		{@const text = i === shown.length - 1 && typing ? typing : line.text}
		{#if line.style === 'log'}
			<div class="log">
				<span class="dim">{stamp(line.stamp ?? 0)}</span>
				<span class:fail={line.status === 'fail'} class:dim={line.status === 'info'}
					>{statusTag(line.status ?? 'info')}</span
				>
				<span>{text}</span>
			</div>
		{:else}
			<div class:display={line.style === 'title' || line.style === 'logo'} class={line.style}>
				{text || ' '}
			</div>
		{/if}
	{/each}
	<span class="cursor">&nbsp;</span>
	<div class="skip dim">{t('boot.skip')}</div>
</div>

<style>
	.boot {
		padding: 3rem 4ch;
		height: 100%;
		position: relative;
		overflow: hidden;
	}
	.boot.verbose {
		padding: 1rem 2ch;
		font-size: 0.9em;
	}
	.log {
		display: flex;
		gap: 1ch;
		white-space: pre;
	}
	.fail {
		background: var(--fg);
		color: var(--bg);
	}
	.title {
		font-size: 3rem;
		line-height: 1;
		margin-bottom: var(--cell-h);
	}
	.logo {
		font-size: 6rem;
		line-height: 1;
		margin-top: calc(var(--cell-h) * 2);
	}
	.plain {
		font-size: 1.4rem;
		line-height: 1.5;
		max-width: 60ch;
		white-space: normal;
	}
	.skip {
		position: absolute;
		bottom: 2rem;
		left: 4ch;
	}
</style>
