<!--
  Baris status meja Capsa Susun: menyusun, menunggu, di antara tangan, atau
  hasil sesi.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isCapsaView } from './capsa';

	let { view }: { view: unknown; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isCapsaView(view) ? view : null);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
</script>

{#if v}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(`pk.over.${v.alasan ?? 'berdiri'}` as Key, { n: signed(v.bersih), hands: v.tangan_ke })}
		{:else if v.fase === 'antara'}
			{t('pk.status.between')}
		{:else if v.saran}
			{t('cs.status_line.arrange')}
		{:else}
			{t('cs.status_line.waiting')}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
</style>
