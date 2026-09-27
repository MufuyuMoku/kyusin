<!--
  Pengaturan tampilan (SPEC §4): tema fosfor, efek CRT satu per satu,
  konsol selalu tampil, dan urutan boot.
-->
<script lang="ts">
	import { back } from '$lib/app.svelte';
	import { EFFECTS, THEMES, motion, save, settings } from '$lib/settings.svelte';
	import Frame from './Frame.svelte';

	function mark(on: boolean) {
		return on ? '[■]' : '[ ]';
	}
</script>

<div class="settings">
	<Frame title="TEMA FOSFOR">
		{#each THEMES as th (th.id)}
			<div>
				<button
					class="tbtn"
					data-nav
					aria-pressed={settings.theme === th.id}
					onclick={() => {
						settings.theme = th.id;
						save();
					}}>{settings.theme === th.id ? '(●)' : '( )'} {th.label}</button
				>
			</div>
		{/each}
	</Frame>

	<Frame title="EFEK CRT">
		{#if motion.reduced}
			<p class="dim">Sistem meminta reduced motion: semua efek dimatikan otomatis.</p>
		{/if}
		{#each EFFECTS as fx (fx.id)}
			<div>
				<button
					class="tbtn"
					data-nav
					aria-pressed={settings.fx[fx.id]}
					onclick={() => {
						settings.fx[fx.id] = !settings.fx[fx.id];
						save();
					}}>{mark(settings.fx[fx.id])} {fx.label}</button
				>
			</div>
		{/each}
	</Frame>

	<Frame title="LAIN-LAIN">
		<div>
			<button
				class="tbtn"
				data-nav
				aria-pressed={settings.consoleAlways}
				onclick={() => {
					settings.consoleAlways = !settings.consoleAlways;
					save();
				}}>{mark(settings.consoleAlways)} Konsol perintah selalu tampil</button
			>
		</div>
		<div>
			<button
				class="tbtn"
				data-nav
				aria-pressed={settings.boot}
				onclick={() => {
					settings.boot = !settings.boot;
					save();
				}}>{mark(settings.boot)} Urutan boot saat dibuka</button
			>
		</div>
	</Frame>

	<div><button class="tbtn" data-nav onclick={back}>[ KEMBALI ]</button></div>
</div>

<style>
	.settings {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 64ch;
	}
	p {
		margin: 0;
	}
</style>
