<!--
  Meja Capsa Susun M5b-2 (SPEC §4, §6.3). Kursi lawan satu baris berslot
  tetap: 13 kartu dalam tiga kelompok (depan 3, tengah 5, belakang 5),
  tertutup selama menyusun dan terbuka setelahnya. Kursimu dua baris:
  tangan (13 slot tetap; kartu yang sudah dipindah meninggalkan slot
  kosong) dan susunan (tiga kelompok). Pilih kartu di tangan lalu baris
  tujuannya; pilih kartu di susunan untuk mengembalikannya. Kartu, sorotan,
  dan pilihan tidak menggeser apa pun. Yang dikirim selalu perintah teks
  (`arrange …` atau `auto`); sah atau tidaknya diputuskan mesin.
-->
<script lang="ts">
	import NavButton from '$lib/components/NavButton.svelte';
	import { t, type Key } from '$lib/i18n.svelte';
	import PixelSprite from './PixelSprite.svelte';
	import { cardPixels } from './cards';
	import { PEEK, PICK, ROW_SIZES, arrangeCommand, isCapsaView, rowsWidth, slotLeft, splitRows } from './capsa';

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

	const v = $derived(isCapsaView(view) ? view : null);
	const me = $derived(v ? v.kursi[v.kamu] : null);
	const arranging = $derived(!!v && v.fase === 'susun' && !!v.saran);
	const can = (usage: string) => interactive && actions.some((a) => a === usage || a.startsWith(`${usage} `));
	const lit = (cmd: string) => highlight.has(cmd);

	// Susunan yang sedang disusun; diulang setiap tangan baru.
	let rows = $state<string[][]>([[], [], []]);
	let selected = $state<string[]>([]);
	let handNo = $state(0);
	$effect(() => {
		if (v && v.tangan_ke !== handNo) {
			handNo = v.tangan_ke;
			rows = [[], [], []];
			selected = [];
		}
	});
	const placed = $derived(new Set(rows.flat()));

	function toggle(c: string) {
		selected = selected.includes(c) ? selected.filter((x) => x !== c) : [...selected, c];
	}

	function moveTo(r: number) {
		const room = ROW_SIZES[r] - rows[r].length;
		const take = selected.slice(0, room);
		rows = rows.map((row, i) => (i === r ? [...row, ...take] : row));
		selected = selected.filter((c) => !take.includes(c));
	}

	function takeBack(c: string) {
		rows = rows.map((row) => row.filter((x) => x !== c));
	}

	function suggestion() {
		if (v?.saran) rows = splitRows(v.saran);
		selected = [];
	}

	const command = $derived(arrangeCommand(rows));

	function seatName(i: number): string {
		return i === v!.kamu ? t('pk.you') : t('pk.seat', { n: i + 1 });
	}

	/** Kartu kursi lain dalam tiga kelompok slot (tertutup atau terbuka). */
	function seatRows(i: number): string[][] {
		const k = v!.kursi[i];
		return k.baris ?? splitRows(k.kartu);
	}

	function seatLine(i: number): string {
		const k = v!.kursi[i];
		const parts = [t('pk.stack', { n: k.tumpukan }), t(`cs.status.${k.status}` as Key)];
		if (k.istimewa) parts.push(t(`cs.special.${k.istimewa}` as Key));
		if (v!.fase !== 'susun' && k.status !== 'berdiri' && k.status !== 'habis' && (k.baris || k.istimewa)) {
			parts.push(t('cs.points', { n: signed(k.poin) }));
			if (k.nama_baris) parts.push(k.nama_baris.map((n) => t(`mj.hand.${n}` as Key)).join(' / '));
		}
		return parts.join(' · ');
	}

	const others = $derived(v ? v.kursi.map((_, i) => i).filter((i) => i !== v.kamu) : []);
	const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
	const handLeft = (j: number) => j * PICK;
</script>

{#if v && me}
	<div class="table" data-fase={v.fase} data-game={game}>
		{#each others as i (i)}
			<div class="row seat" data-seat={i}>
				<span class="label dim">{seatName(i)}{#if v.dealer === i}<span class="marks">D</span>{/if}</span>
				<div class="hand" data-row={`kursi-${i}`} style:width={`calc(${rowsWidth(PEEK)}px + 1ch)`}>
					<div class="cards">
						{#each seatRows(i) as row, r (r)}
							{#each row as c, j (j)}
								<span class="card" data-card={c} style:left={`${slotLeft(r, j, PEEK)}px`}>
									<PixelSprite pixels={cardPixels(c)} size={2} />
								</span>
							{/each}
						{/each}
					</div>
				</div>
				<p class="line" class:dim={v.kursi[i].status === 'berdiri' || v.kursi[i].status === 'habis'}>{seatLine(i)}</p>
			</div>
		{/each}

		<div class="row seat me" data-seat={v.kamu}>
			<span class="label">{t('cs.hand_label')}{#if v.dealer === v.kamu}<span class="marks">D</span>{/if}</span>
			<div class="hand" data-row="tangan" style:width={`calc(${12 * PICK}px + var(--card-w) + 1ch)`}>
				<div class="cards">
					{#if arranging}
						{#each me.kartu as c, j (c)}
							{#if !placed.has(c)}
								<span class="card pick" data-card={c} style:left={`${handLeft(j)}px`}>
									<NavButton pressed={selected.includes(c)} label={t('cs.pick_card', { card: c })} onclick={() => toggle(c)}>
										<PixelSprite pixels={cardPixels(c)} size={2} />
									</NavButton>
									{#if selected.includes(c)}<span class="layer chosen" aria-hidden="true"></span>{/if}
								</span>
							{/if}
						{/each}
					{/if}
				</div>
			</div>
			<p class="line" class:dim={!arranging}>{arranging ? t('cs.left', { n: 13 - placed.size }) : ''}</p>
		</div>

		<div class="row">
			<span class="label">{t('cs.rows_label')}</span>
			<div class="hand" data-row="susun" style:width={`calc(${rowsWidth(PICK)}px + 1ch)`}>
				<div class="cards">
					{#if arranging}
						{#each rows as row, r (r)}
							{#each row as c, j (c)}
								<span class="card pick" data-card={c} style:left={`${slotLeft(r, j, PICK)}px`}>
									<NavButton label={t('cs.unpick_card', { card: c })} onclick={() => takeBack(c)}>
										<PixelSprite pixels={cardPixels(c)} size={2} />
									</NavButton>
								</span>
							{/each}
						{/each}
					{:else}
						{#each seatRows(v.kamu) as row, r (r)}
							{#each row as c, j (j)}
								<span class="card" data-card={c} style:left={`${slotLeft(r, j, PICK)}px`}>
									<PixelSprite pixels={cardPixels(c)} size={2} />
								</span>
							{/each}
						{/each}
					{/if}
					{#each ROW_SIZES as size, r (r)}
						<span class="slots" aria-hidden="true" style:left={`${slotLeft(r, 0, PICK)}px`} style:width={`calc(${(size - 1) * PICK}px + var(--card-w))`}></span>
					{/each}
				</div>
			</div>
			<p class="line">{seatLine(v.kamu)}</p>
		</div>

		<div class="row info dim">
			<span>{t('pk.hand_no', { n: v.tangan_ke })}</span>
			<span>{t('cs.point_value', { n: v.poin_chip })}</span>
			{#if chips !== null}<span>{t('bj.chips', { n: chips })}</span>{/if}
		</div>

		{#if interactive}
			<div class="controls" aria-label={t('bj.controls')}>
				{#if v.fase === 'antara'}
					<div class="bet-row">
						<NavButton disabled={!can('next')} onclick={() => onplay('next')}>[ {t('pk.next')} ]</NavButton>
						<NavButton disabled={!can('leave')} onclick={() => onplay('leave')}>[ {t('pk.leave')} ]</NavButton>
					</div>
					<p class="hint dim">{t('pk.between_hint', { n: signed(me.tumpukan - me.awal) })}</p>
				{:else if arranging}
					<div class="bet-row">
						{#each ROW_SIZES as size, r (r)}
							<NavButton
								disabled={selected.length === 0 || rows[r].length >= size}
								onclick={() => moveTo(r)}>[ {t(`cs.to.${r}` as Key)} {rows[r].length}/{size} ]</NavButton
							>
						{/each}
					</div>
					<div class="bet-row">
						<NavButton disabled={!command || !can('arrange')} onclick={() => command && onplay(command)}
							>[ {t('cs.submit')} ]</NavButton
						>
						<NavButton disabled={!v.saran} onclick={suggestion}>[ {t('cs.suggest')} ]</NavButton>
						<NavButton disabled={placed.size === 0} onclick={() => ((rows = [[], [], []]), (selected = []))}
							>[ {t('cs.clear')} ]</NavButton
						>
						<NavButton sorot={lit('auto')} disabled={!can('auto')} onclick={() => onplay('auto')}>[ {t('cs.auto')} ]</NavButton>
					</div>
					<p class="hint dim">{t('cs.hint')}</p>
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
		padding: 0;
		z-index: 1;
	}
	.pick :global(.tbtn) {
		display: block;
		padding: 0;
		cursor: pointer;
	}
	.slots {
		position: absolute;
		top: 0;
		height: var(--card-h);
		outline: 1px dashed var(--dim);
		outline-offset: -1px;
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
		width: 48ch;
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
	/* Bilah aksi menempel di bawah area gulir (SPEC §4 Rev. 13). */
	.controls {
		position: sticky;
		bottom: 0;
		background: var(--bg);
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
	.hint {
		margin: 0;
		min-height: var(--cell-h);
	}
</style>
