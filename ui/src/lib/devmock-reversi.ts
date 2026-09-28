/**
 * Tiruan pertandingan Reversi untuk `npm run dev` di peramban (SPEC §11:
 * uji UI lewat peramban dengan backend tiruan). Aturan yang sebenarnya dan
 * provably fair ada di Rust dan diuji di sana; ini hanya cukup untuk
 * menggerakkan layar. Tidak ikut build produksi.
 */

import type { FairRecord, MatchState, ReplayData, ReplaySummary, Move } from './backend';
import type { ReversiView } from './games/reversi';

type Cell = '.' | 'X' | 'O';
const DIRS = [
	[0, 1],
	[0, -1],
	[1, 0],
	[-1, 0],
	[1, 1],
	[1, -1],
	[-1, 1],
	[-1, -1]
];

const name = (i: number) => `${'abcdefgh'[i % 8]}${Math.floor(i / 8) + 1}`;
const parse = (s: string) => (Number(s[1]) - 1) * 8 + 'abcdefgh'.indexOf(s[0]);

function initial(): Cell[] {
	const b: Cell[] = Array(64).fill('.');
	b[parse('d4')] = 'O';
	b[parse('e5')] = 'O';
	b[parse('d5')] = 'X';
	b[parse('e4')] = 'X';
	return b;
}

function flips(b: Cell[], me: Cell, i: number): number[] {
	if (b[i] !== '.') return [];
	const opp = me === 'X' ? 'O' : 'X';
	const out: number[] = [];
	const r0 = Math.floor(i / 8);
	const c0 = i % 8;
	for (const [dr, dc] of DIRS) {
		const line: number[] = [];
		let r = r0 + dr;
		let c = c0 + dc;
		while (r >= 0 && r < 8 && c >= 0 && c < 8 && b[r * 8 + c] === opp) {
			line.push(r * 8 + c);
			r += dr;
			c += dc;
		}
		if (line.length && r >= 0 && r < 8 && c >= 0 && c < 8 && b[r * 8 + c] === me) out.push(...line);
	}
	return out;
}

const moves = (b: Cell[], me: Cell) => b.map((_, i) => i).filter((i) => flips(b, me, i).length);

interface Game {
	board: Cell[];
	turn: 0 | 1;
	last: number | null;
	flipped: number[];
	moves: Move[];
	human: number;
	level: number;
}

let game: Game | null = null;
const replays: { summary: ReplaySummary; moves: Move[]; human: number }[] = [];

const color = (seat: number): Cell => (seat === 0 ? 'X' : 'O');
const over = (g: Game) => !moves(g.board, 'X').length && !moves(g.board, 'O').length;
const count = (b: Cell[], c: Cell) => b.filter((x) => x === c).length;

function winners(b: Cell[]): number[] {
	const x = count(b, 'X');
	const o = count(b, 'O');
	return x > o ? [0] : o > x ? [1] : [];
}

function view(g: Game, seat: number): ReversiView {
	const done = over(g);
	const legal = done ? [] : moves(g.board, color(g.turn)).map(name);
	return {
		papan: Array.from({ length: 8 }, (_, r) => g.board.slice(r * 8, r * 8 + 8).join('')),
		giliran: done ? null : g.turn,
		kamu: seat,
		legal: !done && legal.length === 0 ? ['pass'] : legal,
		hitam: count(g.board, 'X'),
		putih: count(g.board, 'O'),
		terakhir: g.last === null ? null : name(g.last),
		dibalik: g.flipped.map(name),
		selesai: done,
		pemenang: done ? winners(g.board) : null,
		menyerah: false
	};
}

function apply(g: Game, cmd: string) {
	if (cmd === 'pass') {
		g.last = null;
		g.flipped = [];
	} else {
		const i = parse(cmd);
		const f = flips(g.board, color(g.turn), i);
		if (!f.length) throw { id: `\`${cmd}\` bukan aksi yang sah saat ini.`, en: `\`${cmd}\` is not a legal action right now.` };
		g.board[i] = color(g.turn);
		for (const j of f) g.board[j] = color(g.turn);
		g.last = i;
		g.flipped = f;
	}
	g.moves.push({ seat: g.turn, command: cmd });
	g.turn = g.turn === 0 ? 1 : 0;
}

const fakeHex = (n: number) => n.toString(16).padStart(2, '0').repeat(32);
const fair = (): FairRecord => ({
	host: 'host',
	commitments: { host: fakeHex(0xa1), player: fakeHex(0xb2) },
	seeds: { host: fakeHex(0x11), player: fakeHex(0x22) },
	excluded: [],
	round_seed: fakeHex(0x33)
});
const verifyOk = {
	ok: true,
	checks: (['commitments', 'round_seed', 'moves', 'result', 'state_hash'] as const).map((step) => ({
		step,
		ok: true
	})),
	error: null
};

function summary(g: Game) {
	const b = count(g.board, 'X');
	const w = count(g.board, 'O');
	return {
		winners: winners(g.board),
		scores: [b, w],
		summary: { id: `Selesai (tiruan): ${b}–${w}.`, en: `Game over (mock): ${b}–${w}.` }
	};
}

function state(g: Game): MatchState {
	const done = over(g);
	const v = view(g, g.human);
	const yours = !done && g.turn === g.human;
	return {
		game: 'reversi',
		seat: g.human,
		seats: [0, 1].map((s) => (s === g.human ? { kind: 'human' } : { kind: 'bot', level: g.level })),
		your_turn: yours,
		bot_turn: !done && g.turn !== g.human,
		over: done,
		view_data: v,
		view_text: { id: '(tiruan)', en: '(mock)' },
		actions: yours
			? v.legal.map((c) => ({ spec: { kind: 'fixed', command: c }, usage: c, concrete: [c] }))
			: [],
		moves: [...g.moves],
		commitments: fair().commitments,
		reveal: done ? fair() : null,
		result: done ? summary(g) : null,
		verify: done ? verifyOk : null,
		replay_id: done ? replays.length : null,
		save_error: null,
		clock: null
	};
}

function save(g: Game) {
	replays.push({
		summary: {
			id: replays.length + 1,
			game: 'reversi',
			started_at: Date.now(),
			finished: over(g),
			moves: g.moves.length,
			seats: state(g).seats,
			result: over(g) ? summary(g) : null
		},
		moves: [...g.moves],
		human: g.human
	});
}

export const reversiMock = {
	match_flag: async (): Promise<MatchState> => {
		if (!game) throw { id: 'Tidak ada permainan yang berjalan.', en: 'No game is running.' };
		return state(game);
	},
	replay_pgn: async (): Promise<string> => {
		throw { id: 'PGN hanya untuk replay catur.', en: 'PGN is only for chess replays.' };
	},
	pgn_open: async (): Promise<ReplayData> => {
		throw { id: 'Tiruan dev tidak punya mesin catur.', en: 'The dev mock has no chess engine.' };
	},
	match_start: async (_id: string, level: number, seat: number): Promise<MatchState> => {
		if (game && !over(game) && game.moves.length) save(game);
		game = { board: initial(), turn: 0, last: null, flipped: [], moves: [], human: seat, level };
		return state(game);
	},
	match_act: async (command: string): Promise<MatchState> => {
		if (!game) throw { id: 'Tidak ada permainan yang berjalan.', en: 'No game is running.' };
		apply(game, command);
		if (over(game)) save(game);
		return state(game);
	},
	match_step: async (): Promise<MatchState> => {
		if (!game) throw { id: 'Tidak ada permainan yang berjalan.', en: 'No game is running.' };
		if (!over(game) && game.turn !== game.human) {
			const legal = moves(game.board, color(game.turn));
			apply(game, legal.length ? name(legal[Math.floor(Math.random() * legal.length)]) : 'pass');
			if (over(game)) save(game);
		}
		return state(game);
	},
	match_leave: async () => {
		if (game && !over(game) && game.moves.length) save(game);
		game = null;
	},
	replay_list: async (): Promise<ReplaySummary[]> => [...replays].reverse().map((r) => r.summary),
	replay_open: async (id: number): Promise<ReplayData> => {
		const r = replays[id - 1];
		if (!r) throw { id: `Replay ${id} tidak ditemukan.`, en: `Replay ${id} not found.` };
		const g: Game = { board: initial(), turn: 0, last: null, flipped: [], moves: [], human: r.human, level: 1 };
		const frames = [{ index: 0, last: null, view_data: view(g, r.human), view_text: { id: '', en: '' } }];
		for (const [i, mv] of r.moves.entries()) {
			apply(g, mv.command);
			frames.push({ index: i + 1, last: mv, view_data: view(g, r.human), view_text: { id: '', en: '' } } as never);
		}
		return {
			id,
			game: 'reversi',
			seat: r.human,
			seats: r.summary.seats,
			frames,
			fair: fair(),
			result: r.summary.result,
			verify: verifyOk,
			tags: []
		};
	}
};

/** Keadaan awal untuk tutorial tiruan. */
export function reversiStartView(seat = 0): ReversiView {
	return view({ board: initial(), turn: 0, last: null, flipped: [], moves: [], human: seat, level: 1 }, seat);
}
