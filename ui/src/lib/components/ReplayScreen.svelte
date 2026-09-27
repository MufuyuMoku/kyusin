<!--
  Penampil replay (SPEC §2.5): putar ulang langkah demi langkah dari seed +
  urutan perintah, dengan catatan provably fair dan hasil verify.
-->
<script lang="ts">
	import { app, back, findGame } from '$lib/app.svelte';
	import { GAME_UI } from '$lib/games';
	import { L, t } from '$lib/i18n.svelte';
	import FairPanel from './FairPanel.svelte';
	import NavButton from './NavButton.svelte';

	const r = $derived(app.replay);
	const ui = $derived(r ? GAME_UI[r.game] : undefined);
	const game = $derived(r ? findGame(r.game) : undefined);
	const last = $derived((r?.frames.length ?? 1) - 1);

	let index = $state(0);
	// Mulai dari keadaan akhir tiap kali replay lain dibuka.
	$effect(() => {
		index = (app.replay?.frames.length ?? 1) - 1;
	});

	const frame = $derived(r?.frames[Math.min(index, last)]);
	const go = (i: number) => (index = Math.max(0, Math.min(last, i)));
</script>

{#if r && frame}
	<div class="replay">
		<div class="head">
			<span class="display big">{game ? L(game.nama) : r.game}</span>
			<span class="dim">{t('replay.title', { id: r.id })}</span>
			<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
		</div>
		<div class="body">
			<div class="left">
				{#if ui}
					<ui.board view={frame.view_data} />
					<ui.status view={frame.view_data} observer />
				{:else}
					<pre>{L(frame.view_text)}</pre>
				{/if}
				<div class="controls">
					<NavButton label={t('replay.nav.first')} disabled={index === 0} onclick={() => go(0)}>[ |&lt; ]</NavButton>
					<NavButton label={t('replay.nav.prev')} disabled={index === 0} onclick={() => go(index - 1)}>[ &lt; ]</NavButton>
					<NavButton label={t('replay.nav.next')} disabled={index >= last} onclick={() => go(index + 1)}>[ &gt; ]</NavButton>
					<NavButton label={t('replay.nav.last')} disabled={index >= last} onclick={() => go(last)}>[ &gt;| ]</NavButton>
				</div>
				<p role="status">
					{t('replay.step', { i: frame.index, n: last })}{frame.last ? ` · ${frame.last.command}` : ''}
				</p>
				{#if r.result && index === last}
					<p>{L(r.result.summary)}</p>
				{/if}
			</div>
			<div class="right">
				<FairPanel commitments={r.fair.commitments} reveal={r.fair} verify={r.verify} />
			</div>
		</div>
	</div>
{/if}

<style>
	.replay {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
	}
	.head {
		display: flex;
		align-items: baseline;
		gap: 2ch;
	}
	.head :global(.tbtn) {
		margin-left: auto;
	}
	.big {
		font-size: 2rem;
	}
	.body {
		display: flex;
		gap: 3ch;
		flex-wrap: wrap;
		align-items: flex-start;
	}
	.left {
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 2);
	}
	.right {
		flex: 1;
		min-width: 40ch;
	}
	.controls {
		display: flex;
		gap: 2ch;
	}
	p {
		margin: 0;
	}
</style>
