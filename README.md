# Web x86 (32-bit Subset) Emulator

Educational 32-bit x86 subset emulator for the web: registers, stack, calling conventions, debugger, visualizations, and sandboxed virtual I/O.

## Quick Start
```bash
# requires: Node 18+, Rust + wasm-pack, pnpm (or npm/yarn)
pnpm i
# build all workspace packages
pnpm -w build
# start the frontend dev server (pnpm workspace)
pnpm -w --filter frontend dev

# If you don't use pnpm, from the `frontend` folder you can use npm/yarn:
cd frontend
npm install
npm run dev
```