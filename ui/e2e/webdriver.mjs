// Klien WebDriver kecil untuk tauri-driver (SPEC §11): perintah hanya masuk
// ke jendela KyuSin lewat WebDriver, bukan ke sistem operasi. Tanpa paket
// npm tambahan; cukup `fetch` bawaan Node.

const ELEMENT = 'element-6066-11e4-a52e-4f735466cecf';

export const KEYS = {
	up: '',
	down: '',
	left: '',
	right: '',
	enter: '',
	escape: ''
};

export class Session {
	constructor(base, id) {
		this.base = base;
		this.id = id;
	}

	/**
	 * Menempel ke WebView2 aplikasi yang sudah berjalan dengan port DevTools
	 * `debuggerAddress` (cara WebView2 yang didokumentasikan Microsoft).
	 * Diteruskan tauri-driver ke msedgedriver apa adanya (D-040).
	 */
	static async attach(base, debuggerAddress) {
		const res = await call(base, 'POST', '/session', {
			capabilities: {
				alwaysMatch: {
					browserName: 'webview2',
					'ms:edgeChromium': true,
					'ms:edgeOptions': { debuggerAddress }
				}
			}
		});
		return new Session(base, res.sessionId);
	}

	cmd(method, path, body) {
		return call(this.base, method, `/session/${this.id}${path}`, body);
	}

	/** Menjalankan fungsi di halaman; `fn` diubah jadi teks, argumen lewat `args`. */
	exec(fn, ...args) {
		return this.cmd('POST', '/execute/sync', {
			script: `return (${fn.toString()}).apply(null, arguments);`,
			args
		});
	}

	async waitFor(fn, what, timeoutMs = 20000, ...args) {
		const until = Date.now() + timeoutMs;
		let last;
		while (Date.now() < until) {
			try {
				last = await this.exec(fn, ...args);
				if (last) return last;
			} catch (e) {
				last = e;
			}
			await new Promise((r) => setTimeout(r, 200));
		}
		throw new Error(`waktu habis menunggu ${what} (terakhir: ${last})`);
	}

	async find(using, value) {
		const res = await this.cmd('POST', '/element', { using, value });
		return res[ELEMENT];
	}

	click(el) {
		return this.cmd('POST', `/element/${el}/click`, {});
	}

	type(el, text) {
		return this.cmd('POST', `/element/${el}/value`, { text });
	}

	/** Gerakan penunjuk ke tengah elemen (hover). */
	hover(el) {
		return this.cmd('POST', '/actions', {
			actions: [
				{
					type: 'pointer',
					id: 'mouse',
					parameters: { pointerType: 'mouse' },
					actions: [{ type: 'pointerMove', duration: 0, origin: { [ELEMENT]: el }, x: 0, y: 0 }]
				}
			]
		});
	}

	async screenshot() {
		return Buffer.from(await this.cmd('GET', '/screenshot'), 'base64');
	}

	async elementScreenshot(el) {
		return Buffer.from(await this.cmd('GET', `/element/${el}/screenshot`), 'base64');
	}

	end() {
		return this.cmd('DELETE', '');
	}
}

async function call(base, method, path, body) {
	const res = await fetch(base + path, {
		method,
		headers: { 'content-type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body)
	});
	const json = await res.json().catch(() => ({}));
	if (!res.ok || json.value?.error) {
		throw new Error(`${method} ${path}: ${res.status} ${JSON.stringify(json.value ?? json)}`);
	}
	return json.value;
}

export { ELEMENT };
