<!--
  Papan taruhan casino dadu dan roda M6a (SPEC §4, §6.4): Roulette
  Eropa/Amerika, Sic Bo, Chuck-a-luck, Big Six, Fan-Tan, dan Craps,
  digerakkan model per game (`papan.ts`). Setiap tempat taruhan adalah sel
  berukuran tetap (label kiri, jumlah kanan); dadu berupa sprite piksel.
  Hover, fokus, pilihan, dan sorotan hasil hanya lapisan absolut, jadi
  tidak ada yang bergeser.

  Taruhan: tombol chip menyusun jumlah, klik sel memasang `bet <tempat> <n>`.
  Roulette: mode LURUS memasang straight/luar langsung; mode GABUNG memilih
  2–6 angka lalu [ + PASANG ] memasang split/street/trio/corner/top
  line/line yang mencakup tepat angka itu. Craps: mode TARIK mengubah klik
  sel menjadi `take <tempat>`. Yang dikirim selalu perintah teks.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { diePixels } from './dice';
	import { chipSteps, isMejaView, type MejaView } from './meja';
	import { RED, ROULETTE_ROWS, combined, crapsComeOut, papanModel, rouletteCoverage } from './papan';

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

	const v = $derived(isMejaView(view) ? (view as MejaView & Record<string, any>) : null);
	const model = $derived(v ? papanModel(game, v) : null);
	const over = $derived(!!v && v.fase === 'selesai');
	const betting = $derived(!!v && (v.fase === 'taruhan' || nextRound));
	const spotUsage = 'bet <tempat> <jumlah>';
	const can = (usage: string) => interactive && (nextRound ? usage.startsWith('bet') : actions.includes(usage));

	/** Taruhan yang tampil di sel: yang sedang disusun, atau ronde terakhir. */
	const shown = $derived.by((): Record<string, number> => {
		if (!v) return {};
		if (model?.craps) return nextRound ? {} : (v.taruhan ?? {});
		if (nextRound || over) return v.taruhan_terakhir ?? {};
		return v.taruhan ?? {};
	});
	const paid = $derived((v?.bayar ?? {}) as Record<string, number>);

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 10);
	const minBet = $derived(v?.min_taruhan ?? 10);
	const tableMax = $derived(v?.maks_taruhan ?? 2000);
	$effect(() => {
		if (amount === 0 && v) amount = v.min_taruhan;
	});
	$effect(() => {
		for (const h of highlight) {
			const m = /^bet \S+ (\d+)$/.exec(h);
			if (m) amount = Number(m[1]);
		}
	});
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));

	function add(n: number) {
		amount = Math.min(tableMax, amount + n);
	}

	/** Batas tempat sekarang: [min, sisa ruang, kelipatan]. */
	function limits(spot: string): [number, number, number] | null {
		if (!v || !model) return null;
		if (model.craps) {
			if (nextRound) return crapsComeOut(spot) ? [10, 2000, /^place-[68]$/.test(spot) ? 30 : 10] : null;
			const b = v.batas?.[spot];
			return b ? [b[0], b[1], b[2]] : null;
		}
		const now = nextRound ? 0 : (v.taruhan?.[spot] ?? 0);
		return [minBet, tableMax - now, step];
	}

	function betOk(spot: string): boolean {
		const l = limits(spot);
		if (!l || !can(spotUsage)) return false;
		if (chips !== null && amount > chips) return false;
		return amount >= l[0] && amount <= l[1] && amount % l[2] === 0;
	}

	// Craps: mode tarik.
	let taking = $state(false);
	const takeable = $derived(new Set<string>(nextRound ? [] : (v?.bisa_ditarik ?? [])));
	const canTake = $derived(interactive && !nextRound && takeable.size > 0 && actions.includes('take <tempat>'));
	$effect(() => {
		if (!canTake) taking = false;
	});

	function cellClick(spot: string) {
		if (taking) onplay(`take ${spot}`);
		else onplay(`bet ${spot} ${amount}`);
	}
	const cellOn = (spot: string) => (taking ? canTake && takeable.has(spot) : betOk(spot));

	// Roulette: mode gabung.
	let joining = $state(false);
	let picked = $state<string[]>([]);
	const american = $derived(!!model?.roulette?.american);
	const joined = $derived(combined(picked, american));
	function pocketClick(p: string) {
		if (!joining) return cellClick(`straight-${p}`);
		if (picked.includes(p)) picked = picked.filter((x) => x !== p);
		else if (picked.length < 6) picked = [...picked, p];
	}
	function place() {
		if (!joined) return;
		onplay(`bet ${joined} ${amount}`);
		picked = [];
	}
	const insideBets = $derived(
		Object.entries(shown)
			.filter(([s]) => /^(split|street|trio|corner|line)-/.test(s))
			.map(([s, n]) => `${model?.label(s)} ·${n}`)
			.join('  ')
	);
	/** Kantong yang dicakup taruhan yang disorot tutorial (`bet <tempat> n`). */
	const litPockets = $derived(
		new Set(
			[...highlight]
				.map((h) => /^bet (\S+) \d+$/.exec(h)?.[1] ?? '')
				.filter((s) => /^(straight|split|street|trio|corner|line)-/.test(s))
				.flatMap(rouletteCoverage)
		)
	);
	const litSpot = (spot: string) => highlight.has(`bet ${spot} ${amount}`) || [...highlight].some((h) => h.startsWith(`bet ${spot} `));
	const hit = (pocket: string) => typeof v?.hasil === 'string' && v.hasil === pocket && (over || nextRound);

	const outcome = $derived.by(() => {
		if (!v || !model) return '';
		const entries = Object.entries(paid);
		if (!entries.length) return '';
		const list = entries.map(([s, n]) => `${model.label(s)} ${signed(n)}`).join(' · ');
		const net = entries.reduce((a, [, n]) => a + n, 0);
		return `${list} · ${t(model.craps ? 'pp.roll_net' : 'mj.round_net', { n: signed(net) })}`;
	});
</script>

{#snippet cell(spot: string, label: string, width: number, extra = '')}
	{@const n = shown[spot] ?? 0}
	<span class="cell {extra}" data-spot={spot} style:width={`${width}ch`}>
		<NavButton sorot={litSpot(spot)} disabled={!interactive || !cellOn(spot)} label={`${label} ${n || ''}`.trim()} onclick={() => cellClick(spot)}>
			<span class="lbl">{label}</span><span class="amt">{n || ''}</span>
		</NavButton>
		{#if (over || nextRound) && (paid[spot] ?? 0) > 0}<span class="layer win" aria-hidden="true"></span>{/if}
	</span>
{/snippet}

{#if v && model}
	<div class="table papan" data-fase={v.fase} data-game={game}>
		<div class="result" data-row="hasil">
			{#if model.dieSlots}<div class="dice">
				{#each Array.from({ length: model.dieSlots }) as _, i (i)}
					<span class="die" data-die={model.dice?.[i] ?? ''}>
						{#if model.dice?.[i]}<PixelSprite pixels={diePixels(model.dice[i])} size={2} />{/if}
					</span>
				{/each}
			</div>{/if}
			<p class="face">{model.face}</p>
		</div>

		{#if model.roulette}
			<div class="wheel" class:american data-row="angka" role="group" aria-label={t('pp.r.layout')}>
				{#each american ? ['0', '00'] : ['0'] as z, i (z)}
					<span
						class="pocket zero"
						data-pocket={z}
						style:grid-row={american ? (i === 0 ? '1 / 4' : '4 / 7') : '1 / 7'}
						style:grid-column="1"
					>
						<NavButton
							sorot={litPockets.has(z)}
							pressed={joining ? picked.includes(z) : undefined}
							disabled={!interactive || (joining ? false : !betOk(`straight-${z}`))}
							label={`${z} ${shown[`straight-${z}`] ?? ''}`.trim()}
							onclick={() => pocketClick(z)}
						>
							<span class="num">{z}</span><span class="amt">{shown[`straight-${z}`] ?? ''}</span>
						</NavButton>
						{#if picked.includes(z)}<span class="layer chosen" aria-hidden="true"></span>{/if}
						{#if hit(z)}<span class="layer win" aria-hidden="true"></span>{/if}
					</span>
				{/each}
				{#each ROULETTE_ROWS as row, r (r)}
					{#each row as n, c (n)}
						{@const p = String(n)}
						<span
							class="pocket"
							class:red={RED.includes(n)}
							data-pocket={p}
							style:grid-row={`${2 * r + 1} / span 2`}
							style:grid-column={`${c + 2}`}
						>
							<NavButton
								sorot={litPockets.has(p)}
								pressed={joining ? picked.includes(p) : undefined}
								disabled={!interactive || (joining ? false : !betOk(`straight-${p}`))}
								label={`${p} ${shown[`straight-${p}`] ?? ''}`.trim()}
								onclick={() => pocketClick(p)}
							>
								<span class="num">{p}</span><span class="amt">{shown[`straight-${p}`] ?? ''}</span>
							</NavButton>
							{#if picked.includes(p)}<span class="layer chosen" aria-hidden="true"></span>{/if}
							{#if hit(p)}<span class="layer win" aria-hidden="true"></span>{/if}
						</span>
					{/each}
					<span class="pocket col" style:grid-row={`${2 * r + 1} / span 2`} style:grid-column="14" data-spot={`column-${3 - r}`}>
						<NavButton
							sorot={litSpot(`column-${3 - r}`)}
							disabled={!interactive || joining || !betOk(`column-${3 - r}`)}
							label={`${model.label(`column-${3 - r}`)} ${shown[`column-${3 - r}`] ?? ''}`.trim()}
							onclick={() => cellClick(`column-${3 - r}`)}
						>
							<span class="num">2:1</span><span class="amt">{shown[`column-${3 - r}`] ?? ''}</span>
						</NavButton>
						{#if (over || nextRound) && (paid[`column-${3 - r}`] ?? 0) > 0}<span class="layer win" aria-hidden="true"></span>{/if}
					</span>
				{/each}
			</div>
		{/if}

		{#each model.groups as g (g.id)}
			<div class="group" data-group={g.id}>
				{#if g.label}<span class="glabel dim">{g.label}</span>{/if}
				<div class="cells" style:grid-template-columns={`repeat(${g.cols}, ${g.width}ch)`}>
					{#each g.cells as c (c.spot)}
						{@render cell(c.spot, c.label, g.width)}
					{/each}
				</div>
			</div>
		{/each}

		{#if model.roulette}
			<p class="line" data-inside>{insideBets ? t('pp.r.inside', { list: insideBets }) : ''}</p>
		{/if}

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: nextRound ? 0 : v.taruhan_meja })}</span>
			{#if model.craps}<span>{t('pp.cr.net', { n: signed(v.bersih) })}</span>{/if}
		</div>
		<p class="outcome">{outcome}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if betting}
					<div class="bet-row">
						<span class="amount" aria-live="polite">{t('bj.bet_amount', { n: amount })}</span>
						{#each chipSteps(step) as n (n)}
							<NavButton disabled={amount + n > tableMax} onclick={() => add(n)}>[ +{n} ]</NavButton>
						{/each}
						<NavButton onclick={() => (amount = minBet)}>[ {t('bj.clear')} ]</NavButton>
					</div>
				{/if}
				{#if model.roulette && betting}
					<div class="bet-row">
						<NavButton pressed={!joining} onclick={() => ((joining = false), (picked = []))}>[ {t('pp.r.mode_straight')} ]</NavButton>
						<NavButton pressed={joining} onclick={() => (joining = true)}>[ {t('pp.r.mode_join')} ]</NavButton>
						<NavButton disabled={!joining || !joined || !betOk(joined)} onclick={place}
							>[ + {joined ? model.label(joined).toUpperCase() : t('pp.r.place')} ]</NavButton
						>
					</div>
				{/if}
				<div class="bet-row">
					{#if !nextRound}
						<NavButton sorot={highlight.has(model.verb)} disabled={!can(model.verb)} onclick={() => onplay(model.verb)}
							>[ {t(`pp.verb.${model.verb}` as Key)} ]</NavButton
						>
						{#if model.craps}
							<NavButton pressed={taking} disabled={!canTake} onclick={() => (taking = !taking)}>[ {t('pp.cr.take')} ]</NavButton>
						{:else}
							<NavButton disabled={!can('clear')} onclick={() => onplay('clear')}>[ {t('mj.clear_bets')} ]</NavButton>
						{/if}
						<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
							>[ {t('bj.leave')} ]</NavButton
						>
					{/if}
				</div>
				<p class="hint dim">
					{#if taking}{t('pp.cr.take_hint')}{:else if joining}{t('pp.r.join_hint', { n: picked.length })}{:else}{t(
							'mj.limit',
							{ min: minBet, max: tableMax, step }
						)}{model.craps ? ` ${t('pp.cr.limit_hint')}` : ''}{/if}
				</p>
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
	.result {
		display: flex;
		align-items: center;
		gap: 2ch;
		height: 34px;
	}
	.dice {
		display: flex;
		gap: 8px;
		height: 30px;
	}
	.die {
		width: 30px;
		height: 30px;
	}
	.face {
		margin: 0;
		width: 40ch;
		white-space: nowrap;
	}
	.wheel {
		display: grid;
		grid-template-columns: repeat(14, 5ch);
		grid-template-rows: repeat(6, calc(var(--cell-h) * 0.75));
		width: max-content;
	}
	.group {
		display: flex;
		gap: 1ch;
		align-items: flex-start;
	}
	.glabel {
		width: 11ch;
		flex: none;
		line-height: calc(var(--cell-h) + 2px);
		white-space: nowrap;
	}
	.cells {
		display: grid;
	}
	.cell,
	.pocket {
		position: relative;
		display: block;
		height: calc(var(--cell-h) + 2px);
	}
	.pocket {
		height: auto;
	}
	.cell :global(.tbtn),
	.pocket :global(.tbtn) {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		height: 100%;
		padding: 0 0.25ch;
		border: 1px solid var(--dim, currentColor);
		margin: 0 -1px -1px 0;
		box-sizing: border-box;
		white-space: nowrap;
		overflow: hidden;
	}
	.pocket :global(.tbtn) {
		flex-direction: column;
		justify-content: center;
		padding: 0;
		line-height: 1.1;
	}
	.pocket.red :global(.tbtn) {
		background: var(--fg);
		color: var(--bg);
	}
	.amt {
		min-width: 4ch;
		text-align: right;
	}
	.pocket .amt {
		min-width: 0;
		text-align: center;
		font-size: 0.8em;
		height: 1.1em;
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.chosen {
		outline: 2px dashed var(--fg);
		outline-offset: -4px;
	}
	.win {
		outline: 3px double var(--fg);
		outline-offset: -1px;
	}
	.line {
		margin: 0;
		height: var(--cell-h);
		width: 64ch;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.outcome {
		margin: 0;
		width: 64ch;
		height: calc(var(--cell-h) * 2);
		overflow: hidden;
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
		align-items: baseline;
	}
	.amount {
		width: 16ch;
	}
	.hint {
		margin: 0;
		max-width: 64ch;
		min-height: calc(var(--cell-h) * 2);
	}
</style>
