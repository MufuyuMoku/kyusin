<!--
  Layar info game: main melawan bot (level dan posisi), tutorial, replay
  terakhir, dan halaman `man <id>` dari manifest + tutorial (SPEC §7.6).
-->
<script lang="ts">
	import { api, type ReplaySummary } from '$lib/backend';
	import { back, findGame, openReplay, startMatch, startTutorial } from '$lib/app.svelte';
	import { GAME_UI } from '$lib/games';
	import { L, errorText, lang, t, type Key, type Localized } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let { id }: { id: string } = $props();

	const game = $derived(findGame(id));
	const levels = $derived(game?.bot_levels ?? 0);
	const seatKeys = $derived(GAME_UI[id]?.seats ?? []);

	let page = $state<Localized | null>(null);
	let error = $state<unknown>(null);
	let level = $state(1);
	let seat = $state(0);
	let replays = $state<ReplaySummary[]>([]);

	$effect(() => {
		const target = id;
		error = null;
		api()
			.then((a) => a.man(target))
			.then((p) => (page = p))
			.catch((e) => (error = e));
		api()
			.then((a) => a.replay_list(target))
			.then((list) => (replays = list.slice(0, 8)))
			.catch(() => (replays = []));
	});

	function date(ms: number): string {
		return new Intl.DateTimeFormat(lang(), { dateStyle: 'medium', timeStyle: 'short' }).format(ms);
	}

	function describe(r: ReplaySummary): string {
		const bot = r.seats.find((s) => s.kind === 'bot');
		const human = r.seats.findIndex((s) => s.kind === 'human');
		let outcome = t('replay.unfinished');
		if (r.result) {
			outcome =
				r.result.winners.length === 0
					? t('replay.draw')
					: r.result.winners.includes(human)
						? t('replay.win')
						: t('replay.lose');
			outcome += ` ${r.result.scores.join('–')}`;
		}
		return t('replay.item', {
			id: r.id,
			date: date(r.started_at),
			level: bot && bot.kind === 'bot' ? bot.level : '-',
			moves: r.moves,
			outcome
		});
	}
</script>

<div class="game">
	<div class="actions">
		<NavButton onclick={() => startTutorial(id)}>[ {t('action.tutorial')} ]</NavButton>
		<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
	</div>

	{#if levels > 0}
		<Frame title={t('play.frame')}>
			<div class="choices">
				<div>
					<p class="dim">{t('play.level')}</p>
					{#each Array.from({ length: levels }, (_, i) => i + 1) as l (l)}
						<div>
							<NavButton pressed={level === l} onclick={() => (level = l)}
								>{level === l ? '(•)' : '( )'} {t(`level.${Math.min(l, 3)}` as Key, { n: l })}</NavButton
							>
						</div>
					{/each}
				</div>
				{#if seatKeys.length}
					<div>
						<p class="dim">{t('play.seat')}</p>
						{#each seatKeys as key, s (key)}
							<div>
								<NavButton pressed={seat === s} onclick={() => (seat = s)}
									>{seat === s ? '(•)' : '( )'} {t(key)}</NavButton
								>
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<div class="start">
				<NavButton onclick={() => startMatch(id, level, seat)}>[ {t('play.start')} ]</NavButton>
			</div>
		</Frame>
	{/if}

	{#if levels > 0}
		<Frame title={t('replay.list')}>
			{#if replays.length === 0}
				<p class="dim">{t('replay.none')}</p>
			{:else}
				{#each replays as r (r.id)}
					<div><NavButton onclick={() => openReplay(r.id)}>› {describe(r)}</NavButton></div>
				{/each}
			{/if}
		</Frame>
	{/if}

	<Frame title={`man ${id}`}>
		{#if error}
			<p>{errorText(error)}</p>
		{:else}
			<pre>{L(page)}</pre>
		{/if}
	</Frame>
</div>

<style>
	.game {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
	}
	.actions {
		display: flex;
		gap: 2ch;
	}
	.choices {
		display: flex;
		gap: 6ch;
		flex-wrap: wrap;
	}
	.start {
		margin-top: var(--cell-h);
	}
	p {
		margin: 0;
	}
</style>
