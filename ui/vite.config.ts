import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { mkdirSync, writeFileSync } from 'node:fs';
import { defineConfig, type Plugin } from 'vite';

/**
 * Mencatat paket npm yang benar-benar masuk bundle klien ke
 * `.svelte-kit/bundled-packages.json`. Pemeriksa lisensi npm
 * (scripts/check-npm-licenses.mjs) membaca daftar ini (SPEC §3, D-006).
 */
function bundledPackages(): Plugin {
	return {
		name: 'kyusin-bundled-packages',
		apply: 'build',
		generateBundle(_options, bundle) {
			if (this.environment?.name !== 'client') return;
			const dirs = new Set<string>();
			for (const chunk of Object.values(bundle)) {
				if (chunk.type !== 'chunk') continue;
				for (const id of chunk.moduleIds) {
					const path = id.replace(/\\/g, '/').replace(/^\0/, '');
					const i = path.lastIndexOf('/node_modules/');
					if (i < 0) continue;
					const rest = path.slice(i + '/node_modules/'.length).split('/');
					const name = rest[0].startsWith('@') ? `${rest[0]}/${rest[1]}` : rest[0];
					dirs.add(path.slice(0, i) + '/node_modules/' + name);
				}
			}
			mkdirSync('.svelte-kit', { recursive: true });
			writeFileSync('.svelte-kit/bundled-packages.json', JSON.stringify([...dirs].sort(), null, '\t'));
		}
	};
}

// SPA statis yang dimuat jendela Tauri; tidak ada server (SPEC §3).
export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: 'index.html',
				precompress: false,
				strict: true
			})
		}),
		bundledPackages()
	],
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		watch: { ignored: ['**/src-tauri/**'] }
	}
});
