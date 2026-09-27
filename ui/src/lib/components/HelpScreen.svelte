<!--
  Bantuan: perintah global, tombol keyboard, dan game dari manifest.
-->
<script lang="ts">
	import { app, back } from '$lib/app.svelte';
	import { helpText } from '$lib/help';
	import { t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
</script>

<div class="help">
	<Frame title={t('help.frame')}>
		<pre>{helpText(app.catalog)}</pre>
	</Frame>
	<div><button class="tbtn" data-nav onclick={back}>[ {t('action.back')} ]</button></div>
</div>

<style>
	.help {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 80ch;
	}
</style>
