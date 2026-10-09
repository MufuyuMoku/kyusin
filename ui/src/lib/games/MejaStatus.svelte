<!--
  Baris status meja casino M5a: apa yang ditunggu dari pemain, atau hasil
  sesi.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isMejaView } from './meja';

	let { view, game = '', observer = false }: { view: unknown; game?: string; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isMejaView(view) ? view : null);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
</script>

{#if v}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(`mj.over.${v.alasan ?? 'berhenti'}` as Key, { n: signed(v.bersih), rounds: v.ronde })}
		{:else if observer}
			{t('mj.observer', { n: v.ronde })}
		{:else if v.fase === 'taruhan'}
			{t('mj.status.taruhan', { min: v.min_taruhan, max: v.maks_taruhan })}
		{:else if game === 'pai-gow-ubin' && v.fase === 'susun'}
			{t('mj.pgu.status')}
		{:else}
			{t(`mj.status.${v.fase}` as Key)}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
		max-width: 64ch;
	}
</style>
