<!--
  Papan berpetak bersama (SPEC §4, D-038): grid CSS dengan sel berukuran
  tetap, garis 1px warna fosfor, koordinat monospace. Isi sel (bidak) lewat
  snippet `piece`. Kursor keyboard, hover, sorotan tutorial, penanda langkah
  sah, dan penanda langkah terakhir semuanya lapisan absolut di atas sel:
  tidak ada yang mengubah ukuran atau posisi sel mana pun.

  Satu kontrol ber-role grid (ikut navigasi bersama): panah memindahkan
  kursor, Enter/Spasi memilih sel, Tab keluar; klik sel juga memilih.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { DOT } from './sprites';

	let {
		colLabels,
		rowLabels,
		squareOf,
		label,
		ariaLabel,
		legal = new Set<string>(),
		highlight = new Set<string>(),
		flipped = new Set<string>(),
		last = null,
		start = null,
		onplay = () => {},
		piece
	}: {
		colLabels: readonly string[];
		rowLabels: readonly string[];
		/** Nama petak (juga perintah teksnya), misalnya `d3`. */
		squareOf: (row: number, col: number) => string;
		/** Label aksesibel per petak. */
		label: (square: string) => string;
		ariaLabel: string;
		/** Petak yang boleh dipilih sekarang. */
		legal?: Set<string>;
		/** Petak yang disorot tutorial. */
		highlight?: Set<string>;
		flipped?: Set<string>;
		last?: string | null;
		/** Petak awal kursor. */
		start?: string | null;
		onplay?: (square: string) => void;
		piece: Snippet<[string]>;
	} = $props();

	const rows = $derived(rowLabels.length);
	const cols = $derived(colLabels.length);
	const uid = `gb-${Math.random().toString(36).slice(2, 8)}`;

	let cursor = $state({ row: 0, col: 0 });
	let focused = $state(false);

	$effect(() => {
		if (!start) return;
		for (let r = 0; r < rows; r++)
			for (let c = 0; c < cols; c++) if (squareOf(r, c) === start) cursor = { row: r, col: c };
	});

	function choose(sq: string) {
		if (legal.has(sq)) onplay(sq);
	}

	function onclick(e: MouseEvent) {
		const cell = (e.target as HTMLElement).closest<HTMLElement>('[data-sq]');
		if (!cell) return;
		cursor = { row: Number(cell.dataset.row), col: Number(cell.dataset.col) };
		choose(cell.dataset.sq!);
	}

	function onkeydown(e: KeyboardEvent) {
		const moves: Record<string, [number, number]> = {
			ArrowUp: [-1, 0],
			ArrowDown: [1, 0],
			ArrowLeft: [0, -1],
			ArrowRight: [0, 1]
		};
		const d = moves[e.key];
		if (d) {
			e.preventDefault();
			e.stopPropagation();
			cursor = {
				row: Math.min(rows - 1, Math.max(0, cursor.row + d[0])),
				col: Math.min(cols - 1, Math.max(0, cursor.col + d[1]))
			};
		} else if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			e.stopPropagation();
			choose(squareOf(cursor.row, cursor.col));
		}
	}
</script>

<div class="wrap" style:--cols={cols}>
	<div class="corner" aria-hidden="true"></div>
	<div class="col-labels dim" aria-hidden="true">
		{#each colLabels as c (c)}<span>{c}</span>{/each}
	</div>
	<div class="row-labels dim" aria-hidden="true">
		{#each rowLabels as r (r)}<span>{r}</span>{/each}
	</div>
	<div
		class="grid"
		role="grid"
		tabindex="0"
		data-nav
		aria-label={ariaLabel}
		aria-activedescendant={`${uid}-${squareOf(cursor.row, cursor.col)}`}
		{onkeydown}
		{onclick}
		onfocus={() => (focused = true)}
		onblur={() => (focused = false)}
	>
		{#each rowLabels as _, r (r)}
			<div role="row" class="row">
				{#each colLabels as _, c (c)}
					{@const sq = squareOf(r, c)}
					<div
						role="gridcell"
						class="cell"
						class:can={legal.has(sq)}
						id={`${uid}-${sq}`}
						data-sq={sq}
						data-row={r}
						data-col={c}
						aria-label={label(sq)}
						aria-selected={cursor.row === r && cursor.col === c}
					>
						<div class="content" class:flip={flipped.has(sq)}>{@render piece(sq)}</div>
						{#if legal.has(sq)}<div class="layer dot"><PixelSprite pixels={DOT} size={2} /></div>{/if}
						{#if last === sq}<div class="layer last"></div>{/if}
						{#if highlight.has(sq)}<div class="layer sorot"></div>{/if}
						<div class="layer hover"></div>
						{#if focused && cursor.row === r && cursor.col === c}<div class="layer cursor-ring"></div>{/if}
					</div>
				{/each}
			</div>
		{/each}
	</div>
</div>

<style>
	.wrap {
		--cell: 40px;
		--label: 3ch;
		display: inline-grid;
		grid-template-columns: var(--label) auto;
		grid-template-rows: var(--cell-h) auto;
	}
	.col-labels {
		display: grid;
		grid-template-columns: repeat(var(--cols), var(--cell));
		column-gap: 1px;
		padding-left: 1px;
		text-align: center;
	}
	.row-labels {
		display: grid;
		grid-auto-rows: var(--cell);
		row-gap: 1px;
		padding-top: 1px;
		align-items: center;
	}
	/* Garis 1px = celah grid di atas latar redup; sel tidak pernah berubah ukuran. */
	.grid {
		display: grid;
		grid-template-columns: repeat(var(--cols), var(--cell));
		grid-auto-rows: var(--cell);
		gap: 1px;
		padding: 1px;
		background: var(--dim);
		outline: none;
		width: max-content;
	}
	.grid:focus-visible {
		background: var(--dim);
		outline: 1px dashed var(--fg);
		outline-offset: 3px;
	}
	.row {
		display: contents;
	}
	.cell {
		position: relative;
		box-sizing: border-box;
		width: var(--cell);
		height: var(--cell);
		background: var(--bg);
		color: var(--fg);
		overflow: hidden;
	}
	.cell.can {
		cursor: pointer;
	}
	.content,
	.layer {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		pointer-events: none;
	}
	.dot {
		color: var(--dim);
	}
	.cell.can .dot {
		color: var(--fg);
	}
	.last {
		inset: 3px;
		border: 1px dotted var(--fg);
	}
	.hover {
		inset: 2px;
		border: 1px dashed transparent;
	}
	.cell:hover .hover {
		border-color: var(--dim);
	}
	.cell.can:hover .hover {
		border-color: var(--fg);
	}
	/* Bukan `.cursor`: nama itu dipakai kursor blok konsol di app.css. */
	.cursor-ring {
		inset: 1px;
		border: 2px solid var(--fg);
	}
	.sorot {
		inset: 1px;
		border: 2px solid var(--fg);
		animation: blink 1.06s steps(1) infinite;
	}
	.flip {
		animation: flip 0.35s steps(3) 1;
	}
	@keyframes blink {
		50% {
			border-color: transparent;
		}
	}
	@keyframes flip {
		0% {
			opacity: 0.2;
		}
		100% {
			opacity: 1;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.sorot,
		.flip {
			animation: none;
		}
	}
</style>
