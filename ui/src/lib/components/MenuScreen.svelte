<!--
  Menu utama: katalog dibangkitkan dari manifest di registry (SPEC §5.2).
-->
<script lang="ts">
	import { app, go } from '$lib/app.svelte';
	import Frame from './Frame.svelte';
</script>

<div class="menu">
	<Frame title="KATALOG">
		{#if app.catalog.length === 0}
			<p>Belum ada cartridge terpasang.</p>
			<p class="dim">Game pertama (Reversi) datang di M1.</p>
		{:else}
			{#each app.catalog as cat (cat.key)}
				<div class="cat">
					<div class="dim">{cat.label.toUpperCase()}</div>
					<ul>
						{#each cat.games as g (g.id)}
							<li>
								<button class="tbtn" data-nav onclick={() => go({ name: 'game', id: g.id })}
									>▸ {g.nama}</button
								>
								<span class="dim"
									>{g.pemain_min === g.pemain_maks
										? g.pemain_min
										: `${g.pemain_min}–${g.pemain_maks}`} pemain{g.rtp_line
										? ` · ${g.rtp_line}`
										: ''}</span
								>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		{/if}
	</Frame>

	<Frame title="SISTEM">
		<ul>
			<li><button class="tbtn" data-nav onclick={() => go({ name: 'settings' })}>▸ Pengaturan</button></li>
			<li><button class="tbtn" data-nav onclick={() => go({ name: 'help' })}>▸ Bantuan</button></li>
		</ul>
	</Frame>
</div>

<style>
	.menu {
		display: grid;
		grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
		gap: 2ch;
		align-items: start;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	li {
		display: flex;
		gap: 2ch;
		align-items: baseline;
	}
	.cat + .cat {
		margin-top: var(--cell-h);
	}
	p {
		margin: 0;
	}
</style>
