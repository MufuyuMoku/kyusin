/**
 * Tiruan backend hanya untuk `npm run dev` di peramban biasa (tanpa Tauri),
 * supaya tampilan bisa dicek cepat. Isinya meniru game fixture; aturan yang
 * sebenarnya tetap di Rust dan diuji di sana. Tidak ikut build produksi.
 */

import type { AppInfo, Category, Step, TutorialState } from './backend';

const steps: Step[] = [
	{
		teks: 'Ada 7 batang. Setiap giliran kamu boleh mengambil 1 sampai 3.',
		sebelum: [],
		aksi: null,
		sorot: [],
		petunjuk: null
	},
	{
		teks: 'Ambil 3 batang supaya tersisa 4.',
		sebelum: [],
		aksi: 'take 3',
		sorot: ['aksi:take 3'],
		petunjuk: 'Pilih tombol yang mengambil tiga batang.'
	},
	{
		teks: 'Lawan mengambil 1, tersisa 3. Ambil semuanya untuk menang.',
		sebelum: ['take 1'],
		aksi: 'take 3',
		sorot: ['aksi:take 3'],
		petunjuk: 'Batang terakhir menang: ambil ketiganya.'
	},
	{
		teks: 'Kamu mengambil batang terakhir dan menang.',
		sebelum: [],
		aksi: null,
		sorot: [],
		petunjuk: null
	}
];

let index = 0;
let left = 7;

function text(): string {
	const sticks = Array.from({ length: left }, () => '│').join(' ');
	const status = left === 0 ? 'Kamu menang.' : 'Giliranmu.';
	return `Batang tersisa: ${left}\n\n  ${sticks}\n\n${status}`;
}

function state(feedback: TutorialState['feedback'] = null): TutorialState {
	const step = steps[index] ?? null;
	const max = Math.min(3, left);
	return {
		game: 'fixture',
		title: 'Batang: dasar',
		index,
		total: steps.length,
		finished: index >= steps.length,
		step,
		view_text: text(),
		actions: step?.aksi
			? [
					{
						spec: {
							kind: 'template',
							verb: 'take',
							params: [{ name: 'n', type: 'int', min: 1, max, step: 1 }]
						},
						usage: 'take <n>',
						concrete: Array.from({ length: max }, (_, i) => `take ${i + 1}`)
					}
				]
			: [],
		feedback
	};
}

function enter() {
	for (const c of steps[index]?.sebelum ?? []) left -= Number(c.split(' ')[1]);
}

export const devApi = {
	app_info: async (): Promise<AppInfo> => ({ name: 'KyuSin', version: '0.1.0-dev' }),
	catalog: async (): Promise<Category[]> => [
		{
			key: 'uji',
			label: 'Uji (fixture)',
			games: [
				{
					id: 'fixture',
					nama: 'Fixture: Batang',
					kategori: 'uji',
					pemain_min: 2,
					pemain_maks: 2,
					jenis: 'giliran',
					lawan: 'bot',
					kompetitif: false,
					lan: false,
					agen: false,
					rtp: null,
					tutorial: 'crates/games/src/fixture/tutorial.toml',
					perintah: [{ pola: 'take <n>', ringkas: 'Ambil 1 sampai 3 batang' }],
					rtp_line: null
				}
			]
		}
	],
	man: async (id: string): Promise<string> => {
		if (id !== 'fixture') throw `man: tidak ada game \`${id}\``;
		return 'NAMA\n    Fixture: Batang (fixture)\n\n(tiruan dev; jalankan lewat Tauri untuk halaman man asli)\n\nPERINTAH\n    take <n>  Ambil 1 sampai 3 batang\n';
	},
	tutorial_start: async (id: string): Promise<TutorialState> => {
		if (id !== 'fixture') throw `tutorial: tidak ada game \`${id}\``;
		index = 0;
		left = 7;
		enter();
		return state();
	},
	tutorial_act: async (command: string): Promise<TutorialState> => {
		const step = steps[index];
		if (!step?.aksi) return state({ kind: 'wrong', hint: 'Langkah ini hanya bacaan; pilih [ LANJUT ].' });
		if (command.trim().split(/\s+/).join(' ') !== step.aksi)
			return state({ kind: 'wrong', hint: step.petunjuk ?? '' });
		left -= Number(step.aksi.split(' ')[1]);
		index += 1;
		enter();
		return state({ kind: 'correct' });
	},
	tutorial_next: async (): Promise<TutorialState> => {
		index += 1;
		enter();
		return state();
	},
	tutorial_stop: async (): Promise<void> => {}
};
