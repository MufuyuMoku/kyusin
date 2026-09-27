<!--
  Tombol bersama untuk semua kontrol yang bisa dipilih (D-033). Selalu bisa
  difokus: saat nonaktif memakai `aria-disabled` (bukan `disabled`), alasan
  dibacakan lewat `aria-describedby`, dan Enter/Spasi/klik tidak berefek.
  Navigasi panah ada di halaman utama (lib/nav.ts).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		onclick,
		disabled = false,
		reason = '',
		pressed = undefined,
		sorot = false,
		label = undefined,
		lang = undefined,
		children
	}: {
		onclick: () => void;
		disabled?: boolean;
		/** Keterangan mengapa nonaktif; tampil dan dibacakan. */
		reason?: string;
		pressed?: boolean;
		/** Disorot tutorial. */
		sorot?: boolean;
		label?: string;
		lang?: string;
		children: Snippet;
	} = $props();

	const reasonId = `nav-reason-${Math.random().toString(36).slice(2, 10)}`;

	function click(e: MouseEvent) {
		if (disabled) {
			e.preventDefault();
			return;
		}
		onclick();
	}
</script>

<button
	class="tbtn"
	class:sorot
	data-nav
	aria-disabled={disabled ? 'true' : undefined}
	aria-describedby={disabled && reason ? reasonId : undefined}
	aria-pressed={pressed}
	aria-label={label}
	{lang}
	onclick={click}>{@render children()}</button
>{#if disabled && reason}<span class="dim reason" id={reasonId}> — {reason}</span>{/if}
