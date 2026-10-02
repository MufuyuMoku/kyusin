<!--
  Tutorial interaktif (SPEC §7.6). Pemain dipandu lewat kontrol visual:
  tombol yang harus disentuh disorot, perintah teks padanannya tampil kecil
  sebagai info. Kontrol di sini generik (dari ActionSpec); game katalog
  nanti memakai kontrol visualnya sendiri dengan target `sorot` yang sama.
-->
<script lang="ts">
	import { app, back, tutorialAct, tutorialNext } from '$lib/app.svelte';
	import { GAME_UI } from '$lib/games';
	import { L, t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	const tut = $derived(app.tutorial);
	const highlighted = $derived(
		new Set(
			(tut?.step?.sorot ?? [])
				.filter((s) => s.startsWith('aksi:') || s.startsWith('petak:'))
				.map((s) => s.slice(s.indexOf(':') + 1))
		)
	);
	const buttons = $derived(tut?.actions.flatMap((a) => a.concrete ?? []) ?? []);
	/** Kontrol visual game ini, bila ada (papan Reversi, dsb.). */
	const ui = $derived(tut ? GAME_UI[tut.game] : undefined);
</script>

{#if tut}
	<div class="tut">
		<div class="head">
			<span class="display big">{L(tut.title)}</span>
			<span class="dim">
				{tut.finished ? t('tutorial.done') : t('tutorial.step', { i: tut.index + 1, n: tut.total })}
			</span>
			<NavButton onclick={back}>[ {t('action.exit')} ]</NavButton>
		</div>

		{#if ui}
			<div>
				<ui.board
					view={tut.view_data}
					game={tut.game}
					interactive={!!tut.step?.aksi}
					highlight={highlighted}
					onplay={tutorialAct}
					actions={tut.actions.map((a) => a.usage)}
				/>
				<ui.status view={tut.view_data} />
			</div>
		{:else}
			<Frame title={t('tutorial.board')}>
				<pre class="view">{L(tut.view_text)}</pre>
			</Frame>
		{/if}

		<Frame title={t('tutorial.frame')}>
			{#if tut.finished}
				<p>{t('tutorial.finished')}</p>
				<div class="controls">
					<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
				</div>
			{:else if tut.step}
				<p>{L(tut.step.teks)}</p>
				<div class="controls">
					{#if tut.step.aksi && ui}
						<!-- Aksi lewat papan; tombol hanya untuk perintah di luar papan (pass). -->
						{#each buttons.filter((b) => b === 'pass') as cmd (cmd)}
							<NavButton sorot={highlighted.has(cmd)} onclick={() => tutorialAct(cmd)}
								>[ {cmd.toUpperCase()} ]</NavButton
							>
						{/each}
					{:else if tut.step.aksi}
						<!-- Label tombol = perintah teks, yang tetap Inggris di kedua bahasa. -->
						{#each buttons as cmd (cmd)}
							<NavButton sorot={highlighted.has(cmd)} onclick={() => tutorialAct(cmd)}
								>[ {cmd.toUpperCase()} ]</NavButton
							>
						{/each}
					{:else}
						<NavButton sorot onclick={tutorialNext}>[ {t('action.next')} ]</NavButton>
					{/if}
				</div>
				{#if tut.feedback?.kind === 'wrong'}
					<p class="hint" role="alert">! {L(tut.feedback.hint)}</p>
				{/if}
				{#if tut.step.aksi}
					<p class="dim small">{t('tutorial.command', { command: tut.step.aksi })}</p>
				{/if}
			{/if}
		</Frame>
	</div>
{/if}

<style>
	.tut {
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
	.view {
		white-space: pre;
	}
	p {
		margin: 0;
	}
	.controls {
		display: flex;
		flex-wrap: wrap;
		gap: 1ch 2ch;
		margin: var(--cell-h) 0;
	}
	.hint {
		margin-bottom: var(--cell-h);
	}
	.small {
		font-size: 0.85em;
	}
</style>
