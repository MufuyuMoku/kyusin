<!--
  Menu jeda pertandingan (SPEC §4 Rev. 9). Dibuka dengan `Esc` atau
  [ KELUAR ] di tengah permainan; fokus awal di Lanjutkan. Jam dan bot
  berhenti selama menu terbuka. `Esc` lagi = Lanjutkan.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { resignMatch, resumePlay, suspendMatch } from '$lib/app.svelte';
	import { NAV_SELECTOR } from '$lib/nav';
	import { t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let box: HTMLElement | undefined = $state();
	let confirmResign = $state(false);

	function focusFirst() {
		queueMicrotask(() =>
			box?.querySelector<HTMLElement>(NAV_SELECTOR)?.focus({ focusVisible: true } as FocusOptions)
		);
	}

	onMount(focusFirst);
</script>

<div class="backdrop">
	<div class="dialog" role="dialog" aria-modal="true" aria-label={t('pause.title')} bind:this={box}>
		<Frame title={t('pause.title')}>
			<div class="items">
				{#if !confirmResign}
					<NavButton onclick={resumePlay}>[ {t('action.continue')} ]</NavButton>
					<NavButton onclick={suspendMatch}>[ {t('action.suspend')} ]</NavButton>
					<NavButton
						onclick={() => {
							confirmResign = true;
							focusFirst();
						}}>[ {t('action.resign')} ]</NavButton
					>
					<p class="dim">{t('pause.note')}</p>
				{:else}
					<NavButton
						onclick={() => {
							confirmResign = false;
							focusFirst();
						}}>[ {t('action.cancel')} ]</NavButton
					>
					<NavButton onclick={resignMatch}>[ {t('action.resign_confirm')} ]</NavButton>
					<p class="dim">{t('pause.resign_note')}</p>
				{/if}
			</div>
		</Frame>
	</div>
</div>

<style>
	.backdrop {
		position: absolute;
		inset: 0;
		z-index: 5;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: calc(var(--cell-h) * 4);
		background: color-mix(in srgb, var(--bg) 80%, transparent);
	}
	.dialog {
		background: var(--bg);
		min-width: 36ch;
	}
	.items {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: calc(var(--cell-h) / 2);
	}
	p {
		margin: 0;
	}
</style>
