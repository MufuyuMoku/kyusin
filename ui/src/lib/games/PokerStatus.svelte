<!--
  Baris status meja antar-pemain: giliran siapa, atau hasil sesi.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isTableView } from './meja_pvp';

	let { view, observer = false }: { view: unknown; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isTableView(view) ? view : null);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
</script>

{#if v}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(`pk.over.${v.alasan ?? 'berdiri'}` as Key, { n: signed(v.bersih), hands: v.tangan_ke })}
		{:else if observer}
			{t('pk.observer', { n: v.tangan_ke })}
		{:else if v.fase === 'antara'}
			{t('pk.status.between')}
		{:else if v.sideshow && v.sideshow[1] === v.kamu}
			{t('pk.status.sideshow')}
		{:else if v.giliran === v.kamu}
			{t('pk.status.turn')}
		{:else}
			{t('pk.status.waiting', { n: (v.giliran ?? v.sideshow?.[1] ?? 0) + 1 })}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
</style>
