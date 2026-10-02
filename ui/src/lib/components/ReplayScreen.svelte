<!--
  Penampil replay (SPEC §2.5): putar ulang langkah demi langkah dari seed +
  urutan perintah, dengan catatan provably fair dan hasil verify. Juga
  menampilkan partai impor PGN (tanpa catatan seed) dan mengekspor PGN
  replay catur (SPEC §6.1).
-->
<script lang="ts">
	import { app, back, findGame } from '$lib/app.svelte';
	import { api } from '$lib/backend';
	import Frame from './Frame.svelte';
	import { GAME_UI } from '$lib/games';
	import { L, errorText, lang, t } from '$lib/i18n.svelte';
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

	// Ekspor PGN (catur): teks ditampilkan dan bisa disalin.
	const canPgn = $derived(!!r && r.id !== null && !!GAME_UI[r.game]?.pgn);
	let pgn = $state<string | null>(null);
	let pgnNote = $state('');
	$effect(() => {
		void app.replay;
		pgn = null;
		pgnNote = '';
	});
	async function showPgn() {
		if (!r || r.id === null) return;
		try {
			pgn = await (await api()).replay_pgn(r.id, lang());
		} catch (e) {
			pgnNote = errorText(e);
		}
	}
	async function copyPgn() {
		if (!pgn) return;
		try {
			await navigator.clipboard.writeText(pgn);
			pgnNote = t('pgn.copied');
		} catch (e) {
			pgnNote = errorText(e);
		}
	}
	const tag = (name: string) => r?.tags.find(([k]) => k === name)?.[1];
</script>

{#if r && frame}
	<div class="replay">
		<div class="head">
			<span class="display big">{game ? L(game.nama) : r.game}</span>
			<span class="dim"
				>{r.id === null ? t('replay.imported') : t('replay.title', { id: r.id })}</span
			>
			<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
		</div>
		<div class="body">
			<div class="left">
				{#if ui}
					<ui.board view={frame.view_data} game={r?.game} />
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
				{#if canPgn}
					<div class="controls">
						<NavButton onclick={showPgn}>[ {t('action.pgn')} ]</NavButton>
						{#if pgn}<NavButton onclick={copyPgn}>[ {t('action.copy')} ]</NavButton>{/if}
					</div>
				{/if}
				{#if pgnNote}<p role="status">{pgnNote}</p>{/if}
			</div>
			<div class="right">
				{#if r.fair}
					<FairPanel commitments={r.fair.commitments} reveal={r.fair} verify={r.verify} />
				{:else}
					<Frame title={t('replay.imported')}>
						{#if tag('White') || tag('Black')}<p>{tag('White') ?? '?'} – {tag('Black') ?? '?'}</p>{/if}
						{#if tag('Event')}<p class="dim">{tag('Event')}{tag('Date') ? ` · ${tag('Date')}` : ''}</p>{/if}
						<p class="dim">{t('replay.no_fair')}</p>
					</Frame>
				{/if}
				{#if pgn}
					<Frame title={t('action.pgn')}>
						<pre class="pgn">{pgn}</pre>
					</Frame>
				{/if}
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
	.pgn {
		white-space: pre-wrap;
		user-select: text;
	}
</style>
