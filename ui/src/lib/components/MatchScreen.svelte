<!--
  Bermain melawan bot (SPEC §1 poin akhir 1). Kontrol visual adalah cara
  utama (papan per game); perintah teks tetap bisa lewat konsol. Setelah
  selesai: hasil, seed yang dibuka, verify otomatis, dan tautan replay.
  `Esc` / [ KELUAR ] di tengah permainan membuka menu jeda (SPEC §4 Rev. 9).
-->
<script lang="ts">
	import {
		app,
		back,
		findGame,
		matchAct,
		matchFlag,
		nextRound,
		openReplay,
		requestBack,
		startMatch
	} from '$lib/app.svelte';
	import ClockPanel from './ClockPanel.svelte';
	import { GAME_UI } from '$lib/games';
	import { signed } from '$lib/format';
	import { L, t } from '$lib/i18n.svelte';
	import FairPanel from './FairPanel.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';
	import PauseMenu from './PauseMenu.svelte';

	/** Kata perintah; sama di kedua bahasa (SPEC §4). */
	const PASS = 'pass';
	const RESIGN = 'resign';

	const m = $derived(app.match);
	const ui = $derived(m ? GAME_UI[m.game] : undefined);
	const game = $derived(m ? findGame(m.game) : undefined);
	const bot = $derived(m?.seats.find((s) => s.kind === 'bot'));
	const level = $derived(bot && bot.kind === 'bot' ? bot.level : 1);
	const buttons = $derived(m?.actions.flatMap((a) => a.concrete ?? []) ?? []);
	const moveButtons = $derived(buttons.filter((b) => b !== RESIGN));
	const canPass = $derived(m?.your_turn && moveButtons.length === 1 && moveButtons[0] === PASS);
	const paused = $derived(app.pauseMenu && !!m && !m.over);
	/** Casino melawan bandar: tanpa kalimat menang/kalah dan tanpa lawan bot. */
	const house = $derived(game?.lawan === 'bandar');
	/** Meja casino antar-pemain (buy-in): menu jeda seperti meja casino. */
	const table = $derived(game?.kategori === 'casino-meja' && game?.lawan === 'bot');
	const usages = $derived(m?.actions.map((a) => a.usage) ?? []);
	/** Casino satu ronde per sesi: taruhan setelah ronde memulai ronde baru. */
	const roundOver = $derived(!!m && m.over && house && !!ui?.perRound);
	const canResign = $derived(!!m && !m.over && m.your_turn && buttons.includes(RESIGN));
	let confirmResign = $state(false);

	// Perintah dari papan/meja yang ditolak mesin (misalnya susunan Capsa
	// yang tidak sah) ditampilkan, bukan diabaikan diam-diam.
	let actError = $state<string | null>(null);
	async function play(cmd: string) {
		try {
			await matchAct(cmd);
			actError = null;
		} catch (e) {
			actError = e && typeof e === 'object' && 'id' in e ? L(e as Parameters<typeof L>[0]) : String(e);
		}
	}

	const names = $derived<[string, string]>(
		(m?.seats ?? []).map((s) => (s.kind === 'bot' ? t('match.bot_name', { level: s.level }) : t('match.you'))) as [
			string,
			string
		]
	);

	const outcome = $derived.by(() => {
		if (!m?.result) return '';
		if (m.result.winners.length === 0) return t('match.draw');
		return m.result.winners.includes(m.seat) ? t('match.win') : t('match.lose');
	});

	/** Daftar langkah berpasangan: "1. d3 c5". */
	const moveList = $derived.by(() => {
		const out: string[] = [];
		const mv = m?.moves ?? [];
		for (let i = 0; i < mv.length; i += 2) {
			out.push(`${i / 2 + 1}. ${mv[i].command}${mv[i + 1] ? ` ${mv[i + 1].command}` : ''}`);
		}
		return out.join('  ');
	});
</script>

{#if m}
	<div class="match">
		{#if paused}
			<PauseMenu house={house || table} canLeave={usages.includes('leave')} />
		{/if}
		<div class="head" inert={paused}>
			<span class="display big">{game ? L(game.nama) : m.game}</span>
			<span class="dim">{house ? t('match.vs_house') : t('match.vs', { level })}</span>
			<NavButton onclick={requestBack}>[ {t('action.exit')} ]</NavButton>
		</div>

		<div class="body" inert={paused}>
			<div class="left">
				{#if ui}
					<!-- Dipasang ulang per pertandingan: kursor mulai dari posisi awal game. -->
					{#key m.started_at}
						<ui.board
							view={m.view_data}
							game={m.game}
							interactive={(m.your_turn || roundOver) && !paused}
							onplay={roundOver ? (cmd: string) => nextRound(m.game, cmd) : play}
							nextRound={roundOver}
							actions={usages}
							chips={m.chips}
						/>
					{/key}
					<ui.status view={m.view_data} botTurn={m.bot_turn} />
					{#if actError}<p class="act-error" role="alert">{t('match.act_error', { error: actError })}</p>{/if}
				{:else}
					<pre>{L(m.view_text)}</pre>
				{/if}
				<div class="controls">
					{#if canPass}
						<NavButton sorot onclick={() => matchAct(PASS)}>[ {PASS.toUpperCase()} ]</NavButton>
					{:else if !ui && m.your_turn}
						{#each moveButtons as cmd (cmd)}
							<NavButton onclick={() => matchAct(cmd)}>[ {cmd.toUpperCase()} ]</NavButton>
						{/each}
					{/if}
					{#if canResign && !confirmResign}
						<NavButton onclick={() => (confirmResign = true)}>[ {t('action.resign')} ]</NavButton>
					{:else if canResign}
						<NavButton
							onclick={() => {
								confirmResign = false;
								matchAct(RESIGN);
							}}>[ {t('action.resign_confirm')} ]</NavButton
						>
						<NavButton onclick={() => (confirmResign = false)}>[ {t('action.cancel')} ]</NavButton>
					{/if}
				</div>
				{#if m.over}
					<p class="result" role="status">{house ? '' : outcome} {m.result ? L(m.result.summary) : ''}</p>
					{#if m.rating}
						<p class="rating">
							{t('match.rating', {
								before: Math.round(m.rating.before),
								after: Math.round(m.rating.after),
								delta: signed(Math.round(m.rating.after) - Math.round(m.rating.before))
							})}
						</p>
					{/if}
					<div class="controls">
						<NavButton onclick={() => startMatch(m.game, level, m.seat, app.lastClock)}
							>[ {house ? t(ui?.perRound ? 'action.new_round' : 'action.new_shoe') : t('action.again')} ]</NavButton
						>
						{#if m.replay_id !== null}
							<NavButton onclick={() => m.replay_id !== null && openReplay(m.replay_id)}
								>[ {t('action.view_replay')} ]</NavButton
							>
						{/if}
						<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
					</div>
					{#if m.replay_id !== null}
						<p class="dim">{t('replay.saved', { id: m.replay_id })}</p>
					{:else if m.save_error}
						<p>{t('replay.save_failed', { error: m.save_error })}</p>
					{/if}
				{/if}
			</div>

			<div class="right">
				{#if m.clock}
					<ClockPanel clock={m.clock} seat={m.seat} {names} onexpired={matchFlag} />
				{/if}
				<FairPanel commitments={m.commitments} reveal={m.reveal} verify={m.verify} />
				{#if moveList}
					<Frame title={t('match.moves')}>
						<p class="moves">{moveList}</p>
					</Frame>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.match {
		position: relative;
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
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
	}
	.controls {
		display: flex;
		flex-wrap: wrap;
		gap: 1ch 2ch;
	}
	p {
		margin: 0;
	}
	.moves {
		white-space: normal;
		word-spacing: 0.3ch;
	}
</style>
