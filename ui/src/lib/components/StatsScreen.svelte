<!--
  Statistik (SPEC §8, §9 M3): rating lokal Glicko-2 per game, rekor
  menang–seri–kalah, dan riwayat pertandingan yang selesai (tautan ke
  replay). Tabel memakai kolom selebar karakter tetap supaya rapi di font
  monospace.
-->
<script lang="ts">
	import { api, type StatsState } from '$lib/backend';
	import { findGame, openReplay } from '$lib/app.svelte';
	import { L, errorText, lang, t, type Key } from '$lib/i18n.svelte';
	import { signed } from '$lib/format';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let stats = $state<StatsState | null>(null);
	let error = $state<unknown>(null);

	$effect(() => {
		api()
			.then((a) => a.stats(null))
			.then((s) => (stats = s))
			.catch((e) => (error = e));
	});

	const name = (id: string) => {
		const g = findGame(id);
		return g ? L(g.nama) : id;
	};

	function date(ms: number): string {
		return new Intl.DateTimeFormat(lang(), { dateStyle: 'medium', timeStyle: 'short' }).format(ms);
	}

	const OUTCOME: Record<string, Key> = { win: 'replay.win', draw: 'replay.draw', loss: 'replay.lose' };

	function item(h: StatsState['history'][number]): string {
		const change = h.rating
			? t('stats.change', {
					before: Math.round(h.rating.before),
					after: Math.round(h.rating.after),
					delta: signed(Math.round(h.rating.after) - Math.round(h.rating.before))
				})
			: '';
		return t('stats.item', {
			date: date(h.finished_at),
			game: name(h.game),
			opponent: h.opponent_level === null ? '-' : t('stats.bot', { level: h.opponent_level }),
			outcome: t(OUTCOME[h.outcome]),
			change
		});
	}

	const COLS = ['game', 'rating', 'played', 'record', 'best'] as const;
	const CASINO_COLS = ['game', 'rounds', 'wagered', 'net'] as const;
	const sum = (k: 'rounds' | 'wagered' | 'net') => stats?.casino.reduce((s, c) => s + c[k], 0) ?? 0;
</script>

<div class="stats">
	{#if error}
		<p role="alert">{errorText(error)}</p>
	{/if}
	<Frame title={t('stats.title')}>
		<p class="dim">{t('stats.note')}</p>
		{#if stats && stats.games.length === 0}
			<p>{t('stats.empty')}</p>
		{:else if stats}
			<div class="table" role="table" aria-label={t('stats.title')}>
				<div class="tr head dim" role="row">
					{#each COLS as c (c)}<span role="columnheader" data-col={c}>{t(`stats.col.${c}` as Key)}</span>{/each}
				</div>
				{#each stats.games as g (g.game)}
					<div class="tr" role="row" data-game={g.game}>
						<span role="cell" data-col="game">{name(g.game)}</span>
						<span role="cell" data-col="rating" class="num"
							>{g.rating
								? `${Math.round(g.rating.rating)} ±${String(Math.round(g.rating.rd)).padStart(3)}`
								: t('stats.unrated')}</span
						>
						<span role="cell" data-col="played" class="num">{g.played}</span>
						<span role="cell" data-col="record" class="num">{g.wins}–{g.draws}–{g.losses}</span>
						<span role="cell" data-col="best" class="num"
							>{g.rating ? Math.round(g.rating.best) : ''}</span
						>
					</div>
				{/each}
			</div>
		{/if}
	</Frame>

	{#if stats && stats.casino.length > 0}
		<Frame title={t('stats.casino')}>
			<p class="dim">{t('stats.casino_note')}</p>
			<div class="table casino" role="table" aria-label={t('stats.casino')}>
				<div class="tr head dim" role="row">
					{#each CASINO_COLS as c (c)}<span role="columnheader" data-col={c}>{t(`stats.casino.${c}` as Key)}</span>{/each}
				</div>
				{#each stats.casino as c (c.game)}
					<div class="tr" role="row" data-game={c.game}>
						<span role="cell" data-col="game">{name(c.game)}</span>
						<span role="cell" data-col="rounds" class="num">{c.rounds}</span>
						<span role="cell" data-col="wagered" class="num">{c.wagered}</span>
						<span role="cell" data-col="net" class="num">{signed(c.net)}</span>
					</div>
				{/each}
				{#if stats.casino.length > 1}
					<div class="tr total" role="row" data-game="total">
						<span role="cell" data-col="game">{t('stats.casino_total')}</span>
						<span role="cell" data-col="rounds" class="num">{sum('rounds')}</span>
						<span role="cell" data-col="wagered" class="num">{sum('wagered')}</span>
						<span role="cell" data-col="net" class="num">{signed(sum('net'))}</span>
					</div>
				{/if}
			</div>
		</Frame>
	{/if}

	<Frame title={t('stats.history')}>
		{#if stats && stats.history.length === 0}
			<p class="dim">{t('stats.empty')}</p>
		{:else if stats}
			{#each stats.history as h (h.id)}
				<div class="hist">
					{#if h.replay_id !== null}
						<NavButton onclick={() => h.replay_id !== null && openReplay(h.replay_id)}>› {item(h)}</NavButton>
					{:else}
						<span>&nbsp; {item(h)}</span>
					{/if}
				</div>
			{/each}
		{/if}
	</Frame>
</div>

<style>
	.stats {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
	}
	p {
		margin: 0 0 calc(var(--cell-h) / 2);
	}
	.table {
		display: flex;
		flex-direction: column;
	}
	.tr {
		display: grid;
		grid-template-columns: 16ch 11ch 6ch 8ch 7ch;
		column-gap: 2ch;
		white-space: pre;
	}
	.num {
		text-align: right;
	}
	.casino .tr {
		grid-template-columns: 16ch 8ch 14ch 12ch;
	}
	.head span:not(:first-child) {
		text-align: right;
	}
</style>
