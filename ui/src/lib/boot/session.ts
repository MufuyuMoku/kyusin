/**
 * Catatan sesi lokal untuk urutan boot (SPEC §4, D-031): kapan aplikasi
 * terakhir dibuka dan sapaan apa yang terakhir tampil. Hanya di
 * penyimpanan jendela; tidak ada yang keluar dari perangkat.
 */

const KEY = 'kyusin.session.v1';

export interface SessionRecord {
	/** Waktu sesi sebelumnya dimulai (epoch ms); `null` = belum pernah. */
	last: number | null;
	/** Id sapaan yang terakhir tampil. */
	greeting: string | null;
	/** Tanggal lokal (YYYY-MM-DD) hari-hari dengan sesi, terbaru di akhir. */
	days: string[];
}

const MAX_DAYS = 60;

/** Tanggal kalender lokal, YYYY-MM-DD. */
export function localDay(ms: number): string {
	const d = new Date(ms);
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** Jumlah hari kalender berturut-turut yang berakhir di `today`. */
export function streak(days: readonly string[], today: number): number {
	const set = new Set(days);
	let n = 0;
	const d = new Date(today);
	while (set.has(localDay(d.getTime()))) {
		n += 1;
		d.setDate(d.getDate() - 1);
	}
	return n;
}

function read(): SessionRecord {
	try {
		const raw = localStorage.getItem(KEY);
		if (raw) {
			const r = JSON.parse(raw) as Partial<SessionRecord>;
			return { last: r.last ?? null, greeting: r.greeting ?? null, days: r.days ?? [] };
		}
	} catch {
		// Penyimpanan tidak tersedia; anggap sesi pertama.
	}
	return { last: null, greeting: null, days: [] };
}

function write(r: SessionRecord) {
	try {
		localStorage.setItem(KEY, JSON.stringify(r));
	} catch {
		// Abaikan; hanya memengaruhi sapaan berikutnya.
	}
}

/**
 * Membaca sesi sebelumnya lalu mencatat sesi ini sebagai yang terakhir.
 * `days` di hasilnya sudah termasuk hari ini (untuk sapaan rentetan hari).
 */
export function startSession(now = Date.now()): SessionRecord {
	const previous = read();
	const today = localDay(now);
	const days = [...previous.days.filter((d) => d !== today), today].slice(-MAX_DAYS);
	write({ last: now, greeting: previous.greeting, days });
	return { ...previous, days };
}

export function rememberGreeting(id: string) {
	write({ ...read(), greeting: id });
}
