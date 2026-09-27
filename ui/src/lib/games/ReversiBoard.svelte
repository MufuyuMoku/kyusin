<!--
  Papan Reversi di grid karakter (SPEC §4): garis box-drawing, █ hitam,
  ░ putih, · langkah sah, [█] langkah terakhir. Main dengan klik, atau
  keyboard: panah memindahkan kursor, Enter/Spasi meletakkan bidak, Tab
  keluar dari papan. Satu elemen yang bisa difokus (role grid) dengan
  aria-activedescendant, jadi tetap satu langkah di navigasi bersama.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import { COLS, isReversiView, squareName } from './reversi';

	let {
		view,
		interactive = false,
		highlight = new Set<string>(),
		onplay = () => {}
	}: {
		view: unknown;
		interactive?: boolean;
		/** Perintah yang disorot tutorial (mis. `d3`). */
		highlight?: Set<string>;
		onplay?: (command: string) => void;
	} = $props();

	const v = $derived(isReversiView(view) ? view : null);
	const legal = $derived(new Set(interactive ? (v?.legal ?? []) : []));
	const flipped = $derived(new Set(v?.dibalik ?? []));
	const uid = `rv-${Math.random().toString(36).slice(2, 8)}`;

	let cursor = $state({ row: 3, col: 3 });
	let focused = $state(false);

	// Kursor mulai di langkah yang disorot atau langkah sah pertama.
	$effect(() => {
		const target = [...highlight].find((h) => legal.has(h)) ?? v?.legal.find((s) => s !== 'pass');
		if (target && target !== 'pass') {
			cursor = { row: Number(target[1]) - 1, col: COLS.indexOf(target[0]) };
		}
	});

	function glyph(ch: string, sq: string): string {
		if (ch === 'X') return '█';
		if (ch === 'O') return '░';
		return legal.has(sq) ? '·' : ' ';
	}

	function cellText(ch: string, sq: string): string {
		const g = glyph(ch, sq);
		return v?.terakhir === sq ? `[${g}]` : ` ${g} `;
	}

	function label(ch: string, sq: string): string {
		const state: Key = ch === 'X' ? 'reversi.cell.black' : ch === 'O' ? 'reversi.cell.white' : 'reversi.cell.empty';
		const parts = [`${sq}: ${t(state)}`];
		if (legal.has(sq)) parts.push(t('reversi.cell.legal'));
		if (v?.terakhir === sq) parts.push(t('reversi.cell.last'));
		return parts.join(', ');
	}

	function play(sq: string) {
		if (legal.has(sq)) onplay(sq);
	}

	/** Klik pada sel (delegasi dari papan). */
	function onclick(e: MouseEvent) {
		const sq = (e.target as HTMLElement).closest<HTMLElement>('[data-sq]')?.dataset.sq;
		if (!sq) return;
		cursor = { row: Number(sq[1]) - 1, col: COLS.indexOf(sq[0]) };
		play(sq);
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
				row: Math.min(7, Math.max(0, cursor.row + d[0])),
				col: Math.min(7, Math.max(0, cursor.col + d[1]))
			};
		} else if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			e.stopPropagation();
			play(squareName(cursor.row, cursor.col));
		}
	}

	// Spasi awal harus lewat ekspresi: Svelte merapikan spasi awal teks markup.
	const HEADER = '    a   b   c   d   e   f   g   h';
	const TOP = '  ┌───┬───┬───┬───┬───┬───┬───┬───┐';
	const SEP = '├───┼───┼───┼───┼───┼───┼───┼───┤';
	const BOTTOM = '  └───┴───┴───┴───┴───┴───┴───┴───┘';
</script>

{#if v}
	<div
		class="board"
		role="grid"
		tabindex="0"
		data-nav
		aria-label={t('reversi.board')}
		aria-activedescendant={`${uid}-${squareName(cursor.row, cursor.col)}`}
		{onkeydown}
		{onclick}
		onfocus={() => (focused = true)}
		onblur={() => (focused = false)}
	>
		<div class="line dim" aria-hidden="true">{HEADER}</div>
		<div class="line dim" aria-hidden="true">{TOP}</div>
		{#each v.papan as row, r (r)}
			<div class="line" role="row">
				<span class="dim" aria-hidden="true">{r + 1} │</span>{#each row.split('') as ch, c (c)}
					{@const sq = squareName(r, c)}
					<span
						role="gridcell"
						id={`${uid}-${sq}`}
						data-sq={sq}
						tabindex="-1"
						class="cell"
						class:legal={legal.has(sq)}
						class:sorot={highlight.has(sq)}
						class:cursor={focused && cursor.row === r && cursor.col === c}
						class:flip={flipped.has(sq)}
						aria-label={label(ch, sq)}
						aria-selected={cursor.row === r && cursor.col === c}>{cellText(ch, sq)}</span
					><span class="dim" aria-hidden="true">│</span>{/each}
			</div>
			<div class="line dim" aria-hidden="true">{r === 7 ? BOTTOM : `  ${SEP}`}</div>
		{/each}
	</div>
{/if}

<style>
	/* Hanya isi baris yang `pre`; spasi template di antara baris diabaikan. */
	.board {
		display: inline-block;
		white-space: normal;
		line-height: var(--cell-h);
		outline: none;
	}
	.board:focus-visible {
		background: none;
		color: inherit;
		outline: 1px dashed var(--dim);
		outline-offset: 2px;
	}
	.line {
		display: block;
		white-space: pre;
	}
	.cell {
		cursor: default;
	}
	.cell.legal {
		cursor: pointer;
	}
	.cell.legal:hover,
	.cell.cursor {
		background: var(--fg);
		color: var(--bg);
		text-shadow: none;
	}
	.cell.sorot {
		outline: 1px solid var(--fg);
		animation: sorot 1.06s steps(1) infinite;
	}
	.cell.flip {
		animation: flip 0.35s steps(3) 1;
	}
	@keyframes sorot {
		50% {
			background: var(--fg);
			color: var(--bg);
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
		.cell.sorot,
		.cell.flip {
			animation: none;
		}
	}
</style>
