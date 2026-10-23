<!--
  Hi-Lo M6b (SPEC §4, §6.5): deretan kartu yang sudah dibuka (sprite piksel,
  slot tetap; delapan terakhir), pengali berjalan, dan tombol HIGHER/LOWER
  dengan pengalinya serta CASH OUT dengan nilai yang akan dibayar.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t } from '$lib/i18n.svelte';
	import ChipBet from './ChipBet.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { cardPixels } from './cards';
	import { isMejaView } from './meja';
	import { litAmount, num, signed, times, type V } from './lotere';

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

	const SLOTS = 8;
	const v = $derived(isMejaView(view) ? (view as V) : null);
	const over = $derived(!!v && v.fase === 'selesai');
	const guessing = $derived(!!v && v.fase === 'tebak' && !nextRound);
	const cards = $derived(((v?.kartu ?? []) as string[]).slice(-SLOTS));
	const mult = $derived((v?.pengali ?? 100) as number);
	const can = (usage: string) => interactive && (nextRound ? usage.startsWith('bet') : actions.includes(usage));

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 100);
	const minBet = $derived(v?.min_taruhan ?? 100);
	const maxBet = $derived.by(() => {
		const table = v?.maks_taruhan ?? 2000;
		if (chips === null) return table;
		return Math.min(table, chips - (chips % step));
	});
	$effect(() => {
		if (amount === 0 && v) amount = v.min_taruhan;
	});
	$effect(() => {
		const a = litAmount(highlight);
		if (a !== null) amount = a;
	});
	const cashValue = $derived(Math.floor(((v?.taruhan ?? 0) * mult) / 100));
	const outcome = $derived.by(() => {
		if (!v || !over || typeof v.bayar !== 'number') return '';
		return `${t(`hl.end.${v.alasan ?? 'kalah'}` as 'hl.end.kalah')} · ${t('mj.round_net', { n: signed(v.bayar) })}`;
	});
</script>

{#if v}
	<div class="table hilo" data-fase={v.fase} data-game={game}>
		<div class="hand" data-row="kartu" style:width={`${SLOTS * 52}px`}>
			{#each cards as c, i (i)}
				<span class="card" class:old={i < cards.length - 1} data-card={c} style:left={`${i * 52}px`}>
					<PixelSprite pixels={cardPixels(c)} size={2} />
				</span>
			{/each}
		</div>
		<p class="line" data-row="pengali">
			{t('hl.mult', { m: times(mult, 100) })}{v.tebakan ? ` · ${t('hl.streak', { n: v.tebakan })}` : ''}
		</p>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: nextRound ? 0 : v.taruhan_meja })}</span>
		</div>
		<p class="outcome">{outcome}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if guessing}
					<div class="bet-row">
						<NavButton sorot={highlight.has('higher')} disabled={!can('higher')} onclick={() => onplay('higher')}
							>[ {t('hl.higher', { m: times(v.pengali_tinggi ?? 0, 100) })} ]</NavButton
						>
						<NavButton sorot={highlight.has('lower')} disabled={!can('lower')} onclick={() => onplay('lower')}
							>[ {t('hl.lower', { m: times(v.pengali_rendah ?? 0, 100) })} ]</NavButton
						>
						<NavButton sorot={highlight.has('cashout')} disabled={!can('cashout')} onclick={() => onplay('cashout')}
							>[ {t('hl.cashout', { n: num(cashValue) })} ]</NavButton
						>
					</div>
					<p class="hint dim">{t('hl.hint')}</p>
				{:else}
					<ChipBet bind:amount min={minBet} max={maxBet} {step} />
					<div class="bet-row">
						<NavButton
							sorot={litAmount(highlight) !== null}
							disabled={!can('bet <jumlah>') || amount < minBet || amount > maxBet}
							onclick={() => onplay(`bet ${amount}`)}>[ {t('hl.deal', { n: num(amount) })} ]</NavButton
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
	.hand {
		position: relative;
		height: 60px;
	}
	.card {
		position: absolute;
		top: 0;
		width: 44px;
		height: 60px;
	}
	.card.old {
		opacity: 0.55;
	}
	.line,
	.outcome {
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.row.info {
		display: flex;
		gap: 3ch;
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
	.bet-row :global(.tbtn) {
		min-width: 18ch;
	}
	.hint {
		margin: 0;
		max-width: 64ch;
	}
</style>
