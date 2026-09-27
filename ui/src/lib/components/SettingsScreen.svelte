<!--
  Pengaturan (SPEC §4): tema fosfor, efek CRT per efek + intensitas,
  bahasa, mode urutan boot, dan konsol selalu tampil. Kotak centang selalu
  menunjukkan keadaan sebenarnya: efek yang dipaksa mati reduced motion
  tampil nonaktif dengan alasannya (D-026).
-->
<script lang="ts">
	import { back } from '$lib/app.svelte';
	import { t, lang, type Key } from '$lib/i18n.svelte';
	import {
		BOOT_MODES,
		EFFECTS,
		INTENSITY_STEP,
		LANGS,
		THEMES,
		effectOn,
		forcedOff,
		save,
		setIntensity,
		settings
	} from '$lib/settings.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	const BAR = 10;

	function mark(on: boolean) {
		return on ? '[■]' : '[ ]';
	}

	const filled = $derived(Math.round((settings.intensity / 100) * BAR));
</script>

<div class="settings">
	<Frame title={t('settings.theme')}>
		{#each THEMES as th (th)}
			<div>
				<NavButton
					pressed={settings.theme === th}
					onclick={() => {
						settings.theme = th;
						save();
					}}>{settings.theme === th ? '(●)' : '( )'} {t(`settings.theme.${th}` as Key)}</NavButton
				>
			</div>
		{/each}
	</Frame>

	<Frame title={t('settings.fx')}>
		{#each EFFECTS as fx (fx)}
			<div>
				<NavButton
					disabled={forcedOff(fx)}
					reason={t('settings.fx.reduced')}
					pressed={effectOn(fx)}
					onclick={() => {
						settings.fx[fx] = !settings.fx[fx];
						save();
					}}>{mark(effectOn(fx))} {t(`settings.fx.${fx}` as Key)}</NavButton
				>
			</div>
		{/each}
		<div class="intensity">
			<span>{t('settings.intensity')}</span>
			<NavButton
				label={t('settings.intensity.down')}
				disabled={settings.intensity <= 0}
				onclick={() => setIntensity(settings.intensity - INTENSITY_STEP)}>[-]</NavButton
			>
			<span aria-hidden="true"
				>{'■'.repeat(filled)}<span class="dim">{'□'.repeat(BAR - filled)}</span></span
			>
			<NavButton
				label={t('settings.intensity.up')}
				disabled={settings.intensity >= 100}
				onclick={() => setIntensity(settings.intensity + INTENSITY_STEP)}>[+]</NavButton
			>
			<span role="status">{settings.intensity}%</span>
		</div>
	</Frame>

	<Frame title={t('settings.language')}>
		{#each LANGS as l (l)}
			<div>
				<NavButton
					lang={l}
					pressed={lang() === l}
					onclick={() => {
						settings.lang = l;
						save();
					}}>{lang() === l ? '(●)' : '( )'} {t(`settings.lang.${l}` as Key)}</NavButton
				>
			</div>
		{/each}
	</Frame>

	<Frame title={t('settings.boot')}>
		{#each BOOT_MODES as mode (mode)}
			<div>
				<NavButton
					pressed={settings.bootMode === mode}
					onclick={() => {
						settings.bootMode = mode;
						save();
					}}>{settings.bootMode === mode ? '(●)' : '( )'} {t(`settings.boot.${mode}` as Key)}</NavButton
				>
			</div>
		{/each}
	</Frame>

	<Frame title={t('settings.other')}>
		<div>
			<NavButton
				pressed={settings.consoleAlways}
				onclick={() => {
					settings.consoleAlways = !settings.consoleAlways;
					save();
				}}>{mark(settings.consoleAlways)} {t('settings.console_always')}</NavButton
			>
		</div>
	</Frame>

	<div><NavButton onclick={back}>[ {t('action.back')} ]</NavButton></div>
</div>

<style>
	.settings {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 72ch;
	}
	.intensity {
		display: flex;
		gap: 1ch;
		margin-top: var(--cell-h);
	}
	.intensity > span:first-child {
		min-width: 12ch;
	}
</style>
