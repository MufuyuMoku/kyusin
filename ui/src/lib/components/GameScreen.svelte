<!--
  Layar info game: halaman `man <id>` dari manifest + tutorial (SPEC §7.6),
  dengan pintu masuk ke tutorial. Bermain melawan bot datang di M1.
-->
<script lang="ts">
	import { api } from '$lib/backend';
	import { back, startTutorial } from '$lib/app.svelte';
	import { L, errorText, t, type Localized } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';

	let { id }: { id: string } = $props();

	let page = $state<Localized | null>(null);
	let error = $state<unknown>(null);

	$effect(() => {
		const target = id;
		error = null;
		api()
			.then((a) => a.man(target))
			.then((p) => (page = p))
			.catch((e) => (error = e));
	});
</script>

<div class="game">
	<div class="actions">
		<button class="tbtn" data-nav onclick={() => startTutorial(id)}>[ {t('action.tutorial')} ]</button>
		<button class="tbtn" data-nav onclick={back}>[ {t('action.back')} ]</button>
	</div>
	<div class="grow">
		<Frame title={`man ${id}`} fill>
			{#if error}
				<p>{errorText(error)}</p>
			{:else}
				<pre>{L(page)}</pre>
			{/if}
		</Frame>
	</div>
</div>

<style>
	.game {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		height: 100%;
	}
	.grow {
		position: relative;
		flex: 1;
		min-height: calc(var(--cell-h) * 6);
	}
	.actions {
		display: flex;
		gap: 2ch;
	}
	p {
		margin: 0;
	}
</style>
