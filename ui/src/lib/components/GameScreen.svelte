<!--
  Layar info game: pertandingan tertunda (SPEC §4), main melawan bot (level
  dan posisi), tutorial, replay terakhir, dan halaman `man <id>` dari
  manifest + tutorial (SPEC §7.6).
-->
<script lang="ts">
	import { api, type ReplaySummary, type SuspendedMatch } from '$lib/backend';
	import {
		back,
		findGame,
		forfeitAndStart,
		go,
		openReplay,
		resumeSuspended,
		startMatch,
		startTutorial
	} from '$lib/app.svelte';
	import { clockText } from '$lib/format';
	import { GAME_UI } from '$lib/games';
	import { L, errorText, lang, t, type Key, type Localized } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let { id }: { id: string } = $props();

	const game = $derived(findGame(id));
	const levels = $derived(game?.bot_levels ?? 0);
	/** Casino melawan bandar: dimainkan tanpa pilihan lawan. */
	const house = $derived(game?.lawan === 'bandar');
	const playable = $derived(levels > 0 || house);
	let chips = $state<number | null>(null);
	const seatKeys = $derived(GAME_UI[id]?.seats ?? []);
	const clocks = $derived(GAME_UI[id]?.clocks ?? []);
	const hasPgn = $derived(!!GAME_UI[id]?.pgn);
	let clockIndex = $state(0);

	function levelLabel(l: number): string {
		const name = t(`level.${Math.min(l, 6)}` as Key, { n: l });
		const elo = game?.bot_ratings?.[l - 1];
		return elo ? `${name} · ${t('play.rating', { elo })}` : name;
	}

	function clockLabel(c: { minutes: number; increment: number } | null): string {
		return c ? t('clock.option', { m: c.minutes, s: c.increment }) : t('clock.none');
	}

	let page = $state<Localized | null>(null);
	let error = $state<unknown>(null);
	let level = $state(1);
	let seat = $state(0);
	let replays = $state<ReplaySummary[]>([]);
	let suspended = $state<SuspendedMatch | null>(null);
	let confirmNew = $state(false);

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
		api()
			.then((a) => a.profile_get())
			.then((p) => (chips = p.chips))
			.catch(() => (chips = null));
		api()
			.then((a) => a.suspended_list())
			.then((list) => (suspended = list.find((s) => s.game === target) ?? null))
			.catch(() => (suspended = null));
	});

	function describeSuspended(s: SuspendedMatch): string {
		const bot = s.seats.find((x) => x.kind === 'bot');
		return t('suspended.item', {
			level: bot && bot.kind === 'bot' ? bot.level : '-',
			moves: s.moves,
			date: date(s.suspended_at)
		});
	}

	function play() {
		const clock = clocks[clockIndex] ?? null;
		if (suspended && !confirmNew) {
			confirmNew = true;
			return;
		}
		confirmNew = false;
		if (suspended) forfeitAndStart(id, level, seat, clock);
		else startMatch(id, level, seat, clock);
	}

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
	{#if suspended}
		<Frame title={t('suspended.title')}>
			<p>{describeSuspended(suspended)}</p>
			{#if suspended.clock}
				<p class="dim">
					{t('suspended.clock', {
						you: clockText(suspended.clock.remaining_ms[suspended.seat]),
						bot: clockText(suspended.clock.remaining_ms[1 - suspended.seat])
					})}
				</p>
			{/if}
			<div class="start">
				<NavButton onclick={() => resumeSuspended(id)}>[ {t('action.continue')} ]</NavButton>
			</div>
		</Frame>
	{/if}

	<div class="actions">
		<NavButton onclick={() => startTutorial(id)}>[ {t('action.tutorial')} ]</NavButton>
		<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
	</div>

	{#if playable}
		<Frame title={t('play.frame')}>
			{#if house}
				<p>{t('play.house', { chips: chips ?? '-', min: 10, max: 2000 })}</p>
			{/if}
			<div class="choices">
				{#if levels > 0}<div>
					<p class="dim">{t('play.level')}</p>
					{#each Array.from({ length: levels }, (_, i) => i + 1) as l (l)}
						<div>
							<NavButton pressed={level === l} onclick={() => (level = l)}
								>{level === l ? '(•)' : '( )'} {levelLabel(l)}</NavButton
							>
						</div>
					{/each}
				</div>{/if}
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
				{#if clocks.length}
					<div>
						<p class="dim">{t('play.clock')}</p>
						{#each clocks as c, i (i)}
							<div>
								<NavButton pressed={clockIndex === i} onclick={() => (clockIndex = i)}
									>{clockIndex === i ? '(•)' : '( )'} {clockLabel(c)}</NavButton
								>
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<div class="start">
				{#if confirmNew}
					<p>{t(house ? 'suspended.forfeit_house' : 'suspended.forfeit')}</p>
					<div class="actions">
						<NavButton onclick={() => (confirmNew = false)}>[ {t('action.cancel')} ]</NavButton>
						<NavButton onclick={play}>[ {t('action.new_game')} ]</NavButton>
					</div>
				{:else}
					<NavButton onclick={play}>[ {t('play.start')} ]</NavButton>
				{/if}
			</div>
		</Frame>
	{/if}

	{#if playable}
		<Frame title={t('replay.list')}>
			{#if hasPgn}
				<div class="import">
					<NavButton onclick={() => go({ name: 'pgn' })}>[ {t('action.import_pgn')} ]</NavButton>
				</div>
			{/if}
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
	.import {
		margin-bottom: calc(var(--cell-h) / 2);
	}
	.start {
		margin-top: var(--cell-h);
	}
	p {
		margin: 0;
	}
</style>
