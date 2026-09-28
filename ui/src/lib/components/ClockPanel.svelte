<!--
  Jam catur (SPEC §6.1). Waktu dihitung host; tampilan ini hanya berdetak
  dari potret terakhir. Saat jam pemain lokal mencapai nol, host diminta
  memeriksa dengan jamnya sendiri (D-045).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ClockState } from '$lib/backend';
	import { t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';

	let {
		clock,
		seat,
		names,
		onexpired
	}: {
		clock: ClockState;
		/** Kursi pemain lokal. */
		seat: number;
		/** Nama kursi 0 dan 1. */
		names: [string, string];
		onexpired: () => void;
	} = $props();

	let received = $state(performance.now());
	let now = $state(performance.now());
	let flagged = false;

	$effect(() => {
		void clock;
		received = performance.now();
		flagged = false;
	});

	onMount(() => {
		const id = setInterval(() => (now = performance.now()), 200);
		return () => clearInterval(id);
	});

	function left(s: number): number {
		const base = clock.remaining_ms[s];
		return clock.running === s ? base - (now - received) : base;
	}

	$effect(() => {
		if (clock.running === seat && left(seat) <= 0 && !flagged) {
			flagged = true;
			onexpired();
		}
	});

	function fmt(ms: number): string {
		const total = Math.max(0, Math.ceil(ms / 1000));
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
	}
</script>

<Frame title={t('clock.frame')}>
	{#each [0, 1] as s (s)}
		<div class="row" class:running={clock.running === s} class:low={left(s) < 10_000}>
			<span class="who">{clock.running === s ? '›' : ' '} {names[s]}</span>
			<span class="time" role="timer" aria-label={names[s]}>{fmt(left(s))}</span>
		</div>
	{/each}
	{#if clock.increment_ms > 0}
		<p class="dim">{t('clock.increment', { s: clock.increment_ms / 1000 })}</p>
	{/if}
</Frame>

<style>
	.row {
		display: flex;
		justify-content: space-between;
		gap: 2ch;
		white-space: pre;
	}
	.time {
		font-family: var(--font-display);
		font-size: 1.6rem;
		line-height: 1;
	}
	.running .time {
		background: var(--fg);
		color: var(--bg);
		padding: 0 0.5ch;
	}
	.low:not(.running) .time {
		text-decoration: underline;
	}
	p {
		margin: 0;
	}
</style>
