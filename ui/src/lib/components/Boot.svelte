<!--
  Urutan boot singkat (SPEC §4): beberapa baris teks ala komputer lama yang
  sedang menyala. Bisa dilewati dengan tombol apa saja atau klik, dan bisa
  dimatikan di pengaturan. Saat reduced motion, teksnya tampil langsung
  tanpa animasi ketik (D-026).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n.svelte';
	import { motion } from '$lib/settings.svelte';

	let { cartridges, version, ondone }: { cartridges: number; version: string; ondone: () => void } =
		$props();

	// Nama produk dan label proyek tidak diterjemahkan.
	const lines = $derived([
		'PROJECT SINNERS',
		'',
		`KyuSin ${version}`,
		'',
		t('boot.memory'),
		t('boot.cartridges', { n: cartridges }),
		t('boot.phosphor'),
		'',
		t('boot.ready')
	]);

	let shown = $state(motion.reduced ? Infinity : 0);
	let finished = false;

	function done() {
		if (finished) return;
		finished = true;
		ondone();
	}

	onMount(() => {
		let tick: ReturnType<typeof setInterval> | undefined;
		let wait: ReturnType<typeof setTimeout> | undefined;
		if (motion.reduced) {
			wait = setTimeout(done, 1600);
		} else {
			tick = setInterval(() => {
				shown += 1;
				if (shown > lines.length) {
					clearInterval(tick);
					wait = setTimeout(done, 450);
				}
			}, 140);
		}
		const skip = () => done();
		window.addEventListener('keydown', skip, { once: true });
		window.addEventListener('pointerdown', skip, { once: true });
		return () => {
			clearInterval(tick);
			clearTimeout(wait);
			window.removeEventListener('keydown', skip);
			window.removeEventListener('pointerdown', skip);
		};
	});
</script>

<div class="boot" role="status" aria-live="polite">
	{#each lines.slice(0, shown) as line, i (i)}
		<div class:display={i === 0} class:title={i === 0}>{line || ' '}</div>
	{/each}
	<span class="cursor">&nbsp;</span>
	<div class="skip dim">{t('boot.skip')}</div>
</div>

<style>
	.boot {
		padding: 3rem 4ch;
		white-space: pre;
		height: 100%;
		position: relative;
	}
	.title {
		font-size: 3rem;
		line-height: 1;
	}
	.skip {
		position: absolute;
		bottom: 2rem;
		left: 4ch;
	}
</style>
