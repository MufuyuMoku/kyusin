<!--
  Kartu gosok M6b (SPEC §4, §6.5): sembilan sel berukuran tetap. Sel yang
  belum digosok berpola arsir (CSS, bukan glyph) dan bisa diklik
  (`scratch n`); sel yang terbuka menunjukkan pengalinya. Tiga sel bernilai
  sama diberi lapisan garis setelah kartu terbuka. Tabel hadiah di kanan.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t } from '$lib/i18n.svelte';
	import ChipBet from './ChipBet.svelte';
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

	/** Tabel hadiah (D-071): pengali dan peluang per 100.000 kartu. */
	const PRIZES: [number, number][] = [
		[1000, 2],
		[100, 50],
		[50, 200],
		[20, 600],
		[10, 1500],
		[5, 3000],
		[2, 10000],
		[1, 16000]
	];

	const v = $derived(isMejaView(view) ? (view as V) : null);
	const over = $derived(!!v && v.fase === 'selesai');
	const scratching = $derived(!!v && v.fase === 'gosok' && !nextRound);
	const cells = $derived(((v?.sel ?? []) as (number | null)[]).length ? (v?.sel as (number | null)[]) : Array(9).fill(null));
	const prize = $derived((v?.hadiah ?? null) as number | null);
	const can = (usage: string) => interactive && (nextRound ? usage.startsWith('bet') : actions.includes(usage));

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 10);
	const minBet = $derived(v?.min_taruhan ?? 10);
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
	const scratchable = $derived(new Set(actions.filter((a) => /^scratch \d$/.test(a))));
	const canScratch = (i: number) =>
		interactive && scratching && cells[i] === null && (actions.includes('scratch <sel>') || scratchable.has(`scratch ${i + 1}`));
</script>

{#if v}
	<div class="table gosok" data-fase={v.fase} data-game={game}>
		<div class="top">
			<div class="card" role="group" aria-label={t('gs.card')} data-row="kartu">
				{#each cells as c, i (i)}
					<span class="cell" class:hidden={c === null} data-cell={i + 1} data-value={c ?? ''}>
						{#if c === null}
							<NavButton
								sorot={highlight.has(`scratch ${i + 1}`)}
								disabled={!canScratch(i)}
								label={t('gs.scratch_cell', { n: i + 1 })}
								onclick={() => onplay(`scratch ${i + 1}`)}><span class="foil" aria-hidden="true"></span></NavButton
							>
						{:else}
							<span class="value">{times(c)}</span>
						{/if}
						{#if over && prize && c === prize}<span class="layer win" aria-hidden="true"></span>{/if}
					</span>
				{/each}
			</div>
			<div class="prizes" data-row="hadiah">
				<p class="head dim">{t('gs.odds')}</p>
				{#each PRIZES as [m, w] (m)}
					<p class="prize" class:won={over && prize === m}><span>{times(m)}</span><span>{num(w)}</span></p>
				{/each}
			</div>
		</div>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: nextRound ? 0 : v.taruhan_meja })}</span>
		</div>
		<p class="outcome">
			{#if over && typeof v.bayar === 'number'}{prize ? t('gs.win', { m: times(prize) }) : t('gs.lose')} · {t('mj.round_net', {
					n: signed(v.bayar)
				})}{/if}
		</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if scratching}
					<div class="bet-row">
						<NavButton sorot={highlight.has('scratch all')} disabled={!can('scratch all')} onclick={() => onplay('scratch all')}
							>[ {t('gs.scratch_all')} ]</NavButton
						>
					</div>
					<p class="hint dim">{t('gs.hint')}</p>
				{:else}
					<ChipBet bind:amount min={minBet} max={maxBet} {step} />
					<div class="bet-row">
						<NavButton
							sorot={litAmount(highlight) !== null}
							disabled={!can('bet <jumlah>') || amount < minBet || amount > maxBet}
							onclick={() => onplay(`bet ${amount}`)}>[ {t('gs.buy', { n: num(amount) })} ]</NavButton
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
		gap: calc(var(--cell-h) / 3);
	}
	.top {
		display: flex;
		gap: 4ch;
		align-items: flex-start;
	}
	.card {
		display: grid;
		grid-template-columns: repeat(3, 10ch);
		grid-auto-rows: 56px;
		gap: 6px;
		padding: 6px;
		border: 1px solid var(--dim, currentColor);
	}
	.cell {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--dim, currentColor);
		box-sizing: border-box;
	}
	.cell :global(.tbtn) {
		position: absolute;
		inset: 0;
		padding: 0;
	}
	.foil {
		position: absolute;
		inset: 3px;
		background: repeating-linear-gradient(45deg, var(--dim, currentColor) 0 2px, transparent 2px 6px);
	}
	.value {
		font-size: 1.4em;
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.win {
		outline: 3px double var(--fg);
		outline-offset: -4px;
	}
	.prizes {
		width: 18ch;
	}
	.prizes p {
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.prize {
		display: flex;
		justify-content: space-between;
	}
	.prize.won {
		background: var(--fg);
		color: var(--bg);
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
