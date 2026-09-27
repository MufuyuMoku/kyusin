<!--
  Sprite piksel SVG buatan sendiri (SPEC §4, D-038): grafis ala komputer
  80-an yang tidak bergantung pada font. Diwarnai `currentColor`, jadi
  mengikuti token tema lewat CSS.
-->
<script lang="ts">
	let { pixels, size = 2 }: { pixels: readonly string[]; size?: number } = $props();

	/** Gabungkan piksel beruntun per baris menjadi satu persegi panjang. */
	const runs = $derived.by(() => {
		const out: { x: number; y: number; w: number }[] = [];
		pixels.forEach((row, y) => {
			let start = -1;
			for (let x = 0; x <= row.length; x++) {
				const on = row[x] === 'X';
				if (on && start < 0) start = x;
				if (!on && start >= 0) {
					out.push({ x: start, y, w: x - start });
					start = -1;
				}
			}
		});
		return out;
	});
	const w = $derived(pixels[0]?.length ?? 0);
	const h = $derived(pixels.length);
</script>

<svg
	width={w * size}
	height={h * size}
	viewBox={`0 0 ${w} ${h}`}
	shape-rendering="crispEdges"
	aria-hidden="true"
	focusable="false"
>
	{#each runs as r, i (i)}<rect x={r.x} y={r.y} width={r.w} height="1" fill="currentColor" />{/each}
</svg>

<style>
	svg {
		display: block;
	}
</style>
