<!--
  Tutorial interaktif (SPEC §7.6). Pemain dipandu lewat kontrol visual:
  tombol yang harus disentuh disorot, perintah teks padanannya tampil kecil
  sebagai info. Kontrol di sini generik (dari ActionSpec); game katalog
  nanti memakai kontrol visualnya sendiri dengan target `sorot` yang sama.
-->
<script lang="ts">
	import { app, back, tutorialAct, tutorialNext } from '$lib/app.svelte';
	import { L, t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';

	const tut = $derived(app.tutorial);
	const highlighted = $derived(
		new Set((tut?.step?.sorot ?? []).filter((s) => s.startsWith('aksi:')).map((s) => s.slice(5)))
	);
	const buttons = $derived(tut?.actions.flatMap((a) => a.concrete ?? []) ?? []);
</script>

{#if tut}
	<div class="tut">
		<div class="head">
			<span class="display big">{L(tut.title)}</span>
			<span class="dim">
				{tut.finished ? t('tutorial.done') : t('tutorial.step', { i: tut.index + 1, n: tut.total })}
			</span>
			<button class="tbtn" data-nav onclick={back}>[ {t('action.exit')} ]</button>
		</div>

		<Frame title={t('tutorial.board')}>
			<pre class="view">{L(tut.view_text)}</pre>
		</Frame>

		<Frame title={t('tutorial.frame')}>
			{#if tut.finished}
				<p>{t('tutorial.finished')}</p>
				<div class="controls">
					<button class="tbtn" data-nav onclick={back}>[ {t('action.back')} ]</button>
				</div>
			{:else if tut.step}
				<p>{L(tut.step.teks)}</p>
				<div class="controls">
					{#if tut.step.aksi}
						<!-- Label tombol = perintah teks, yang tetap Inggris di kedua bahasa. -->
						{#each buttons as cmd (cmd)}
							<button
								class="tbtn"
								class:sorot={highlighted.has(cmd)}
								data-nav
								onclick={() => tutorialAct(cmd)}>[ {cmd.toUpperCase()} ]</button
							>
						{/each}
					{:else}
						<button class="tbtn sorot" data-nav onclick={tutorialNext}>[ {t('action.next')} ]</button>
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
	.head .tbtn {
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
	.sorot {
		outline: 1px solid var(--fg);
		animation: sorot 1.06s steps(1) infinite;
	}
	@keyframes sorot {
		50% {
			background: var(--fg);
			color: var(--bg);
		}
	}
	.hint {
		margin-bottom: var(--cell-h);
	}
	.small {
		font-size: 0.85em;
	}
</style>
