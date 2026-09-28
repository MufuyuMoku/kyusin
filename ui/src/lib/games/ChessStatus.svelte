<!--
  Baris status catur: giliran, skak, dan langkah terakhir dengan keterangan
  langkah khusus (rokade, en passant, promosi) supaya tampil jelas (D-041).
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isChessView } from './catur';

	let {
		view,
		botTurn = false,
		observer = false
	}: { view: unknown; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isChessView(view) ? view : null);
	const colorOf = (seat: number) => t(seat === 0 ? 'catur.white' : 'catur.black');
	const PIECE: Record<string, Key> = {
		q: 'catur.piece.q',
		r: 'catur.piece.r',
		b: 'catur.piece.b',
		n: 'catur.piece.n'
	};

	/** Keterangan langkah khusus terakhir. */
	const note = $derived.by(() => {
		const l = v?.terakhir;
		if (!l) return '';
		if (l.rokade === 'pendek') return t('catur.note.castle_short');
		if (l.rokade === 'panjang') return t('catur.note.castle_long');
		if (l.en_passant) return t('catur.note.en_passant', { square: `${l.ke[0]}${l.dari[1]}` });
		if (l.promosi) return t('catur.note.promotion', { piece: t(PIECE[l.promosi]) });
		return '';
	});
</script>

{#if v}
	{#if v.terakhir}
		<p class="last">{t('catur.last', { san: v.terakhir.san })}{note ? ` ${note}` : ''}</p>
	{/if}
	<p class="status" role="status">
		{#if v.selesai}
			{t('catur.over')}
		{:else if observer && v.giliran !== null}
			{t('catur.their_turn', { color: colorOf(v.giliran) })}
		{:else if v.giliran === v.kamu}
			{t('catur.your_turn', { color: colorOf(v.kamu) })}
		{:else if botTurn}
			{t('match.bot_turn')}
		{:else if v.giliran !== null}
			{t('catur.their_turn', { color: colorOf(v.giliran) })}
		{/if}
		{#if v.skak && !v.selesai}<strong class="check">{t('catur.check_now')}</strong>{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
	.check {
		margin-left: 1ch;
		padding: 0 0.5ch;
		background: var(--fg);
		color: var(--bg);
	}
</style>
