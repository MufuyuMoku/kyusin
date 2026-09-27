<!--
  Bantuan: perintah global, tombol keyboard, dan game dari manifest.
-->
<script lang="ts">
	import { app, back } from '$lib/app.svelte';
	import { helpText } from '$lib/help';
	import Frame from './Frame.svelte';
</script>

<div class="help">
	<Frame title="BANTUAN">
		<pre>{helpText(app.catalog)}</pre>
	</Frame>
	<div><button class="tbtn" data-nav onclick={back}>[ KEMBALI ]</button></div>
</div>

<style>
	.help {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 80ch;
	}
</style>
