import { svelte } from '@sveltejs/vite-plugin-svelte'
import { execSync } from 'node:child_process'
import { defineConfig } from 'vite'

function gitRevision(): string {
    try {
        return execSync('git rev-parse HEAD', { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim()
    } catch {
        return ''
    }
}

// https://vite.dev/config/
export default defineConfig({
    plugins: [svelte()],
    define: {
        __GIT_REVISION__: JSON.stringify(gitRevision())
    },
    worker: {
        format: 'es'
    }
})
