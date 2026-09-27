<!--
  Baris status Reversi: jumlah bidak dan giliran, dari view_data.
-->
<script lang="ts">
	import { t } from '$lib/i18n.svelte';
	import { isReversiView } from './reversi';

	let {
		view,
		botTurn = false,
		observer = false
	}: { view: unknown; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isReversiView(view) ? view : null);
	const colorOf = (seat: number) => t(seat === 0 ? 'reversi.black' : 'reversi.white');
</script>

{#if v}
	<p>{t('reversi.count', { b: v.hitam, w: v.putih })}</p>
	<p class="status" role="status">
		{#if v.selesai}
			{t('reversi.over')}
		{:else if observer && v.giliran !== null}
			{t('reversi.their_turn', { color: colorOf(v.giliran) })}
		{:else if v.giliran === v.kamu && v.legal[0] === 'pass'}
			{t('reversi.must_pass')}
		{:else if v.giliran === v.kamu}
			{t('reversi.your_turn', { color: colorOf(v.kamu) })}
		{:else if botTurn}
			{t('reversi.bot_turn')}
		{:else if v.giliran !== null}
			{t('reversi.their_turn', { color: colorOf(v.giliran) })}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
</style>
