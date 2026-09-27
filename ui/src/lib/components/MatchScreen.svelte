<!--
  Bermain melawan bot (SPEC §1 poin akhir 1). Kontrol visual adalah cara
  utama (papan per game); perintah teks tetap bisa lewat konsol. Setelah
  selesai: hasil, seed yang dibuka, verify otomatis, dan tautan replay.
-->
<script lang="ts">
	import { app, back, findGame, matchAct, openReplay, startMatch } from '$lib/app.svelte';
	import { GAME_UI } from '$lib/games';
	import { L, t } from '$lib/i18n.svelte';
	import FairPanel from './FairPanel.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	/** Kata perintah; sama di kedua bahasa (SPEC §4). */
	const PASS = 'pass';

	const m = $derived(app.match);
	const ui = $derived(m ? GAME_UI[m.game] : undefined);
	const game = $derived(m ? findGame(m.game) : undefined);
	const bot = $derived(m?.seats.find((s) => s.kind === 'bot'));
	const level = $derived(bot && bot.kind === 'bot' ? bot.level : 1);
	const buttons = $derived(m?.actions.flatMap((a) => a.concrete ?? []) ?? []);
	const canPass = $derived(m?.your_turn && buttons.length === 1 && buttons[0] === PASS);

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
		<div class="head">
			<span class="display big">{game ? L(game.nama) : m.game}</span>
			<span class="dim">{t('match.vs', { level })}</span>
			<NavButton onclick={back}>[ {t('action.exit')} ]</NavButton>
		</div>

		<div class="body">
			<div class="left">
				{#if ui}
					<ui.board view={m.view_data} interactive={m.your_turn} onplay={matchAct} />
					<ui.status view={m.view_data} botTurn={m.bot_turn} />
				{:else}
					<pre>{L(m.view_text)}</pre>
				{/if}
				<div class="controls">
					{#if canPass}
						<NavButton sorot onclick={() => matchAct(PASS)}>[ {PASS.toUpperCase()} ]</NavButton>
					{:else if !ui && m.your_turn}
						{#each buttons as cmd (cmd)}
							<NavButton onclick={() => matchAct(cmd)}>[ {cmd.toUpperCase()} ]</NavButton>
						{/each}
					{/if}
				</div>
				{#if m.over}
					<p class="result" role="status">{outcome} {m.result ? L(m.result.summary) : ''}</p>
					<div class="controls">
						<NavButton onclick={() => startMatch(m.game, level, m.seat)}>[ {t('action.again')} ]</NavButton>
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
