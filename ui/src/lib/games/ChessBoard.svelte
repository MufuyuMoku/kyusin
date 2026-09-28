<!--
  Papan catur (SPEC §4, §6.1; D-041, D-043). Papan berpetak bersama +
  sprite piksel: bidak putih penuh, bidak hitam bergaris tepi.

  Input visual: klik bidak lalu klik tujuan, atau seret bidak ke tujuan.
  Saat bidak dipilih, tujuan sahnya diberi titik. Promosi memunculkan
  pilihan bergambar. Keyboard: panah + Enter/Spasi dengan urutan yang sama.
  Yang dikirim ke backend selalu SAN dari daftar langkah sah.
-->
<script lang="ts">
	import { t, type Key } from '$lib/i18n.svelte';
	import NavButton from '$lib/components/NavButton.svelte';
	import GridBoard from './GridBoard.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { FILES, PROMOTIONS, isChessView, pieceAt, squareAt, type ChessMove } from './catur';
	import { CHESS, outline, type ChessPiece } from './sprites';

	let {
		view,
		interactive = false,
		highlight = new Set<string>(),
		onplay = () => {}
	}: {
		view: unknown;
		interactive?: boolean;
		/** Target sorotan tutorial: `aksi:<san>` atau `petak:<sq>`, sudah dipisah. */
		highlight?: Set<string>;
		onplay?: (command: string) => void;
	} = $props();

	const v = $derived(isChessView(view) ? view : null);
	/** Sudut pandang: hitam di bawah bila pemain memegang hitam. */
	const flip = $derived(v?.kamu === 1);
	const moves = $derived<ChessMove[]>(interactive && v && v.giliran === v.kamu ? v.langkah : []);
	const sources = $derived(new Set(moves.map((m) => m.dari)));

	let selected = $state<string | null>(null);
	let promo = $state<{ dari: string; ke: string } | null>(null);

	// Pilihan dibatalkan tiap kali posisi berubah.
	$effect(() => {
		void v?.fen;
		selected = null;
		promo = null;
	});

	const targetsOf = (from: string) => new Set(moves.filter((m) => m.dari === from).map((m) => m.ke));
	const targets = $derived(selected ? targetsOf(selected) : new Set<string>());
	const clickable = $derived(new Set([...sources, ...targets]));

	/** Sorotan tutorial: petak langsung, atau asal+tujuan langkah SAN. */
	const lit = $derived.by(() => {
		const out = new Set<string>();
		for (const h of highlight) {
			const m = v?.langkah.find((x) => x.san === h || x.san.replace(/[+#]$/, '') === h);
			if (m) {
				out.add(m.dari);
				out.add(m.ke);
			} else if (/^[a-h][1-8]$/.test(h)) out.add(h);
		}
		return out;
	});

	function attempt(from: string, to: string) {
		const found = moves.filter((m) => m.dari === from && m.ke === to);
		if (found.length === 1) {
			selected = null;
			onplay(found[0].san);
		} else if (found.length > 1) {
			promo = { dari: from, ke: to };
		}
	}

	function pick(sq: string) {
		if (promo) return;
		if (selected && targets.has(sq)) attempt(selected, sq);
		else if (sources.has(sq)) selected = selected === sq ? null : sq;
		else selected = null;
	}

	function promote(p: string) {
		if (!promo) return;
		const m = moves.find((x) => x.dari === promo!.dari && x.ke === promo!.ke && x.promosi === p);
		promo = null;
		selected = null;
		if (m) onplay(m.san);
	}

	function spriteOf(ch: string): { pixels: readonly string[]; white: boolean } | null {
		if (ch === '.') return null;
		const white = ch === ch.toUpperCase();
		const base = CHESS[ch.toLowerCase() as ChessPiece];
		return { pixels: white ? base : outline(base), white };
	}

	const PIECE_KEY: Record<string, Key> = {
		k: 'catur.piece.k',
		q: 'catur.piece.q',
		r: 'catur.piece.r',
		b: 'catur.piece.b',
		n: 'catur.piece.n',
		p: 'catur.piece.p'
	};

	function label(sq: string): string {
		if (!v) return sq;
		const ch = pieceAt(v, sq);
		const parts = [sq];
		if (ch === '.') parts.push(t('catur.cell.empty'));
		else
			parts.push(
				`${t(ch === ch.toUpperCase() ? 'catur.white' : 'catur.black')} ${t(PIECE_KEY[ch.toLowerCase()])}`
			);
		if (targets.has(sq)) parts.push(t('catur.cell.target'));
		if (selected === sq) parts.push(t('catur.cell.selected'));
		if (v.raja_skak === sq) parts.push(t('catur.check'));
		return parts.join(', ');
	}

	const cols = $derived(flip ? [...FILES].reverse() : [...FILES]);
	const rows = $derived(flip ? ['1', '2', '3', '4', '5', '6', '7', '8'] : ['8', '7', '6', '5', '4', '3', '2', '1']);
	const start = $derived(selected ?? [...lit][0] ?? moves[0]?.dari ?? v?.terakhir?.ke ?? 'e2');
</script>

{#if v}
	<div class="chess">
		<GridBoard
			colLabels={cols}
			rowLabels={rows}
			squareOf={(r, c) => squareAt(r, c, flip)}
			{label}
			ariaLabel={t('catur.board')}
			legal={clickable}
			{targets}
			highlight={lit}
			last={v.terakhir ? [v.terakhir.dari, v.terakhir.ke] : null}
			{selected}
			alert={v.raja_skak}
			{start}
			onplay={pick}
			draggable={(sq) => sources.has(sq) && !promo}
			ondrop={(from, to) => {
				selected = from;
				if (targetsOf(from).has(to)) attempt(from, to);
				else selected = null;
			}}
		>
			{#snippet piece(sq)}
				{@const s = spriteOf(pieceAt(v, sq))}
				{#if s}<span class:black={!s.white}><PixelSprite pixels={s.pixels} size={2} /></span>{/if}
			{/snippet}
		</GridBoard>

		{#if promo}
			<div class="promo" role="group" aria-label={t('catur.promotion')}>
				<span>{t('catur.promotion')}</span>
				{#each PROMOTIONS as p (p)}
					<NavButton label={t(PIECE_KEY[p])} onclick={() => promote(p)}
						><span class="promo-piece"><PixelSprite pixels={v.kamu === 0 ? CHESS[p] : outline(CHESS[p])} size={2} /></span
						></NavButton
					>
				{/each}
				<NavButton onclick={() => (promo = null)}>[ {t('action.cancel')} ]</NavButton>
			</div>
		{/if}
	</div>
{/if}

<style>
	.chess {
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 2);
	}
	.promo {
		display: flex;
		align-items: center;
		gap: 1ch;
	}
	.promo-piece {
		display: inline-grid;
		place-items: center;
		width: 32px;
		height: 32px;
		vertical-align: middle;
	}
</style>
