<!--
  Baris status casino lotere M6b: apa yang ditunggu dari pemain menurut
  game dan fase, atau hasil pertandingan.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { isMejaView } from './meja';
	import { signed } from './lotere';

	let { view, game = '', observer = false }: { view: unknown; game?: string; botTurn?: boolean; observer?: boolean } = $props();

	const v = $derived(isMejaView(view) ? view : null);
</script>

{#if v}
	<p class="status" role="status">
		{#if v.fase === 'selesai'}
			{t(`lt.over.${v.alasan ?? 'berhenti'}` as Key, { n: signed(v.bersih) })}
		{:else if observer}
			{t('mj.observer', { n: v.ronde })}
		{:else}
			{t(`lt.status.${game}.${v.fase}` as Key, { min: v.min_taruhan, max: v.maks_taruhan })}
		{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
		max-width: 64ch;
	}
</style>
