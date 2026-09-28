<!--
  Papan berpetak bersama (SPEC §4, D-038): grid CSS dengan sel berukuran
  tetap, garis 1px warna fosfor, koordinat monospace. Isi sel (bidak) lewat
  snippet `piece`. Kursor keyboard, hover, sorotan tutorial, titik sasaran,
  petak terpilih, langkah terakhir, penanda skak, dan sasaran seret semuanya
  lapisan absolut di atas sel: tidak ada yang mengubah ukuran atau posisi
  sel mana pun. Bidak yang diseret digambar di lapisan `fixed` terpisah.

  Satu kontrol ber-role grid (ikut navigasi bersama): panah memindahkan
  kursor, Enter/Spasi memilih sel, Tab keluar; klik sel juga memilih. Seret
  (penunjuk atau sentuh) dari petak `draggable` ke petak lain memanggil
  `ondrop`.
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
		targets = undefined,
		highlight = new Set<string>(),
		flipped = new Set<string>(),
		last = null,
		selected = null,
		alert = null,
		start = null,
		onplay = () => {},
		draggable = undefined,
		ondrop = undefined,
		piece
	}: {
		colLabels: readonly string[];
		rowLabels: readonly string[];
		/** Nama petak, misalnya `d3`. */
		squareOf: (row: number, col: number) => string;
		/** Label aksesibel per petak. */
		label: (square: string) => string;
		ariaLabel: string;
		/** Petak yang bisa dipilih sekarang (klik/Enter). */
		legal?: Set<string>;
		/** Petak yang diberi titik sasaran; bawaan sama dengan `legal`. */
		targets?: Set<string>;
		/** Petak yang disorot tutorial. */
		highlight?: Set<string>;
		flipped?: Set<string>;
		/** Petak langkah terakhir (satu atau dua). */
		last?: string | readonly string[] | null;
		/** Petak yang sedang dipilih (bidak yang akan dijalankan). */
		selected?: string | null;
		/** Petak peringatan, misalnya raja yang diskak. */
		alert?: string | null;
		/** Petak awal kursor. */
		start?: string | null;
		onplay?: (square: string) => void;
		/** Petak yang bidaknya boleh diseret. */
		draggable?: (square: string) => boolean;
		ondrop?: (from: string, to: string) => void;
		piece: Snippet<[string]>;
	} = $props();

	const rows = $derived(rowLabels.length);
	const cols = $derived(colLabels.length);
	const dots = $derived(targets ?? legal);
	const lastSet = $derived(new Set(last === null ? [] : typeof last === 'string' ? [last] : last));
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

	// --- Seret-lepas ------------------------------------------------------
	let drag = $state<{ from: string; x0: number; y0: number; x: number; y: number; active: boolean } | null>(
		null
	);
	let over = $state<string | null>(null);
	/**
	 * Klik yang menyusul pelepasan seret diabaikan, tetapi hanya sesaat:
	 * peramban tidak selalu mengirim klik penutup seret, dan klik pemain
	 * berikutnya tidak boleh ikut tertelan.
	 */
	let suppressUntil = 0;

	function cellAt(x: number, y: number): HTMLElement | null {
		return (document.elementFromPoint(x, y) as HTMLElement | null)?.closest<HTMLElement>('[data-sq]') ?? null;
	}

	function onpointerdown(e: PointerEvent) {
		if (e.button !== 0 || !draggable || !ondrop) return;
		const cell = (e.target as HTMLElement).closest<HTMLElement>('[data-sq]');
		const sq = cell?.dataset.sq;
		if (!sq || !draggable(sq)) return;
		drag = { from: sq, x0: e.clientX, y0: e.clientY, x: e.clientX, y: e.clientY, active: false };
		(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
	}

	function onpointermove(e: PointerEvent) {
		if (!drag) return;
		drag.x = e.clientX;
		drag.y = e.clientY;
		if (!drag.active && Math.hypot(drag.x - drag.x0, drag.y - drag.y0) > 4) drag.active = true;
		if (drag.active) over = cellAt(e.clientX, e.clientY)?.dataset.sq ?? null;
	}

	function onpointerup(e: PointerEvent) {
		const was = drag;
		drag = null;
		over = null;
		if (!was?.active) return;
		suppressUntil = performance.now() + 150;
		const to = cellAt(e.clientX, e.clientY)?.dataset.sq;
		if (to && to !== was.from) ondrop?.(was.from, to);
	}

	function onclick(e: MouseEvent) {
		if (performance.now() < suppressUntil) return;
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
		{onpointerdown}
		{onpointermove}
		{onpointerup}
		onpointercancel={() => {
			drag = null;
			over = null;
		}}
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
						<div class="content" class:flip={flipped.has(sq)} class:lifted={drag?.active && drag.from === sq}>
							{@render piece(sq)}
						</div>
						{#if dots.has(sq)}<div class="layer dot"><PixelSprite pixels={DOT} size={2} /></div>{/if}
						{#if lastSet.has(sq)}<div class="layer last"></div>{/if}
						{#if selected === sq}<div class="layer selected"></div>{/if}
						{#if alert === sq}<div class="layer alert"></div>{/if}
						{#if highlight.has(sq)}<div class="layer sorot"></div>{/if}
						{#if over === sq}<div class="layer drop"></div>{/if}
						<div class="layer hover"></div>
						{#if focused && cursor.row === r && cursor.col === c}<div class="layer cursor-ring"></div>{/if}
					</div>
				{/each}
			</div>
		{/each}
	</div>
	{#if drag?.active}
		<div class="ghost" aria-hidden="true" style:left={`${drag.x}px`} style:top={`${drag.y}px`}>
			{@render piece(drag.from)}
		</div>
	{/if}
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
		touch-action: none;
		user-select: none;
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
	.lifted {
		opacity: 0.25;
	}
	.dot {
		color: var(--fg);
	}
	.last {
		inset: 3px;
		border: 1px dotted var(--fg);
	}
	.selected {
		inset: 0;
		border: 3px double var(--fg);
	}
	.alert {
		inset: 4px;
		border: 2px dashed var(--fg);
		animation: blink 0.8s steps(1) infinite;
	}
	.drop {
		inset: 2px;
		border: 2px solid var(--fg);
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
	.ghost {
		position: fixed;
		z-index: 60;
		transform: translate(-50%, -50%);
		pointer-events: none;
		color: var(--fg);
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
		.flip,
		.alert {
			animation: none;
		}
	}
</style>
