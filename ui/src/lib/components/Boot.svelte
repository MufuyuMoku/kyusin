<!--
  Urutan boot singkat (SPEC §4): satu-satunya momen khas. Bisa dilewati
  dengan tombol apa saja atau klik, dan bisa dimatikan di pengaturan.
-->
<script lang="ts">
	import { onMount } from 'svelte';

	let { cartridges, version, ondone }: { cartridges: number; version: string; ondone: () => void } =
		$props();

	const lines = $derived([
		'PROJECT SINNERS',
		'',
		`KyuSin ${version}`,
		'',
		'MEMERIKSA MEMORI ............ OK',
		`MEMBACA CARTRIDGE ........... ${cartridges}`,
		'MENYALAKAN FOSFOR ........... OK',
		'',
		'SIAP.'
	]);

	let shown = $state(0);
	let finished = false;

	function done() {
		if (finished) return;
		finished = true;
		ondone();
	}

	onMount(() => {
		const tick = setInterval(() => {
			shown += 1;
			if (shown > lines.length) {
				clearInterval(tick);
				setTimeout(done, 450);
			}
		}, 140);
		const skip = () => done();
		window.addEventListener('keydown', skip, { once: true });
		window.addEventListener('pointerdown', skip, { once: true });
		return () => {
			clearInterval(tick);
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
	<div class="skip dim">tekan tombol apa saja untuk melewati</div>
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
