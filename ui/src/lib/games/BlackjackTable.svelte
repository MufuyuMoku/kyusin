<!--
  Meja Blackjack (SPEC §4, §6.3; D-056). Kartu berupa sprite piksel SVG
  (`cards.ts`). Setiap tangan menempati kotak berukuran tetap dan kartunya
  diposisikan absolut, jadi kartu baru, hover, fokus, dan penanda tangan
  aktif (lapisan garis tepi) tidak menggeser elemen mana pun.

  Kontrol: tombol chip menyusun taruhan, [ BAGI ] mengirim `bet <n>`;
  lalu HIT/STAND/DOUBLE/SPLIT/SURRENDER, atau INSURANCE/TOLAK. Tombol
  yang tidak tersedia tetap tampil (redup, dengan alasan) supaya tata
  letak tidak berubah. Yang dikirim ke backend selalu perintah teks.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { CARD_OFFSET, CHIP_STEPS, isBlackjackView, roundNet } from './blackjack';
	import { cardPixels } from './cards';

	let {
		view,
		interactive = false,
		highlight = new Set<string>(),
		onplay = () => {},
		actions = [],
		chips = null
	}: {
		view: unknown;
		interactive?: boolean;
		highlight?: Set<string>;
		onplay?: (command: string) => void;
		/** Bentuk perintah yang sah sekarang (`bet <jumlah>`, `hit`, …). */
		actions?: string[];
		/** Saldo chip profil; `null` di tutorial dan replay. */
		chips?: number | null;
	} = $props();

	const v = $derived(isBlackjackView(view) ? view : null);
	const can = (cmd: string) => interactive && actions.includes(cmd);
	const canBet = $derived(interactive && actions.some((a) => a.startsWith('bet')));

	/** Taruhan yang sedang disusun. */
	let amount = $state(10);
	const maxBet = $derived.by(() => {
		const table = v?.maks_taruhan ?? 2000;
		return chips === null ? table : Math.min(table, Math.floor(chips / 10) * 10);
	});

	// Tutorial menyorot `bet <n>`: taruhan disetel ke jumlah itu.
	$effect(() => {
		for (const h of highlight) {
			const m = /^bet (\d+)$/.exec(h);
			if (m) amount = Number(m[1]);
		}
	});

	function add(n: number) {
		amount = Math.min(maxBet, amount + n);
	}

	const minBet = $derived(v?.min_taruhan ?? 10);
	const betOk = $derived(amount >= minBet && amount <= maxBet);
	const lit = (cmd: string) => highlight.has(cmd);
	const litBet = $derived([...highlight].some((h) => h.startsWith('bet ')));

	const RESULT: Record<string, Key> = {
		menang: 'bj.result.menang',
		kalah: 'bj.result.kalah',
		seri: 'bj.result.seri',
		menyerah: 'bj.result.menyerah'
	};

	function handLine(i: number): string {
		const h = v!.tangan[i];
		const value = h.lunak && h.nilai < 21 ? t('bj.soft', { n: h.nilai }) : String(h.nilai);
		const parts = [value, t('bj.bet', { n: h.taruhan })];
		if (h.nilai > 21) parts.push(t('bj.bust'));
		return parts.join(' · ');
	}

	function resultLine(i: number): string {
		const h = v!.tangan[i];
		if (!h.hasil || h.bayar === null) return '';
		const sign = h.bayar > 0 ? `+${h.bayar}` : String(h.bayar);
		const text = `${t(RESULT[h.hasil])} ${sign}`;
		return h.blackjack ? `${t('bj.blackjack')} · ${text}` : text;
	}

	/** Alasan tombol aksi tidak tersedia. */
	function reason(cmd: string): string {
		if (!v || v.fase !== 'giliran') return t('bj.reason.not_now');
		const h = v.aktif === null ? null : v.tangan[v.aktif];
		switch (cmd) {
			case 'double':
				return t('bj.reason.two_cards');
			case 'split':
				return v.tangan.length >= 4
					? t('bj.reason.max_hands')
					: h && h.kartu.length === 2 && h.kartu[0][0] === h.kartu[1][0]
						? t('bj.reason.not_now')
						: t('bj.reason.pair');
			case 'surrender':
				return t('bj.reason.first_action');
			default:
				return t('bj.reason.not_now');
		}
	}

	const PLAY = ['hit', 'stand', 'double', 'split', 'surrender'] as const;
	const unavailable = $derived(
		PLAY.filter((c) => !can(c))
			.map((c) => `${c.toUpperCase()}: ${reason(c)}`)
			.join(' · ')
	);
	const net = $derived(v ? roundNet(v) : null);
</script>

{#if v}
	<div class="table" data-fase={v.fase}>
		<div class="row dealer">
			<span class="label dim">{t('bj.dealer')}</span>
			<div class="hand" aria-label={t('bj.dealer')}>
				<div class="cards">
					{#each v.bandar as c, i (i)}
						<span class="card" style:left={`${i * CARD_OFFSET}px`} data-card={c}>
							<PixelSprite pixels={cardPixels(c)} size={2} />
						</span>
					{/each}
				</div>
				<p class="line">{v.bandar_nilai !== null ? v.bandar_nilai : ''}</p>
			</div>
		</div>

		<div class="row hands">
			<span class="label dim">{t('bj.you')}</span>
			{#each Array.from({ length: 4 }, (_, i) => i) as i (i)}
				{@const h = v.tangan[i]}
				<div class="hand" class:empty={!h} data-hand={i} aria-label={h ? t('bj.hand', { n: i + 1 }) : undefined}>
					{#if h}
						<div class="cards">
							{#each h.kartu as c, j (j)}
								<span class="card" style:left={`${j * CARD_OFFSET}px`} data-card={c}>
									<PixelSprite pixels={cardPixels(c)} size={2} />
								</span>
							{/each}
						</div>
						<p class="line">{handLine(i)}</p>
						<p class="line outcome">{resultLine(i)}</p>
						{#if v.aktif === i}<div class="layer active" aria-hidden="true"></div>{/if}
					{:else}
						<!-- Kotak kosong berukuran sama: split tidak mengubah ukuran apa pun. -->
						<div class="cards"></div>
						<p class="line"></p>
						<p class="line"></p>
					{/if}
				</div>
			{/each}
		</div>

		<div class="row info dim">
			<span>{chips === null ? '' : t('bj.chips', { n: chips })}</span>
			<span>{t('bj.at_risk', { n: v.taruhan_meja })}</span>
			<span>{t('bj.shoe', { left: v.sisa, used: v.kartu_terpakai, cut: v.potong })}</span>
			<span>{net === null ? '' : t('bj.round_net', { n: net > 0 ? `+${net}` : String(net) })}</span>
		</div>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if v.fase === 'taruhan'}
					<div class="bet-row">
						<span class="amount" aria-live="polite">{t('bj.bet_amount', { n: amount })}</span>
						{#each CHIP_STEPS as n (n)}
							<NavButton disabled={!canBet || amount + n > maxBet} onclick={() => add(n)}>[ +{n} ]</NavButton>
						{/each}
						<NavButton disabled={!canBet} onclick={() => (amount = minBet)}>[ {t('bj.clear')} ]</NavButton>
					</div>
					<div class="bet-row">
						<NavButton
							sorot={litBet}
							disabled={!canBet || !betOk}
							onclick={() => onplay(`bet ${amount}`)}>[ {t('bj.deal', { n: amount })} ]</NavButton
						>
						<NavButton sorot={lit('leave')} disabled={!can('leave')} onclick={() => onplay('leave')}
							>[ {t('bj.leave')} ]</NavButton
						>
					</div>
					<p class="hint dim">{t('bj.reason.limit', { max: maxBet })}</p>
				{:else if v.fase === 'asuransi'}
					<div class="bet-row">
						<NavButton sorot={lit('insure')} disabled={!can('insure')} onclick={() => onplay('insure')}
							>[ {t('bj.insure', { n: v.biaya.insure ?? 0 })} ]</NavButton
						>
						<NavButton sorot={lit('decline')} disabled={!can('decline')} onclick={() => onplay('decline')}
							>[ {t('bj.decline')} ]</NavButton
						>
					</div>
				{:else if v.fase === 'giliran'}
					<div class="bet-row">
						{#each PLAY as cmd (cmd)}
							<NavButton
								sorot={lit(cmd)}
								disabled={!can(cmd)}
								label={can(cmd) ? undefined : `${cmd.toUpperCase()}: ${reason(cmd)}`}
								onclick={() => onplay(cmd)}>[ {cmd.toUpperCase()} ]</NavButton
							>
						{/each}
					</div>
					<!-- Alasan tombol yang tidak tersedia, di satu baris tetap (D-033). -->
					<p class="hint dim">{unavailable}</p>
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
		width: 8ch;
		flex: none;
	}
	.hand {
		position: relative;
		width: calc(var(--card-w) + 6 * 18px + 1ch);
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
	.line {
		margin: 0;
		height: var(--cell-h);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.active {
		outline: 1px solid var(--fg);
		outline-offset: -1px;
	}
	/* Satu baris tetap: angka yang berubah tidak boleh melipat baris dan
	   menggeser kontrol di bawahnya. */
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
