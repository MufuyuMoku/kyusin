<!--
  Baris status Blackjack: apa yang ditunggu dari pemain, atau hasil shoe.
-->
<script lang="ts">
	import { t } from '$lib/i18n.svelte';
	import { isBlackjackView } from './blackjack';

	let { view, observer = false }: { view: unknown; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isBlackjackView(view) ? view : null);
</script>

{#if v}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(v.alasan === 'shoe_habis' ? 'bj.over.shoe' : 'bj.over.left', { n: v.bersih > 0 ? `+${v.bersih}` : String(v.bersih) })}
		{:else if observer}
			{t('bj.observer', { n: v.ronde })}
		{:else if v.fase === 'taruhan'}
			{t('bj.status.bet', { min: v.min_taruhan, max: v.maks_taruhan })}
		{:else if v.fase === 'asuransi'}
			{t('bj.status.insurance')}
		{:else}
			{t('bj.status.turn', { n: (v.aktif ?? 0) + 1 })}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
</style>
