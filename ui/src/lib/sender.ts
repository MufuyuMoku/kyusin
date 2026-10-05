/**
 * Pengirim perintah dari papan/meja ke host (D-067): paling banyak satu
 * perintah sedang dikirim; klik lain selama itu diabaikan supaya perintah
 * tidak terkirim dua kali. Bila host tidak menjawab dalam `timeoutMs`,
 * penanda dilepas dan kesalahan `TIMEOUT` dilaporkan, jadi kontrol bisa
 * dipakai lagi dan meja tidak terkunci. Jawaban yang datang terlambat
 * diabaikan oleh pengirim (layar tetap diperbarui oleh pemanggil).
 */

export const TIMEOUT = Symbol('timeout');
export const ACT_TIMEOUT_MS = 10_000;

export type SendError = typeof TIMEOUT | unknown;

export class Sender {
	sending: string | null = null;
	private token = 0;
	private readonly timeoutMs: number;
	private readonly onUpdate: (sending: string | null, error: SendError | null) => void;

	constructor(timeoutMs: number, onUpdate: (sending: string | null, error: SendError | null) => void) {
		this.timeoutMs = timeoutMs;
		this.onUpdate = onUpdate;
	}

	/** Menjalankan `run` untuk `command`, kecuali ada perintah lain yang sedang dikirim. */
	async send(command: string, run: () => Promise<unknown>): Promise<void> {
		if (this.sending) return;
		const mine = ++this.token;
		this.sending = command;
		this.onUpdate(command, null);
		let timer: ReturnType<typeof setTimeout> | undefined;
		const timeout = new Promise<typeof TIMEOUT>((resolve) => {
			timer = setTimeout(() => resolve(TIMEOUT), this.timeoutMs);
		});
		let error: SendError | null = null;
		try {
			const result = await Promise.race([run().then(() => null), timeout]);
			if (result === TIMEOUT) error = TIMEOUT;
		} catch (e) {
			error = e;
		} finally {
			clearTimeout(timer);
		}
		if (mine !== this.token) return;
		this.sending = null;
		this.onUpdate(null, error);
	}
}
