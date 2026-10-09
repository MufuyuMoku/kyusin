<!--
  Baris status papan taruhan M6a: apa yang ditunggu dari pemain (dengan
  kata kerja game: putar, lempar, buka; Craps: titik atau come-out), atau
  hasil sesi.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isMejaView } from './meja';
	import { papanModel } from './papan';

	let { view, game = '', observer = false }: { view: unknown; game?: string; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isMejaView(view) ? (view as Record<string, any>) : null);
	const model = $derived(isMejaView(view) ? papanModel(game, view) : null);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
</script>

{#if v && model}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(`mj.over.${v.alasan ?? 'berhenti'}` as Key, { n: signed(v.bersih), rounds: v.ronde })}
		{:else if observer}
			{t('mj.observer', { n: v.ronde })}
		{:else if model.craps}
			{v.titik ? t('pp.status.point', { p: v.titik }) : t('pp.status.comeout')}
		{:else}
			{t(`pp.status.${model.verb}` as Key, { min: v.min_taruhan, max: v.maks_taruhan })}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
		/* Selebar papan: kalimat panjang dibungkus, tidak melebarkan kolom kiri. */
		max-width: 64ch;
	}
</style>
