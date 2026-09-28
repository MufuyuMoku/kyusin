<!--
  Konsol perintah `> _` (SPEC §4). Opsional: mati secara bawaan, dibuka
  dengan `:` atau `` ` ``. Autocomplete (Tab) dan riwayat (↑/↓).
-->
<script lang="ts">
	import { tick } from 'svelte';
	import {
		app,
		current,
		findGame,
		gameCommands,
		go,
		home,
		matchAct,
		print,
		requestBack,
		startMatch,
		startTutorial,
		tutorialAct
	} from '$lib/app.svelte';
	import {
		EFFECT_IDS,
		GLOBAL_COMMANDS,
		History,
		LANG_IDS,
		THEME_IDS,
		complete,
		parse
	} from '$lib/commands';
	import { helpText, lsText } from '$lib/help';
	import { errorText, lang, t, type Key } from '$lib/i18n.svelte';
	import { forcedOff, save, settings, type Effect, type Lang, type Theme } from '$lib/settings.svelte';

	const HISTORY_KEY = 'kyusin.history.v1';

	function loadHistory(): string[] {
		try {
			return JSON.parse(localStorage.getItem(HISTORY_KEY) ?? '[]');
		} catch {
			return [];
		}
	}

	const history = new History(loadHistory());
	let value = $state('');
	let caret = $state(0);
	let options = $state<string[]>([]);
	let input: HTMLInputElement | undefined = $state();
	let outputEl: HTMLDivElement | undefined = $state();

	const visible = $derived(app.consoleOpen || settings.consoleAlways);

	$effect(() => {
		if (app.consoleOpen) tick().then(() => input?.focus());
	});

	$effect(() => {
		void app.output.length;
		tick().then(() => outputEl?.scrollTo({ top: outputEl.scrollHeight }));
	});

	function ctx() {
		return {
			gameIds: app.catalog.flatMap((c) => c.games.map((g) => g.id)),
			categoryKeys: app.catalog.map((c) => c.key),
			gameCommands: gameCommands()
		};
	}

	function persist() {
		try {
			localStorage.setItem(HISTORY_KEY, JSON.stringify(history.entries));
		} catch {
			// Riwayat berlaku untuk sesi ini saja.
		}
	}

	async function run(line: string) {
		history.push(line);
		persist();
		print(`> ${line}`);
		const p = parse(line, ctx());
		try {
			switch (p.kind) {
				case 'empty':
					return;
				case 'unknown':
					print(t('console.unknown', { name: p.name }));
					return;
				case 'game': {
					if (current().name === 'match' && app.match) {
						await matchAct(p.command);
						return;
					}
					if (!app.tutorial) {
						print(t('console.no_game'));
						return;
					}
					await tutorialAct(p.command);
					const f = app.tutorial?.feedback;
					if (f?.kind === 'wrong') print(`! ${f.hint[lang()]}`);
					return;
				}
				case 'global':
					return runGlobal(p.name, p.args);
			}
		} catch (e) {
			print(errorText(e));
		}
	}

	async function runGlobal(name: string, args: string[]) {
		const [a, b] = args;
		switch (name) {
			case 'help': {
				const c = GLOBAL_COMMANDS.find((c) => c.name === a);
				print(c ? `${c.name} ${c.args}  —  ${t(c.summary as Key)}` : helpText(app.catalog));
				return;
			}
			case 'ls':
				print(lsText(app.catalog, a));
				return;
			case 'man':
			case 'tutorial': {
				if (!a)
					return print(
						t('console.need_id', { command: name, example: `${name} ${ctx().gameIds[0] ?? '<id>'}` })
					);
				if (!findGame(a)) return print(t('console.unknown_game', { command: name, id: a }));
				if (name === 'man') go({ name: 'game', id: a });
				else await startTutorial(a);
				return;
			}
			case 'play': {
				const game = a ? findGame(a) : undefined;
				if (!a) return print(t('console.play_usage'));
				if (!game) return print(t('console.unknown_game', { command: name, id: a }));
				if (!game.bot_levels) return print(t('console.no_bot', { id: a }));
				const level = Number(b ?? 1);
				if (!Number.isInteger(level) || level < 1 || level > game.bot_levels)
					return print(t('console.play_usage'));
				await startMatch(a, level, 0);
				return;
			}
			case 'verify': {
				const report =
					current().name === 'replay' ? app.replay?.verify : current().name === 'match' ? app.match?.verify : null;
				if (!report) return print(t('console.no_verify'));
				for (const c of report.checks) print(`${c.ok ? '[✓]' : '[×]'} ${t(`verify.step.${c.step}` as Key)}`);
				print(report.ok ? t('verify.ok') : t('verify.fail'));
				if (report.error) print(report.error[lang()]);
				return;
			}
			case 'menu':
				return home();
			case 'back':
				return requestBack();
			case 'settings':
				if (current().name !== 'settings') go({ name: 'settings' });
				return;
			case 'theme':
				if (!THEME_IDS.includes(a)) return print(t('console.theme_usage', { options: THEME_IDS.join(', ') }));
				settings.theme = a as Theme;
				save();
				return;
			case 'fx':
				if (!EFFECT_IDS.includes(a) || (b !== 'on' && b !== 'off'))
					return print(t('console.fx_usage', { options: EFFECT_IDS.join('|') }));
				settings.fx[a as Effect] = b === 'on';
				save();
				if (b === 'on' && forcedOff(a as Effect)) print(t('console.reduced', { effect: a }));
				return;
			case 'lang':
				if (!LANG_IDS.includes(a)) return print(t('console.lang_usage'));
				settings.lang = a as Lang;
				save();
				return;
			case 'clear':
				app.output = [];
				return;
		}
	}

	function syncCaret() {
		caret = input?.selectionStart ?? value.length;
	}

	async function onkeydown(e: KeyboardEvent) {
		if (e.key === 'Tab') {
			e.preventDefault();
			const c = complete(value, ctx());
			value = c.value;
			options = c.options.length > 1 ? c.options : [];
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			value = history.up(value);
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			value = history.down();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const line = value;
			value = '';
			options = [];
			await run(line);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			options = [];
			app.consoleOpen = false;
			input?.blur();
			return;
		} else {
			options = [];
		}
		await tick();
		input?.setSelectionRange(value.length, value.length);
		syncCaret();
	}
</script>

{#if visible}
	<section class="console" aria-label={t('console.label')}>
		{#if app.output.length}
			<div class="output" bind:this={outputEl} aria-live="polite">
				{#each app.output as line, i (i)}
					<div>{line || ' '}</div>
				{/each}
			</div>
		{/if}
		{#if options.length}
			<div class="options dim">{options.join('  ')}</div>
		{/if}
		<label class="prompt">
			<span aria-hidden="true">&gt;&nbsp;</span>
			<span class="field">
				<input
					bind:this={input}
					bind:value
					{onkeydown}
					oninput={syncCaret}
					onclick={syncCaret}
					onkeyup={syncCaret}
					onfocus={() => (app.consoleOpen = true)}
					spellcheck="false"
					autocomplete="off"
					aria-label={t('console.input')}
				/>
				<span class="mirror" aria-hidden="true"
					>{value.slice(0, caret)}<span class="cursor">{value[caret] ?? ' '}</span></span
				>
			</span>
		</label>
	</section>
{/if}

<style>
	.console {
		border-top: 1px solid var(--dim);
		padding: 0.25rem 2ch 0.5rem;
	}
	.output {
		max-height: calc(var(--cell-h) * 8);
		overflow-y: auto;
		white-space: pre-wrap;
	}
	.options {
		white-space: pre-wrap;
	}
	.prompt {
		display: flex;
		white-space: pre;
	}
	.field {
		position: relative;
		flex: 1;
	}
	input {
		position: absolute;
		inset: 0;
		width: 100%;
		font: inherit;
		color: transparent;
		caret-color: transparent;
		background: transparent;
		border: 0;
		padding: 0;
		outline: none;
	}
	input:focus-visible {
		background: transparent;
	}
	.mirror {
		white-space: pre;
		pointer-events: none;
	}
	.cursor {
		color: var(--bg);
	}
	.field:not(:focus-within) .cursor {
		animation: none;
		background: transparent;
		color: inherit;
		outline: 1px solid var(--dim);
	}
</style>
