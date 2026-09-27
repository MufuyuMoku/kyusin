<!--
  Tutorial interaktif (SPEC §7.6). Pemain dipandu lewat kontrol visual:
  tombol yang harus disentuh disorot, perintah teks padanannya tampil kecil
  sebagai info. Kontrol di sini generik (dari ActionSpec); game katalog
  nanti memakai kontrol visualnya sendiri dengan target `sorot` yang sama.
-->
<script lang="ts">
	import { app, back, tutorialAct, tutorialNext } from '$lib/app.svelte';
	import Frame from './Frame.svelte';

	const t = $derived(app.tutorial);
	const highlighted = $derived(
		new Set((t?.step?.sorot ?? []).filter((s) => s.startsWith('aksi:')).map((s) => s.slice(5)))
	);
	const buttons = $derived(t?.actions.flatMap((a) => a.concrete ?? []) ?? []);

	function label(command: string) {
		return `[ ${command.toUpperCase()} ]`;
	}
</script>

{#if t}
	<div class="tut">
		<div class="head">
			<span class="display big">{t.title}</span>
			<span class="dim">
				{t.finished ? 'selesai' : `langkah ${t.index + 1}/${t.total}`}
			</span>
			<button class="tbtn" data-nav onclick={back}>[ KELUAR ]</button>
		</div>

		<Frame title="PAPAN">
			<pre class="view">{t.view_text}</pre>
		</Frame>

		<Frame title="TUTORIAL">
			{#if t.finished}
				<p>Tutorial selesai.</p>
				<div class="controls">
					<button class="tbtn" data-nav onclick={back}>[ KEMBALI ]</button>
				</div>
			{:else if t.step}
				<p>{t.step.teks}</p>
				<div class="controls">
					{#if t.step.aksi}
						{#each buttons as cmd (cmd)}
							<button
								class="tbtn"
								class:sorot={highlighted.has(cmd)}
								data-nav
								onclick={() => tutorialAct(cmd)}>{label(cmd)}</button
							>
						{/each}
					{:else}
						<button class="tbtn sorot" data-nav onclick={tutorialNext}>[ LANJUT ]</button>
					{/if}
				</div>
				{#if t.feedback?.kind === 'wrong'}
					<p class="hint" role="alert">! {t.feedback.hint}</p>
				{/if}
				{#if t.step.aksi}
					<p class="dim small">perintah: {t.step.aksi}</p>
				{/if}
			{/if}
		</Frame>
	</div>
{/if}

<style>
	.tut {
		display: flex;
		flex-direction: column;
		gap: var(--cell-h);
	}
	.head {
		display: flex;
		align-items: baseline;
		gap: 2ch;
	}
	.head .tbtn {
		margin-left: auto;
	}
	.big {
		font-size: 2rem;
	}
	.view {
		white-space: pre;
	}
	p {
		margin: 0;
	}
	.controls {
		display: flex;
		flex-wrap: wrap;
		gap: 1ch 2ch;
		margin: var(--cell-h) 0;
	}
	.sorot {
		outline: 1px solid var(--fg);
		animation: sorot 1.06s steps(1) infinite;
	}
	@keyframes sorot {
		50% {
			background: var(--fg);
			color: var(--bg);
		}
	}
	.hint {
		margin-bottom: var(--cell-h);
	}
	.small {
		font-size: 0.85em;
	}
</style>
