<!--
  Profil (SPEC §9 M3, D-050): satu profil per instalasi, namanya bisa
  diganti. Nama dipakai sapaan boot; kosong = tanpa nama.
-->
<script lang="ts">
	import { api, type ProfileState } from '$lib/backend';
	import { app, go } from '$lib/app.svelte';
	import { errorText, lang, t } from '$lib/i18n.svelte';
	import Frame from './Frame.svelte';
	import NavButton from './NavButton.svelte';

	let profile = $state<ProfileState | null>(null);
	let draft = $state('');
	let saved = $state(false);
	let error = $state<unknown>(null);

	$effect(() => {
		api()
			.then((a) => a.profile_get())
			.then((p) => {
				profile = p;
				draft = p.name ?? '';
			})
			.catch((e) => (error = e));
	});

	async function save() {
		try {
			const a = await api();
			profile = await a.profile_set_name(draft);
			draft = profile.name ?? '';
			saved = true;
			// Sapaan boot berikutnya memakai nama baru.
			if (app.info) app.info.profile = { ...app.info.profile, name: profile.name ?? undefined };
		} catch (e) {
			error = e;
		}
	}

	function date(ms: number): string {
		return new Intl.DateTimeFormat(lang(), { dateStyle: 'medium' }).format(ms);
	}
</script>

<div class="profile">
	<Frame title={t('profile.title')}>
		{#if error}
			<p role="alert">{errorText(error)}</p>
		{/if}
		{#if profile}
			<p class="current">
				{t('profile.name')}: <span class="name">{profile.name ?? t('profile.unnamed')}</span>
			</p>
			<form
				class="row"
				onsubmit={(e) => {
					e.preventDefault();
					save();
				}}
			>
				<label for="profile-name">{t('profile.name')}</label>
				<input
					id="profile-name"
					class="field"
					bind:value={draft}
					oninput={() => (saved = false)}
					spellcheck="false"
					autocomplete="off"
					maxlength={profile.name_max}
					aria-describedby="profile-hint"
				/>
				<NavButton onclick={save}>[ {t('profile.save')} ]</NavButton>
			</form>
			<p class="dim" id="profile-hint">{t('profile.hint', { n: profile.name_max })}</p>
			{#if saved}<p role="status">{t('profile.saved')}</p>{/if}
			<p class="dim">{t('profile.created', { date: date(profile.created_at) })}</p>
			<p class="dim">{t('profile.note')}</p>
		{/if}
	</Frame>
	<div>
		<NavButton onclick={() => go({ name: 'stats' })}>[ {t('action.stats')} ]</NavButton>
	</div>
</div>

<style>
	.profile {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
		max-width: 72ch;
	}
	.row {
		display: flex;
		align-items: baseline;
		gap: 2ch;
		margin: calc(var(--cell-h) / 2) 0;
	}
	.field {
		width: 26ch;
		max-width: 100%;
		font: inherit;
		color: var(--fg);
		background: transparent;
		border: 0;
		border-bottom: 1px dashed var(--dim);
		padding: 0 0.5ch;
		caret-color: var(--fg);
	}
	.field:focus-visible {
		background: transparent;
		color: var(--fg);
		border-bottom: 1px solid var(--fg);
	}
	p {
		margin: 0;
	}
	.name {
		font-family: var(--font-display);
		font-size: 1.4rem;
	}
</style>
