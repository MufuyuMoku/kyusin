<!--
  Provably fair (SPEC §2.3, §5.4): komitmen tampil sejak awal ronde; seed
  dan hasil verify tampil setelah selesai.
-->
<script lang="ts">
	import type { FairRecord, VerifyReport } from '$lib/backend';
	import { L, t, type Key } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';

	let {
		commitments,
		reveal = null,
		verify = null
	}: {
		commitments: Record<string, string>;
		reveal?: FairRecord | null;
		verify?: VerifyReport | null;
	} = $props();

	function who(id: string): string {
		if (id === 'host') return t('fair.host');
		if (id === 'player') return t('fair.player');
		return id;
	}

	const short = (h: string) => `${h.slice(0, 16)}…${h.slice(-8)}`;
	const width = $derived(Math.max(...Object.keys(commitments).map((k) => who(k).length), 4));
</script>

<Frame title={t('fair.frame')}>
	<p class="dim">{t('fair.commitments')}</p>
	{#each Object.entries(commitments) as [id, hash] (id)}
		<div class="row"><span>{who(id).padEnd(width)}</span> <span title={hash}>{short(hash)}</span></div>
	{/each}
	{#if !reveal}
		<p class="dim gap">{t('fair.hidden')}</p>
	{:else}
		<p class="dim gap">{t('fair.seeds')}</p>
		{#each Object.entries(reveal.seeds) as [id, seed] (id)}
			<div class="row"><span>{who(id).padEnd(width)}</span> <span title={seed}>{short(seed)}</span></div>
		{/each}
		<div class="row">
			<span>{t('fair.round').padEnd(width)}</span> <span title={reveal.round_seed}>{short(reveal.round_seed)}</span>
		</div>
	{/if}
	{#if verify}
		<div class="gap">
			{#each verify.checks as c (c.step)}
				<div>{c.ok ? '[✓]' : '[×]'} {t(`verify.step.${c.step}` as Key)}</div>
			{/each}
			<p class:bad={!verify.ok} role="status">{verify.ok ? t('verify.ok') : t('verify.fail')}</p>
			{#if verify.error}<p>{L(verify.error)}</p>{/if}
		</div>
	{/if}
</Frame>

<style>
	p {
		margin: 0;
	}
	.row {
		white-space: pre;
	}
	.gap {
		margin-top: var(--cell-h);
	}
	.bad {
		background: var(--fg);
		color: var(--bg);
		display: inline-block;
		padding: 0 0.5ch;
	}
</style>
