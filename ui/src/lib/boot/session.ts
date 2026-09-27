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
}

function read(): SessionRecord {
	try {
		const raw = localStorage.getItem(KEY);
		if (raw) {
			const r = JSON.parse(raw) as Partial<SessionRecord>;
			return { last: r.last ?? null, greeting: r.greeting ?? null };
		}
	} catch {
		// Penyimpanan tidak tersedia; anggap sesi pertama.
	}
	return { last: null, greeting: null };
}

function write(r: SessionRecord) {
	try {
		localStorage.setItem(KEY, JSON.stringify(r));
	} catch {
		// Abaikan; hanya memengaruhi sapaan berikutnya.
	}
}

/** Membaca sesi sebelumnya lalu mencatat sesi ini sebagai yang terakhir. */
export function startSession(now = Date.now()): SessionRecord {
	const previous = read();
	write({ last: now, greeting: previous.greeting });
	return previous;
}

export function rememberGreeting(id: string) {
	write({ ...read(), greeting: id });
}
