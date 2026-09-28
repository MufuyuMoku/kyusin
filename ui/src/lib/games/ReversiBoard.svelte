<!--
  Papan Reversi (SPEC §4, D-038): papan berpetak bersama + sprite piksel.
  Bidak putih (terang) diisi penuh, bidak hitam (gelap) berupa cincin.
  Main dengan klik atau keyboard (panah, Enter/Spasi).
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import GridBoard from './GridBoard.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { COLS, isReversiView, squareName } from './reversi';
	import { DISC_DARK, DISC_LIGHT } from './sprites';

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

	const ROWS = ['1', '2', '3', '4', '5', '6', '7', '8'];
	const v = $derived(isReversiView(view) ? view : null);
	const legal = $derived(new Set(interactive ? (v?.legal ?? []).filter((s) => s !== 'pass') : []));

	function at(sq: string): string {
		const r = Number(sq[1]) - 1;
		const c = COLS.indexOf(sq[0]);
		return v?.papan[r]?.[c] ?? '.';
	}

	function label(sq: string): string {
		const ch = at(sq);
		const state: Key =
			ch === 'X' ? 'reversi.cell.black' : ch === 'O' ? 'reversi.cell.white' : 'reversi.cell.empty';
		const parts = [`${sq}: ${t(state)}`];
		if (legal.has(sq)) parts.push(t('reversi.cell.legal'));
		if (v?.terakhir === sq) parts.push(t('reversi.cell.last'));
		return parts.join(', ');
	}

	/** Posisi awal kursor (SPEC §4): petak tengah. */
	const start = 'd4';
</script>

{#if v}
	<GridBoard
		colLabels={COLS.split('')}
		rowLabels={ROWS}
		squareOf={squareName}
		{label}
		ariaLabel={t('reversi.board')}
		{legal}
		{highlight}
		flipped={new Set(v.dibalik)}
		last={v.terakhir}
		{start}
		{onplay}
	>
		{#snippet piece(sq)}
			{#if at(sq) === 'O'}
				<PixelSprite pixels={DISC_LIGHT} size={2} />
			{:else if at(sq) === 'X'}
				<PixelSprite pixels={DISC_DARK} size={2} />
			{/if}
		{/snippet}
	</GridBoard>
{/if}
