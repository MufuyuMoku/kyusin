<!--
  Meja antar-pemain M5b-1 (SPEC §4, §6.3): Texas Hold'em, Omaha, Teen
  Patti. Setiap kursi satu baris berukuran tetap (label, kartu dengan slot
  tetap diposisikan absolut, keterangan satu baris), lalu kartu meja (poker),
  pot, dan kontrol. Kartu baru, hover, fokus, dan penanda giliran (lapisan
  garis tepi) tidak menggeser apa pun. Yang dikirim selalu perintah teks.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { cardPixels } from './cards';
	import { BOARD_OFFSET, HOLE, OFFSET, currentBet, isTableView, presets } from './meja_pvp';

	let {
		view,
		game = '',
		interactive = false,
		highlight = new Set<string>(),
		onplay = () => {},
		actions = [],
		chips = null
	}: {
		view: unknown;
		game?: string;
		interactive?: boolean;
		highlight?: Set<string>;
		onplay?: (command: string) => void;
		actions?: string[];
		chips?: number | null;
	} = $props();

	const v = $derived(isTableView(view) ? view : null);
	const teen = $derived(game === 'teen-patti');
	const hole = $derived(HOLE[game] ?? 2);
	const can = (usage: string) => interactive && actions.includes(usage);
	const lit = (cmd: string) => highlight.has(cmd);

	// Jumlah bet/raise yang disusun (total di babak ini).
	let amount = $state(0);
	const min = $derived(v?.naik_min ?? 0);
	const max = $derived(v?.naik_maks ?? 0);
	const step = $derived(v?.blind?.[1] ?? 20);
	$effect(() => {
		if (v && (amount < min || amount > max)) amount = min;
	});
	$effect(() => {
		for (const h of highlight) {
			const m = /^(?:bet|raise) (\d+)$/.exec(h);
			if (m) amount = Number(m[1]);
		}
	});
	const verb = $derived(v && currentBet(v) === 0 ? 'bet' : 'raise');
	const litRaise = $derived([...highlight].some((h) => /^(bet|raise) \d+$/.test(h)));

	function seatName(i: number): string {
		return i === v!.kamu ? t('pk.you') : t('pk.seat', { n: i + 1 });
	}

	function marks(i: number): string {
		const m: string[] = [];
		if (v!.dealer === i) m.push('D');
		if (!teen && v!.sb === i && v!.fase === 'main') m.push('SB');
		if (!teen && v!.bb === i && v!.fase === 'main') m.push('BB');
		return m.join(' ');
	}

	function seatLine(i: number): string {
		const k = v!.kursi[i];
		const parts = [t('pk.stack', { n: k.tumpukan })];
		if (!teen && (k.taruhan ?? 0) > 0) parts.push(t('pk.bet', { n: k.taruhan ?? 0 }));
		if (teen && k.taruhan_tangan > 0) parts.push(t('pk.in_pot', { n: k.taruhan_tangan }));
		parts.push(t(`pk.status.${k.status}` as Key));
		if (teen && k.status === 'aktif') parts.push(t(k.terlihat ? 'pk.seen' : 'pk.blind'));
		if (k.tangan) parts.push(t(`mj.hand.${k.tangan}` as Key));
		if (v!.fase !== 'main' && k.menang > 0) parts.push(t('pk.won', { n: k.menang }));
		return parts.join(' · ');
	}

	const lastActions = $derived(
		v ? v.log.slice(-3).map(([s, c]) => `${seatName(s)}: ${c}`).join(' · ') : ''
	);
	const askedSideshow = $derived(!!v?.sideshow && v.sideshow[1] === v.kamu);
	const cost = (c: string) => v?.biaya[c] ?? 0;
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
</script>

{#if v}
	<div class="table" data-fase={v.fase} data-game={game}>
		{#each v.kursi as k, i (i)}
			<div class="row seat" class:me={i === v.kamu} data-seat={i}>
				<span class="label" class:dim={i !== v.kamu}>{seatName(i)}<span class="marks">{marks(i)}</span></span>
				<div class="hand" data-row={`kursi-${i}`} style:width={`calc(${(hole - 1) * OFFSET}px + var(--card-w) + 1ch)`}>
					<div class="cards">
						{#each k.kartu as c, j (j)}
							<span class="card" data-card={c} style:left={`${j * OFFSET}px`}>
								<PixelSprite pixels={cardPixels(c)} size={2} />
							</span>
						{/each}
					</div>
					{#if v.giliran === i}<div class="layer turn" aria-hidden="true"></div>{/if}
				</div>
				<p class="line" class:dim={k.status === 'fold' || k.status === 'pack' || k.status === 'habis' || k.status === 'berdiri'}>
					{seatLine(i)}
				</p>
			</div>
		{/each}

		{#if !teen}
			<div class="row">
				<span class="label dim">{t('pk.board')}</span>
				<div class="hand" data-row="meja" style:width={`calc(${4 * BOARD_OFFSET}px + var(--card-w) + 1ch)`}>
					<div class="cards">
						{#each v.meja_kartu ?? [] as c, j (j)}
							<span class="card" data-card={c} style:left={`${j * BOARD_OFFSET}px`}>
								<PixelSprite pixels={cardPixels(c)} size={2} />
							</span>
						{/each}
					</div>
				</div>
			</div>
		{/if}

		<div class="row info dim">
			<span>{t('pk.pot', { n: v.pot })}</span>
			{#if teen}<span>{t('pk.stake', { n: v.stake ?? 0 })}</span>{:else}<span
					>{t('pk.blinds', { sb: v.blind?.[0] ?? 0, bb: v.blind?.[1] ?? 0 })}</span
				>{/if}
			<span>{t('pk.hand_no', { n: v.tangan_ke })}</span>
			{#if chips !== null}<span>{t('bj.chips', { n: chips })}</span>{/if}
		</div>
		<p class="line log dim">{lastActions}</p>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if v.fase === 'antara'}
					<div class="bet-row">
						<NavButton disabled={!can('next')} onclick={() => onplay('next')}>[ {t('pk.next')} ]</NavButton>
						<NavButton disabled={!can('leave')} onclick={() => onplay('leave')}>[ {t('pk.leave')} ]</NavButton>
					</div>
					<p class="hint dim">{t('pk.between_hint', { n: signed(v.kursi[v.kamu].tumpukan - v.kursi[v.kamu].awal) })}</p>
				{:else if askedSideshow}
					<div class="bet-row">
						<NavButton sorot={lit('accept')} disabled={!can('accept')} onclick={() => onplay('accept')}
							>[ {t('pk.accept')} ]</NavButton
						>
						<NavButton sorot={lit('deny')} disabled={!can('deny')} onclick={() => onplay('deny')}>[ {t('pk.deny')} ]</NavButton>
					</div>
					<p class="hint dim">{t('pk.sideshow_hint')}</p>
				{:else if teen}
					<div class="bet-row">
						{#if !v.kursi[v.kamu].terlihat}
							<NavButton sorot={lit('see')} disabled={!can('see')} onclick={() => onplay('see')}>[ {t('pk.see')} ]</NavButton>
						{/if}
						<NavButton sorot={lit('pack')} disabled={!can('pack')} onclick={() => onplay('pack')}>[ {'pack'.toUpperCase()} ]</NavButton>
						{#each ['chaal', 'raise', 'show', 'sideshow'] as cmd (cmd)}
							<NavButton sorot={lit(cmd)} disabled={!can(cmd)} onclick={() => onplay(cmd)}
								>[ {cmd.toUpperCase()}{cost(cmd) ? ` · ${cost(cmd)}` : ''} ]</NavButton
							>
						{/each}
					</div>
					<p class="hint dim">{t('pk.teen_hint')}</p>
				{:else}
					<div class="bet-row">
						<NavButton sorot={lit('fold')} disabled={!can('fold')} onclick={() => onplay('fold')}>[ {'fold'.toUpperCase()} ]</NavButton>
						{#if (v.panggil ?? 0) === 0}
							<NavButton sorot={lit('check')} disabled={!can('check')} onclick={() => onplay('check')}>[ {'check'.toUpperCase()} ]</NavButton>
						{:else}
							<NavButton sorot={lit('call')} disabled={!can('call')} onclick={() => onplay('call')}
								>[ {'call'.toUpperCase()} · {v.panggil} ]</NavButton
							>
						{/if}
						<NavButton
							sorot={litRaise}
							disabled={!can(`${verb} <jumlah>`) || amount < min || amount > max}
							onclick={() => onplay(`${verb} ${amount}`)}>[ {verb.toUpperCase()} · {amount} ]</NavButton
						>
					</div>
					<div class="bet-row">
						<span class="amount">{t('pk.amount', { n: amount })}</span>
						{#each presets(v) as p (p.key)}
							<NavButton disabled={!max} onclick={() => (amount = p.amount)}>[ {t(`pk.preset.${p.key}` as Key)} ]</NavButton>
						{/each}
						<NavButton disabled={!max || amount - step < min} onclick={() => (amount = Math.max(min, amount - step))}
							>[ −{step} ]</NavButton
						>
						<NavButton disabled={!max || amount + step > max} onclick={() => (amount = Math.min(max, amount + step))}
							>[ +{step} ]</NavButton
						>
					</div>
					<p class="hint dim">
						{max ? t(v.pot_limit ? 'pk.range_pl' : 'pk.range_nl', { min, max }) : t('pk.no_raise')}
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
		gap: calc(var(--cell-h) / 3);
	}
	.row {
		display: flex;
		gap: 2ch;
		align-items: center;
	}
	.label {
		width: 12ch;
		flex: none;
		white-space: nowrap;
	}
	.marks {
		margin-left: 1ch;
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
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.turn {
		outline: 1px solid var(--fg);
		outline-offset: -1px;
	}
	.line {
		margin: 0;
		height: var(--cell-h);
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
		width: 14ch;
	}
	.hint {
		margin: 0;
		min-height: calc(var(--cell-h) * 2);
	}
</style>
