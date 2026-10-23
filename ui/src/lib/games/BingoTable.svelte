<!--
  Bingo M6b (SPEC §4, §6.5): empat kartu 5×5 dari seed ronde (yang tidak
  dimainkan diredupkan), papan 75 bola (B I N G O × 15), dan pola tiap
  kartu. Petak yang kena berwarna terbalik; ukuran petak tetap.
  [ n KARTU ] = `cards n`; [ TARIK · n ] = `bet n` (per kartu).
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import ChipBet from './ChipBet.svelte';
	import { isMejaView } from './meja';
	import { BINGO_COLUMNS, litAmount, num, signed, type V } from './lotere';

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
	const cards = $derived((v?.kartu ?? []) as number[][]);
	const count = $derived((v?.jumlah_kartu ?? 1) as number);
	const balls = $derived(new Set<number>(v?.bola ?? []));
	const can = (usage: string) => interactive && (nextRound ? usage === 'cards <jumlah_kartu>' || usage.startsWith('bet') : actions.includes(usage));

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 10);
	const minBet = $derived(v?.min_taruhan ?? 10);
	const maxBet = $derived.by(() => {
		const table = v?.maks_taruhan ?? 2000;
		if (chips === null) return table;
		const room = Math.floor(chips / count);
		return Math.min(table, room - (room % step));
	});
	$effect(() => {
		if (amount === 0 && v) amount = v.min_taruhan;
	});
	$effect(() => {
		const a = litAmount(highlight);
		if (a !== null) amount = a;
	});

	const pattern = (i: number) => {
		const p = (v?.pola ?? [])[i];
		return typeof p === 'string' ? t(`bg.pattern.${p}` as Key) : over && i < count ? t('bg.none') : '';
	};
	const cardNet = (i: number) => {
		const n = (v?.bayar_kartu ?? [])[i];
		return typeof n === 'number' ? signed(n) : '';
	};
	const columns = BINGO_COLUMNS.map((c, i) => ({ c, nums: Array.from({ length: 15 }, (_, k) => i * 15 + k + 1) }));
</script>

{#if v}
	<div class="table bingo" data-fase={v.fase} data-game={game}>
		<div class="cards">
			{#each cards as card, i (i)}
				<div class="card" class:off={i >= count} data-card-index={i}>
					<div class="grid">
						{#each BINGO_COLUMNS as c (c)}<span class="head dim">{c}</span>{/each}
						{#each card as n, k (k)}
							<span class="sq" class:free={n === 0} class:hit={n !== 0 && balls.has(n)} data-sq={n}>{n === 0 ? '*' : n}</span>
						{/each}
					</div>
					<p class="line" data-pattern={i}>{pattern(i)}{cardNet(i) ? ` ${cardNet(i)}` : ''}</p>
				</div>
			{/each}
		</div>

		<div class="balls" role="group" aria-label={t('bg.balls')} data-row="bola">
			{#each columns as col (col.c)}
				<span class="head dim">{col.c}</span>
				{#each col.nums as n (n)}<span class="ball" class:on={balls.has(n)} data-ball={n}>{n}</span>{/each}
			{/each}
		</div>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bg.cost', { cards: count, n: num(amount * count) })}</span>
			<span>{balls.size ? t('bg.drawn_count', { n: balls.size }) : ''}</span>
		</div>
		<p class="outcome">{over && typeof v.bayar === 'number' ? t('mj.round_net', { n: signed(v.bayar) }) : ''}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				<div class="bet-row">
					{#each [1, 2, 3, 4] as n (n)}
						<NavButton
							sorot={highlight.has(`cards ${n}`)}
							pressed={count === n}
							disabled={!can('cards <jumlah_kartu>') || n > cards.length}
							onclick={() => onplay(`cards ${n}`)}>[ {t('bg.cards', { n })} ]</NavButton
						>
					{/each}
				</div>
				<ChipBet bind:amount min={minBet} max={maxBet} {step} />
				<div class="bet-row">
					<NavButton
						sorot={litAmount(highlight) !== null}
						disabled={!can('bet <jumlah>') || amount < minBet || amount > maxBet}
						onclick={() => onplay(`bet ${amount}`)}>[ {t('kn.draw', { n: num(amount) })} ]</NavButton
					>
					{#if !nextRound}
						<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
							>[ {t('bj.leave')} ]</NavButton
						>
					{/if}
				</div>
				<p class="hint dim">{t('bg.limit', { min: minBet, max: maxBet, step })}</p>
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
	.cards {
		display: flex;
		gap: 2ch;
	}
	.card.off {
		opacity: 0.4;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(5, 3ch);
		grid-auto-rows: var(--cell-h);
		column-gap: 0.25ch;
		text-align: center;
	}
	.sq.hit {
		background: var(--fg);
		color: var(--bg);
	}
	.line {
		margin: 0;
		height: var(--cell-h);
		width: 16ch;
		white-space: nowrap;
	}
	.balls {
		display: grid;
		grid-template-columns: 2ch repeat(15, 3ch);
		grid-auto-rows: calc(var(--cell-h) * 0.85);
		font-size: 0.85em;
		text-align: center;
	}
	.ball {
		color: var(--dim, currentColor);
	}
	.ball.on {
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
