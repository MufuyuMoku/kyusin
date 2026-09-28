<!--
  Impor PGN (SPEC §6.1): tempel teks satu partai, lalu buka di penampil
  replay. Langkah diperiksa terhadap mesin aturan asli; tidak disimpan.
-->
<script lang="ts">
	import { back, openPgn } from '$lib/app.svelte';
	import { errorText, t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let text = $state('');
	let error = $state('');

	async function open() {
		error = '';
		try {
			await openPgn(text);
		} catch (e) {
			error = errorText(e);
		}
	}
</script>

<div class="pgn">
	<Frame title={t('pgn.frame')}>
		<label for="pgn-text">{t('pgn.label')}</label>
		<textarea id="pgn-text" bind:value={text} rows="12" spellcheck="false" aria-describedby="pgn-help"
		></textarea>
		<p id="pgn-help" class="dim">{t('pgn.help')}</p>
		{#if error}<p role="alert">{error}</p>{/if}
	</Frame>
	<div class="controls">
		<NavButton disabled={!text.trim()} onclick={open}>[ {t('action.open')} ]</NavButton>
		<NavButton onclick={back}>[ {t('action.back')} ]</NavButton>
	</div>
</div>

<style>
	.pgn {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 80ch;
	}
	label {
		display: block;
	}
	textarea {
		width: 100%;
		font: inherit;
		color: var(--fg);
		background: transparent;
		border: 1px dashed var(--dim);
		padding: 0.5ch;
		resize: vertical;
	}
	textarea:focus-visible {
		border-color: var(--fg);
		background: transparent;
		color: var(--fg);
	}
	.controls {
		display: flex;
		gap: 2ch;
	}
	p {
		margin: 0;
	}
</style>
