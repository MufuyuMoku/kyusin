<!--
  Keno M6b (SPEC §4, §6.5): papan 80 angka (8 × 10) dan tabel bayar untuk
  jumlah angka yang dipilih. Angka pilihan berwarna terbalik; angka yang
  ditarik dan yang kena diberi lapisan garis (tidak menggeser apa pun).
  Klik angka = `mark`/`unmark`; [ TARIK · n ] = `bet n`.
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

	const v = $derived(isMejaView(view) ? (view as V) : null);
	const over = $derived(!!v && v.fase === 'selesai');
	const picks = $derived(new Set<number>(nextRound ? [] : (v?.pilihan ?? [])));
	const shownPicks = $derived(new Set<number>(v?.pilihan ?? []));
	const drawn = $derived(new Set<number>(v?.tarikan ?? []));
	const can = (usage: string) => interactive && (nextRound ? usage === 'mark <angka>' : actions.includes(usage));
	const lit = (n: number) => highlight.has(`mark ${n}`) || highlight.has(`unmark ${n}`);

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

	function toggle(n: number) {
		if (picks.has(n)) onplay(`unmark ${n}`);
		else onplay(`mark ${n}`);
	}
	const full = $derived(picks.size >= 10);
	const rows = Array.from({ length: 8 }, (_, r) => Array.from({ length: 10 }, (_, c) => r * 10 + c + 1));
	/** Tabel bayar: dari view (jumlah pilihan sekarang). */
	const table = $derived((v?.tabel ?? []) as [number, number][]);
	const hits = $derived((v?.kena ?? []) as number[]);
	const outcome = $derived.by(() => {
		if (!v || !over || v.bayar === null || v.bayar === undefined) return '';
		const pay = table.find(([h]) => h === hits.length)?.[1] ?? 0;
		return [
			t('kn.hits', { n: hits.length, of: shownPicks.size }),
			pay ? times(pay, 10) : t('kn.no_pay'),
			t('mj.round_net', { n: signed(v.bayar) })
		].join(' · ');
	});
</script>

{#if v}
	<div class="table keno" data-fase={v.fase} data-game={game}>
		<div class="top">
			<div class="board" role="group" aria-label={t('kn.board')}>
				{#each rows as row, r (r)}
					{#each row as n (n)}
						<span
							class="cell"
							class:picked={shownPicks.has(n)}
							data-num={n}
							data-drawn={drawn.has(n) ? '' : undefined}
						>
							<NavButton
								sorot={lit(n)}
								pressed={picks.has(n)}
								disabled={!can(picks.has(n) ? 'unmark <angka>' : 'mark <angka>') || (!picks.has(n) && full)}
								label={String(n)}
								onclick={() => toggle(n)}>{n}</NavButton
							>
							{#if drawn.has(n)}<span class="layer" class:hit={shownPicks.has(n)} class:ball={!shownPicks.has(n)} aria-hidden="true"
								></span>{/if}
						</span>
					{/each}
				{/each}
			</div>
			<div class="pays" data-row="tabel">
				<p class="head dim">{t('kn.pays', { n: shownPicks.size })}</p>
				{#each table as [h, m] (h)}
					<p class="pay" class:won={over && hits.length === h}>
						<span>{t('kn.hit_row', { n: h })}</span><span>{times(m, 10)}</span>
					</p>
				{:else}
					<p class="pay dim">{t('kn.pick_first')}</p>
				{/each}
			</div>
		</div>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('kn.picked', { n: shownPicks.size })}</span>
			<span>{drawn.size ? t('kn.drawn_count', { n: drawn.size }) : ''}</span>
		</div>
		<p class="outcome">{outcome}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				<ChipBet bind:amount min={minBet} max={maxBet} {step} />
				<div class="bet-row">
					<NavButton
						sorot={litAmount(highlight) !== null}
						disabled={nextRound || !can('bet <jumlah>') || amount < minBet || amount > maxBet}
						onclick={() => onplay(`bet ${amount}`)}>[ {t('kn.draw', { n: num(amount) })} ]</NavButton
					>
					<NavButton disabled={nextRound || !can('clear')} onclick={() => onplay('clear')}>[ {t('kn.clear')} ]</NavButton>
					{#if !nextRound}
						<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
							>[ {t('bj.leave')} ]</NavButton
						>
					{/if}
				</div>
				<p class="hint dim">{t('mj.limit', { min: minBet, max: maxBet, step })}</p>
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
		gap: 3ch;
		align-items: flex-start;
	}
	.board {
		display: grid;
		grid-template-columns: repeat(10, 4.5ch);
		grid-auto-rows: calc(var(--cell-h) + 4px);
	}
	.cell {
		position: relative;
	}
	.cell :global(.tbtn) {
		display: block;
		width: 100%;
		height: 100%;
		padding: 0;
		text-align: center;
		border: 1px solid var(--dim, currentColor);
		margin: 0 -1px -1px 0;
		box-sizing: border-box;
	}
	.cell.picked :global(.tbtn) {
		background: var(--fg);
		color: var(--bg);
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.ball {
		outline: 2px dashed var(--fg);
		outline-offset: -4px;
	}
	.hit {
		outline: 3px double var(--bg);
		outline-offset: -4px;
	}
	.pays {
		width: 18ch;
	}
	.pays p {
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
	}
	.pay {
		display: flex;
		justify-content: space-between;
	}
	.pay.won {
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
		width: 64ch;
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
