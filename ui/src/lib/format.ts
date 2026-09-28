/**
 * Mengisi parameter teks terjemahan (D-039).
 *
 * - `{nama}` diganti nilai parameter.
 * - `{nama|tunggal|jamak}` memilih bentuk menurut aturan jamak bahasa
 *   (`Intl.PluralRules`) untuk angka `nama`, misalnya
 *   `{n} {n|chip|chips}` → "1 chip", "2 chips". Nilai yang bukan angka
 *   (misalnya rentang "2–4") memakai bentuk jamak. Bahasa Indonesia tidak
 *   memakainya karena kata benda tidak berubah bentuk.
 *
 * Tanpa ketergantungan Svelte supaya bisa diuji dengan `node --test`.
 */

export type Params = Record<string, string | number>;

export function format(template: string, params: Params, lang: string): string {
	const rules = new Intl.PluralRules(lang);
	let s = template.replace(/\{(\w+)\|([^|}]*)\|([^}]*)\}/g, (_, name: string, one: string, other: string) => {
		const n = Number(params[name]);
		return Number.isFinite(n) && rules.select(n) === 'one' ? one : other;
	});
	for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, String(v));
	return s;
}

/** Sisa jam catur sebagai `mm:ss` (dibulatkan ke atas, tidak negatif). */
export function clockText(ms: number): string {
	const total = Math.max(0, Math.ceil(ms / 1000));
	const m = Math.floor(total / 60);
	const s = total % 60;
	return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Selisih bertanda untuk perubahan rating: `+12`, `-8`, `0`. */
export function signed(n: number): string {
	return n > 0 ? `+${n}` : `${n}`;
}
