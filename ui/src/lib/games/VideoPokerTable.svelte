<!--
  Video Poker M6b (SPEC §4, §6.5): tabel bayar varian (dua kolom; tangan
  sekarang disorot), lima kartu sprite piksel di slot tetap, dan baris TAHAN
  berukuran tetap di bawah tiap kartu. Memilih kartu yang ditahan hanya
  mengubah lapisan dan teks di slot itu, jadi tidak ada kartu yang bergeser.
  [ DRAW ] mengirim `draw <posisi…>`.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import ChipBet from './ChipBet.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { cardPixels } from './cards';
	import { isMejaView } from './meja';
	import { drawCommand, litAmount, litPositions, num, signed, times, type V } from './lotere';

	let {
		view,
		game = '',
		interactive = false,
		highlight = new Set<string>(),
		onplay = () => {},
		actions = [],
		chips = null,
		nextRound = false
	}: {
		view: unknown;
		game?: string;
		interactive?: boolean;
		highlight?: Set<string>;
		onplay?: (command: string) => void;
		actions?: string[];
		chips?: number | null;
		nextRound?: boolean;
	} = $props();

	const v = $derived(isMejaView(view) ? (view as V) : null);
	const over = $derived(!!v && v.fase === 'selesai');
	const holding = $derived(!!v && v.fase === 'tahan' && !nextRound);
	const cards = $derived((v?.kartu ?? []) as string[]);
	const table = $derived((v?.tabel ?? []) as [string, number][]);
	const can = (usage: string) => interactive && (nextRound ? usage.startsWith('bet') : actions.includes(usage));

	let held = $state<boolean[]>([false, false, false, false, false]);
	const shownHeld = $derived(over ? ((v?.tahan ?? []) as boolean[]) : held);
	const lit = $derived(litPositions(highlight));

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 10);
	const minBet = $derived(v?.min_taruhan ?? 10);
	const maxBet = $derived.by(() => {
		const tableMax = v?.maks_taruhan ?? 2000;
		if (chips === null) return tableMax;
		return Math.min(tableMax, chips - (chips % step));
	});
	$effect(() => {
		if (amount === 0 && v) amount = v.min_taruhan;
	});
	$effect(() => {
		const a = litAmount(highlight);
		if (a !== null) amount = a;
	});

	function toggle(i: number) {
		held = held.map((h, k) => (k === i ? !h : h));
	}
	function draw() {
		const cmd = drawCommand(held);
		held = [false, false, false, false, false];
		onplay(cmd);
	}
	const hand = $derived((v?.tangan ?? null) as string | null);
	const handName = (k: string) => t(`vp.hand.${k}` as Key);
	const outcome = $derived.by(() => {
		if (!v || !over || typeof v.bayar !== 'number') return '';
		return `${hand ? handName(hand) : t('vp.no_pay')} · ${t('mj.round_net', { n: signed(v.bayar) })}`;
	});
	const half = $derived(Math.ceil(table.length / 2));
</script>

{#if v}
	<div class="table videopoker" data-fase={v.fase} data-game={game}>
		<div class="paytable" data-row="tabel">
			{#each [table.slice(0, half), table.slice(half)] as col, c (c)}
				<div class="col">
					{#each col as [k, m] (k)}
						<p class="pay" class:on={hand === k} data-hand={k}><span>{handName(k)}</span><span>{times(m)}</span></p>
					{/each}
				</div>
			{/each}
		</div>

		<div class="hand" data-row="kartu">
			{#each Array.from({ length: 5 }) as _, i (i)}
				<div class="slot">
					{#if cards[i]}
						{#if holding && interactive}
							<span class="card pick" data-card={cards[i]}>
								<NavButton
									pressed={held[i]}
									sorot={lit.has(i)}
									label={t('vp.hold_card', { card: cards[i] })}
									onclick={() => toggle(i)}
								>
									<PixelSprite pixels={cardPixels(cards[i])} size={2} />
								</NavButton>
								{#if held[i]}<span class="layer chosen" aria-hidden="true"></span>{/if}
							</span>
						{:else}
							<span class="card" data-card={cards[i]}><PixelSprite pixels={cardPixels(cards[i])} size={2} /></span>
						{/if}
					{:else}
						<span class="card empty" aria-hidden="true"></span>
					{/if}
					<span class="held">{shownHeld[i] ? t('vp.held') : ''}</span>
				</div>
			{/each}
		</div>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: nextRound ? 0 : v.taruhan_meja })}</span>
			<span>{holding && hand ? t('vp.dealt', { hand: handName(hand) }) : ''}</span>
		</div>
		<p class="outcome">{outcome}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if holding}
					<div class="bet-row">
						<NavButton sorot={[...highlight].some((h) => h.startsWith('draw'))} disabled={!can('draw')} onclick={draw}
							>[ {t('vp.draw')} ]</NavButton
						>
					</div>
					<p class="hint dim">{t('vp.hint')}</p>
				{:else}
					<ChipBet bind:amount min={minBet} max={maxBet} {step} />
					<div class="bet-row">
						<NavButton
							sorot={litAmount(highlight) !== null}
							disabled={!can('bet <jumlah>') || amount < minBet || amount > maxBet}
							onclick={() => onplay(`bet ${amount}`)}>[ {t('bj.deal', { n: num(amount) })} ]</NavButton
						>
						{#if !nextRound}
							<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
								>[ {t('bj.leave')} ]</NavButton
							>
						{/if}
					</div>
					<p class="hint dim">{t('mj.limit', { min: minBet, max: maxBet, step })}</p>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.table {
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 2);
	}
	.paytable {
		display: flex;
		gap: 3ch;
	}
	.col {
		width: 27ch;
	}
	.pay {
		display: flex;
		justify-content: space-between;
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.pay.on {
		background: var(--fg);
		color: var(--bg);
	}
	.hand {
		display: flex;
		gap: 12px;
	}
	.slot {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 56px;
	}
	.card {
		position: relative;
		display: block;
		width: 44px;
		height: 60px;
	}
	.card.empty {
		border: 1px dashed var(--dim, currentColor);
		box-sizing: border-box;
	}
	.pick :global(.tbtn) {
		display: block;
		padding: 0;
		cursor: pointer;
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.chosen {
		outline: 2px solid var(--fg);
		outline-offset: 2px;
	}
	.held {
		width: 100%;
		height: var(--cell-h);
		text-align: center;
		white-space: nowrap;
	}
	.row.info {
		display: flex;
		gap: 3ch;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.outcome {
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.controls {
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 3);
	}
	.bet-row {
		display: flex;
		flex-wrap: wrap;
		gap: 1ch 2ch;
	}
	.hint {
		margin: 0;
		max-width: 64ch;
	}
</style>
