<!--
  Bingkai box-drawing di atas grid karakter (SPEC §4). Lebar dan tinggi
  dibulatkan ke kelipatan sel supaya garis selalu tersambung.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title = '',
		fill = false,
		children
	}: { title?: string; fill?: boolean; children: Snippet } = $props();

	let outerW = $state(0);
	let outerH = $state(0);
	let contentH = $state(0);
	let probeW = $state(0);
	let probeH = $state(0);

	const cw = $derived(probeW / 10 || 9);
	const lh = $derived(probeH || 22);
	const cols = $derived(Math.max(8, Math.floor(outerW / cw)));
	const rows = $derived(
		fill ? Math.max(3, Math.floor(outerH / lh)) : Math.max(3, Math.ceil(contentH / lh) + 2)
	);

	const top = $derived.by(() => {
		const label = title ? `─ ${title} ` : '';
		const fillLen = Math.max(0, cols - 2 - label.length);
		return '┌' + label + '─'.repeat(fillLen) + '┐';
	});
	const bottom = $derived('└' + '─'.repeat(Math.max(0, cols - 2)) + '┘');
	const side = $derived(Array.from({ length: Math.max(0, rows - 2) }, () => '│').join('\n'));
</script>

<div class="outer" class:fill bind:clientWidth={outerW} bind:clientHeight={outerH}>
	<span class="probe" aria-hidden="true" bind:clientWidth={probeW} bind:clientHeight={probeH}
		>0000000000</span
	>
	<div
		class="frame"
		style:width="{cols * cw}px"
		style:height={fill ? `${rows * lh}px` : undefined}
	>
		<div class="line" aria-hidden="true">{top}</div>
		<div class="mid">
			<pre class="side" aria-hidden="true">{side}</pre>
			<div class="content" bind:clientHeight={contentH}>
				{@render children()}
			</div>
			<pre class="side" aria-hidden="true">{side}</pre>
		</div>
		<div class="line" aria-hidden="true">{bottom}</div>
	</div>
</div>

<style>
	.outer {
		position: relative;
		width: 100%;
	}
	.outer.fill {
		position: absolute;
		inset: 0;
	}
	.probe {
		position: absolute;
		display: inline-block;
		visibility: hidden;
		white-space: pre;
	}
	.frame {
		display: flex;
		flex-direction: column;
	}
	.line {
		white-space: pre;
		overflow: hidden;
		color: var(--dim);
	}
	.mid {
		display: flex;
		flex: 1;
		min-height: 0;
	}
	.side {
		white-space: pre;
		color: var(--dim);
		overflow: hidden;
		flex: none;
		width: 1ch;
	}
	.content {
		flex: 1;
		min-width: 0;
		padding: 0 1ch;
		overflow: auto;
		align-self: flex-start;
		max-height: 100%;
	}
	.fill .content {
		align-self: stretch;
	}
</style>
