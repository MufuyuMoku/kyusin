<!--
  Baris penyusun taruhan bersama casino lotere M6b: jumlah taruhan, tombol
  chip menurut kelipatan meja, dan ULANG. Lebar jumlah tetap, jadi angka
  baru tidak menggeser tombol.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t } from '$lib/i18n.svelte';
	import { chipSteps } from './meja';

	let {
		amount = $bindable(),
		min,
		max,
		step
	}: {
		amount: number;
		min: number;
		max: number;
		step: number;
	} = $props();
</script>

<div class="bet-row">
	<span class="amount" aria-live="polite">{t('bj.bet_amount', { n: amount })}</span>
	{#each chipSteps(step) as n (n)}
		<NavButton disabled={amount + n > max} onclick={() => (amount = Math.min(max, amount + n))}>[ +{n} ]</NavButton>
	{/each}
	<NavButton onclick={() => (amount = min)}>[ {t('bj.clear')} ]</NavButton>
</div>

<style>
	.bet-row {
		display: flex;
		flex-wrap: wrap;
		gap: 1ch 2ch;
		align-items: baseline;
	}
	.amount {
		width: 16ch;
	}
</style>
