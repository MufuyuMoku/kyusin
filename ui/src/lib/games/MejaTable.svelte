<!--
  Meja casino M5a (SPEC §4, §6.3): satu komponen untuk sepuluh game melawan
  bandar, digerakkan model per game (`meja.ts`). Kartu berupa sprite piksel
  SVG; setiap baris punya slot kartu tetap dan kartu diposisikan absolut,
  jadi kartu baru, hover, fokus, dan pilihan (lapisan garis tepi) tidak
  menggeser elemen mana pun.

  Taruhan: tombol chip menyusun jumlah; game satu taruhan memakai
  [ BAGI · n ] (`bet <n>`), game bertempat memakai [ + TEMPAT ]
  (`bet <tempat> <n>`) lalu [ BAGI ] (`deal`). Pilihan di tengah ronde
  tampil sebagai tombol; Pai Gow memilih dua kartu tangan depan. Game satu
  ronde per sesi (`nextRound`) tetap menampilkan ronde terakhir sambil
  menerima taruhan untuk ronde berikutnya. Yang dikirim selalu perintah teks.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t } from '$lib/i18n.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { cardPixels } from './cards';
	import { dominoPixels } from './domino';
	import { cardLeft, chipSteps, isMejaView, rowSpan, tableModel } from './meja';

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
		/** Ronde sudah selesai dan taruhan berikutnya memulai pertandingan baru. */
		nextRound?: boolean;
	} = $props();

	const v = $derived(isMejaView(view) ? view : null);
	const model = $derived(v ? tableModel(game, v) : null);
	const betting = $derived(!!v && (v.fase === 'taruhan' || nextRound));
	const can = (usage: string) => interactive && (nextRound ? usage.startsWith('bet') : actions.includes(usage));
	const placed = $derived((v?.taruhan ?? {}) as Record<string, number>);

	let amount = $state(0);
	const step = $derived(v?.langkah_taruhan ?? 10);
	const minBet = $derived(v?.min_taruhan ?? 10);
	const multiplier = $derived(v?.pengali?.bet ?? 1);
	const maxBet = $derived.by(() => {
		const table = v?.maks_taruhan ?? 2000;
		if (chips === null) return table;
		const room = Math.floor(chips / multiplier);
		return Math.min(table, room - (room % step));
	});
	$effect(() => {
		if (amount === 0 && v) amount = v.min_taruhan;
	});

	// Tutorial: `bet <tempat> <n>` / `bet <n>` menyetel jumlah taruhan,
	// `set a b` memilih kartu.
	// Pilihan berupa indeks: Pai Gow ubin punya ubin kembar (dua 6-6).
	let selected = $state<number[]>([]);
	$effect(() => {
		for (const h of highlight) {
			const m = /^bet (?:\S+ )?(\d+)$/.exec(h);
			if (m) amount = Number(m[1]);
		}
	});
	const litSpot = (spot: string) => [...highlight].some((h) => h.startsWith(`bet ${spot} `));
	const litSingle = $derived([...highlight].some((h) => /^bet \d+$/.test(h)));
	const litCards = $derived(
		new Set([...highlight].filter((h) => h.startsWith('set ')).flatMap((h) => h.split(' ').slice(1)))
	);

	function add(n: number) {
		amount = Math.min(maxBet, amount + n);
	}

	function toggle(i: number) {
		if (selected.includes(i)) selected = selected.filter((c) => c !== i);
		else if (selected.length < (model?.pick ?? 0)) selected = [...selected, i];
	}
	const pickRow = $derived(model?.rows.find((r) => r.selectable));
	const pix = (c: string) => (model?.tiles ? dominoPixels(c) : cardPixels(c));

	function spotRoom(spot: string): number {
		return (v?.maks_taruhan ?? 2000) - (nextRound ? 0 : (placed[spot] ?? 0));
	}

	const spotUsage = 'bet <tempat> <jumlah>';
	const amountOk = $derived(amount >= minBet && amount <= maxBet);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
	const shoe = $derived(
		v && v.sisa !== null && v.kartu_terpakai !== null
			? t('mj.shoe', { left: v.sisa, used: v.kartu_terpakai, cut: v.potong ?? 0 })
			: ''
	);
</script>

{#if v && model}
	<div class="table" data-fase={v.fase} data-game={game}>
		{#each model.rows as row (row.id)}
			<div class="row">
				<span class="label dim">{row.label}</span>
				<div class="hand" data-row={row.id} style:width={`calc(${rowSpan(row)}px + var(--card-w) + 1ch)`}>
					<div class="cards">
						{#each row.cards as c, i (i)}
							{#if row.selectable && interactive}
								<span class="card pick" data-card={c} style:left={`${cardLeft(row, i)}px`}>
									<NavButton
										pressed={selected.includes(i)}
										sorot={litCards.has(c)}
										label={t('mj.pick_card', { card: c })}
										onclick={() => toggle(i)}
									>
										<PixelSprite pixels={pix(c)} size={2} />
									</NavButton>
									{#if selected.includes(i)}<span class="layer chosen" aria-hidden="true"></span>{/if}
								</span>
							{:else}
								<span class="card" data-card={c} style:left={`${cardLeft(row, i)}px`}>
									<PixelSprite pixels={pix(c)} size={2} />
								</span>
							{/if}
						{/each}
					</div>
					<p class="line">{row.line ?? ''}</p>
				</div>
			</div>
		{/each}

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: v.taruhan_meja })}</span>
			{#if shoe}<span>{shoe}</span>{/if}
		</div>
		<p class="line outcome">{model.outcome}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if betting}
					<div class="bet-row">
						<span class="amount" aria-live="polite">{t('bj.bet_amount', { n: amount })}</span>
						{#each chipSteps(step) as n (n)}
							<NavButton disabled={amount + n > maxBet} onclick={() => add(n)}>[ +{n} ]</NavButton>
						{/each}
						<NavButton onclick={() => (amount = minBet)}>[ {t('bj.clear')} ]</NavButton>
					</div>
					{#if model.spots}
						<div class="bet-row">
							{#each model.spots as s (s.spot)}
								<NavButton
									sorot={litSpot(s.spot)}
									disabled={!can(spotUsage) || !amountOk || amount > spotRoom(s.spot)}
									onclick={() => onplay(`bet ${s.spot} ${amount}`)}>[ + {s.label.toUpperCase()} ]</NavButton
								>
							{/each}
						</div>
						<p class="line spots" data-spots>
							{model.spots.map((s) => `${s.label}: ${nextRound ? 0 : (placed[s.spot] ?? 0)}`).join(' · ')}
						</p>
						<div class="bet-row">
							<NavButton sorot={highlight.has('deal')} disabled={!can('deal')} onclick={() => onplay('deal')}
								>[ {t('mj.deal')} ]</NavButton
							>
							<NavButton disabled={!can('clear')} onclick={() => onplay('clear')}>[ {t('mj.clear_bets')} ]</NavButton>
							{#if !nextRound}
								<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
									>[ {t('bj.leave')} ]</NavButton
								>
							{/if}
						</div>
					{:else}
						<div class="bet-row">
							<NavButton
								sorot={litSingle}
								disabled={!can('bet <jumlah>') || !amountOk}
								onclick={() => onplay(`bet ${amount}`)}>[ {t('bj.deal', { n: amount })} ]</NavButton
							>
							{#if !nextRound}
								<NavButton sorot={highlight.has('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
									>[ {t('bj.leave')} ]</NavButton
								>
							{/if}
						</div>
					{/if}
					<p class="hint dim">
						{t('mj.limit', { min: minBet, max: maxBet, step })}{multiplier > 1 ? ` ${t('mj.per_spot', { n: multiplier })}` : ''}
					</p>
				{:else if model.choices.length || model.pick}
					<div class="bet-row">
						{#if model.pick}
							<NavButton
								sorot={[...highlight].some((h) => h.startsWith('set '))}
								disabled={selected.length !== model.pick}
								onclick={() => onplay(`set ${selected.map((i) => pickRow?.cards[i]).join(' ')}`)}>[ {t('mj.cmd.set')} ]</NavButton
							>
						{/if}
						{#each model.choices as c (c.cmd)}
							<NavButton sorot={highlight.has(c.cmd)} disabled={!can(c.cmd)} onclick={() => onplay(c.cmd)}
								>[ {c.label.toUpperCase()}{v.biaya[c.cmd] ? ` · ${v.biaya[c.cmd]}` : ''} ]</NavButton
							>
						{/each}
					</div>
					<p class="hint dim">
						{model.pick
							? t(model.tiles ? 'mj.pgu.pick_hint' : 'mj.pick_hint', { n: selected.length })
							: t('mj.choice_hint')}
					</p>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.table {
		--card-w: 44px;
		--card-h: 60px;
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 2);
	}
	.row {
		display: flex;
		gap: 2ch;
		align-items: flex-start;
	}
	.label {
		width: 10ch;
		flex: none;
	}
	.hand {
		position: relative;
		flex: none;
		padding: 4px;
	}
	.cards {
		position: relative;
		height: var(--card-h);
	}
	.card {
		position: absolute;
		top: 0;
		width: var(--card-w);
		height: var(--card-h);
		background: var(--bg);
		padding: 0;
		border: 0;
		color: inherit;
		font: inherit;
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
		outline-offset: -2px;
	}
	.line {
		margin: 0;
		height: var(--cell-h);
		/* Lebar tetap: teks panjang meluber ke kanan (tidak ada apa pun di
		   sana), bukan terpotong, dan tidak menggeser apa pun. */
		width: 60ch;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.info {
		gap: 3ch;
		height: var(--cell-h);
		white-space: nowrap;
		overflow: hidden;
	}
	.controls {
		display: flex;
		flex-direction: column;
		gap: calc(var(--cell-h) / 2);
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
		min-height: calc(var(--cell-h) * 2);
	}
</style>
