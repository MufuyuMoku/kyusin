<!--
  Menu utama: katalog dibangkitkan dari manifest di registry (SPEC §5.2).
-->
<script lang="ts">
	import { app, go } from '$lib/app.svelte';
	import { L, t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	function players(min: number, max: number) {
		return t('menu.players', { n: min === max ? min : `${min}–${max}` });
	}
</script>

<div class="menu">
	<Frame title={t('menu.catalog')}>
		{#if app.catalog.length === 0}
			<p>{t('menu.empty')}</p>
			<p class="dim">{t('menu.empty_hint')}</p>
		{:else}
			{#each app.catalog as cat (cat.key)}
				<div class="cat">
					<div class="dim">{L(cat.label).toUpperCase()}</div>
					<ul>
						{#each cat.games as g (g.id)}
							<li>
								<NavButton onclick={() => go({ name: 'game', id: g.id })}>▸ {L(g.nama)}</NavButton>
								<span class="dim"
									>{players(g.pemain_min, g.pemain_maks)}{g.rtp_line
										? ` · ${L(g.rtp_line)}`
										: ''}</span
								>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		{/if}
	</Frame>

	<Frame title={t('menu.system')}>
		<ul>
			<li>
				<NavButton onclick={() => go({ name: 'settings' })}>▸ {t('menu.settings')}</NavButton>
			</li>
			<li>
				<NavButton onclick={() => go({ name: 'help' })}>▸ {t('menu.help')}</NavButton>
			</li>
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
