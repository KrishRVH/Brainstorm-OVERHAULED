# Brainstorm OVERHAULED for Balatro

Current release: **v4.0.3**.

This has been a year-long rewrite project. Its development commits were
squashed and its prerewrite releases pruned to minimize confusion for users.

<img width="1820" height="1626" alt="image" src="https://github.com/user-attachments/assets/37c21cae-9b54-45a4-bbed-e75440d9dfbc" />


**Just want to install it?** Use the packaged release below. Building from
source and installing the CUDA Toolkit are not required.

Brainstorm OVERHAULED is a Balatro mod that rapidly searches for seeds
matching voucher/pack/tag/Joker/Erratic Deck filters and integrates directly
into the game loop through Lua plus a native Rust DLL.

Brainstorm OVERHAULED is KRVH's full rewrite of OceanRamen's original
Brainstorm. Its native engine, Hex, is KRVH's Rust rewrite of the core
seed-search engine from MathIsFun0's original Immolate, source-verified against
Balatro and extended for Brainstorm. It adds first-shop Joker search, dual-tag
filters, Erratic Deck filters, save/load state slots, searchable Joker UI,
resettable preferences, live auto-reroll scan counts, benchmark automation,
release packaging, Steamodded metadata, and Lovely loader hooks.

## Installation

1. Install the [latest Steamodded release](https://github.com/Steamodded/smods/wiki/Installing-Steamodded-windows#step-3-installing-steamodded) for Balatro.
2. Install [Lovely](https://github.com/ethangreen-dev/lovely-injector).
3. Download the zip from the
   [latest release](https://github.com/KrishRVH/Brainstorm-OVERHAULED/releases/latest).
4. Extract it into `%AppData%\Balatro\Mods\`. The archive already
   contains the required `Brainstorm` folder.
5. Reload Balatro.

Experimental CUDA acceleration is included but disabled by default. Enable it
with `AR: Use CUDA (Experimental)` on a compatible NVIDIA GPU. It requires no
CUDA Toolkit or custom files and falls back to the Rust CPU engine
automatically when the GPU path is unavailable or unsupported.

## Credits

This project is licensed under CC BY-NC-SA 4.0.

- Brainstorm OVERHAULED is KRVH's full rewrite of OceanRamen's original
  Brainstorm:
  https://github.com/OceanRamen/Brainstorm. It is licensed under the Mozilla
  Public License Version 2.0.
- Hex is KRVH's Rust rewrite of the core seed-search engine from MathIsFun0's
  original Immolate, source-verified against Balatro and extended for
  Brainstorm:
  https://github.com/SpectralPack/Immolate/tree/26f41efcc313f045bc8bdbf49e5851c56ac40b31.

## Features

- Auto-reroll with dual-tag support (order-agnostic or same-tag-twice).
- First-shop filters: voucher, two pack slots (e.g., Mega Spectral), specific
  Joker in shop slots or Buffoon packs, Observatory (Telescope + Mega
  Celestial), Perkeo (The Soul rolls Perkeo).
- Erratic Deck filters for face-card count, no-face searches, and suit-ratio
  searches.
- Joker list is alphabetized, searchable, and excludes first-shop impossible
  targets such as Legendary/Soul-only Jokers, enhancement-gated Jokers, and
  pool-flag-gated Jokers.
- Enable toggle, save/load state (Z/X + 1-5), reroll hotkeys (Ctrl+R,
  Ctrl+A), settings UI (Ctrl+T), and throttled live scan-count text during
  auto-reroll.
- Rust benchmark harness compares current speed against the Original Brainstorm
  DLL where the older ABI supports the same fixture, and reports comparable
  result mismatches.
- Optional experimental CUDA acceleration for supported filters, controlled by
  the `AR: Use CUDA (Experimental)` setting; unavailable or unsupported GPU
  searches fall back to the Rust CPU engine.
- Locally verified release artifacts are published as immutable versioned
  GitHub releases and the newest release is marked **Latest**.

## Development Requirements

These tools are needed only to build, test, or deploy from source:

- WSL2 for building/deploying from this repo on Windows.
- mise for development tasks: https://mise.jdx.dev/
- Rust 1.96+ with the Windows GNU target:

  ```bash
  rustup target add x86_64-pc-windows-gnu
  ```

- LuaJIT, LuaRocks, and Stylua are required for Lua validation.
- MinGW-w64 and Wine are required for Windows DLL builds, DLL validation, and
  benchmarks.
- `file`, `sha256sum`, `rsync`, `zip`, and `unzip` support validation,
  deployment, and packaging; `gh` is required only to publish a release.
- WSL interoperability (`wslpath`, `cmd.exe`, and `powershell.exe`) is required
  for the native-Windows regression and CUDA benchmark gates.
- CUDA Toolkit 12.4 with `nvcc` and GCC 12 are required to build the release's
  experimental GPU module from source. The default fat binary contains native
  kernels for compute capabilities 5.0, 5.2, 6.0, 6.1, 7.0, 7.5, 8.0, 8.6,
  8.9, and 9.0, plus an 8.9 PTX fallback for newer NVIDIA GPUs. Advanced builds
  may override `BRAINSTORM_CUDA_ARCHES`, `BRAINSTORM_CUDA_PTX_ARCH`,
  `CUDAHOSTCXX`, or `NVCC`; set `BRAINSTORM_SKIP_CUDA_BUILD=1` only when
  intentionally validating the CPU-only fallback. End users do not need the
  CUDA Toolkit. A missing or incompatible NVIDIA driver or device falls back
  to Rust CPU. CUDA initialization and runtime failures stay on CPU for the
  rest of that game process; restart Balatro to retry the GPU.

## Build & Deploy (from source)

`mise.toml` is the development interface. Run `mise trust` once per checkout,
then install/check the toolchain:

```bash
mise run setup
```

Build and deploy with:

```bash
mise run build
mise run deploy
```

If auto-detection cannot find your Balatro mods folder, set `TARGET` to the
full `.../Balatro/Mods/Brainstorm` path.

`mise run build` builds the Rust native DLL and writes
`target/rust/Hex.dll`.

`mise run lint` runs Lua formatting, LuaJIT bytecode syntax checks, luacheck,
rustfmt, clippy, and private-item rustdoc checks. `mise run check-rust` adds
unit tests, DLL export/import validation, and hit/composite benchmark smokes.
`mise run check` runs Lua lint, the mocked module/frame/status lifecycle smoke,
and the Rust validation gate.

Strict user-facing regression check against a frozen current-ABI DLL:

```bash
BENCH_BASELINE_DLL=/path/to/frozen/Hex.dll mise run bench-current-compare
```

Historical full-suite benchmark report:

```bash
mise run bench-full
```

Both use the same `threads=0` path as Lua auto-reroll. The current/current
command runs natively on Windows, freezes and hashes its artifacts, requires
exact result/scanned equality, hard-gates p50/p95/mean latency, and reports p99.
Confirm a p99 signal with 501 repeats and eight cycles, which supplies enough
samples to enable its hard gate; `Hex/BENCH.md` gives the exact command.
Original-DLL ratios are informational except for the one-candidate baseline
because its seed order differs. `BENCH_EXECUTOR=wine` is a portability
diagnostic only.

DLL UX-fixture benchmark report using UI-reachable cases and Lua-style
`threads=0`:

```bash
mise run bench-ux
```

For true in-game Lua timing, profile `Brainstorm.auto_reroll()` inside Balatro.
For native Linux-side Rust profiling without the Windows DLL ABI, use
`brainstorm_bench` as described in `Hex/BENCH.md`.

See `Hex/BENCH.md` for benchmark workflows.

The closest benchmark to the in-game experimental CUDA path runs the Windows
DLL and driver natively from WSL:

```bash
mise run bench-cuda-long-windows
```

## Versioning & Release

The source of truth for the mod version is `[manifest].version` in
`lovely.toml`. `steamodded_compat.lua`, the Hex crate metadata, and the
current-release line at the top of this README carry the same exact
`MAJOR.MINOR.PATCH` version and are checked by `mise run check-version`.

Use this when bumping versions:

```bash
VERSION=<VERSION> mise run bump-version
```

`mise run release` runs validation, builds `target/rust/Hex.dll`, stages a
`Brainstorm/` install folder, and creates
`release/Brainstorm_OVERHAULED_v<VERSION>.zip`.

Commit the synchronized version bump and push `master`, then run
`mise run publish-release`. It locally rebuilds and validates the package,
creates and pushes the exact annotated `v<VERSION>` tag without force, uploads
the zip and basename-only checksums directly to a draft GitHub release,
downloads and verifies those assets, and publishes the release as **Latest**.
Published tags and releases are never overwritten; an interrupted unpublished
draft is discarded and recreated after its tag provenance is revalidated.
GitHub Actions intentionally has no release workflow; the local publish task
uses no runner minutes.

## Documentation

- `AGENTS.md`: contributor and agent-facing project rules.
- `Brainstorm_Audit.md`: source-verified future-work and multi-ante design
  audit.
- `BalatroSource_Guide.md`: verified Balatro source mechanics relevant to
  search parity and future mod work.
- `Hex/BENCH.md`: benchmark harness, gates, and fixture groups.
- `NOTICE.md`: project, rewrite, Hex, Immolate, and third-party attribution
  notices.

## Release Contents

The generated release zip contains exactly this install-ready payload. For a
local source build, run `mise run stage-payload` and use
`target/package/Brainstorm/`; do not copy the source manifest directly because
packaging also writes `VERSION` and disables Lovely's development-only Lua
dumping.

```
Brainstorm/
├── Brainstorm.lua
├── Hex.dll                # Native DLL
├── LICENSE
├── NOTICE.md
├── UI.lua
├── VERSION
├── lovely.toml
└── steamodded_compat.lua
```

User settings are generated at runtime in Balatro's Love save directory and are
not part of the release payload.

## Usage

- Open settings: Ctrl+T. Toggle auto-reroll: Ctrl+A. Manual reroll: Ctrl+R.
- Save/load state: Z/X + 1-5.
- Configure filters: dual tags, voucher, pack (two shop slots), Joker
  (searchable list + location), one Soul, Observatory, Perkeo.
- Impossible first-shop Joker targets are hidden from the Joker selector, and
  impossible native filter combinations return no match immediately.
- Configure Erratic Deck filters when searching for opening hands by face-card
  count, no faces, or suit concentration.
- Use "Enable Brainstorm OVERHAULED" to disable runtime actions without
  losing settings.
- Turn on `AR: Use CUDA (Experimental)` to opt into the experimental GPU path;
  leave it off to force Rust CPU. Searches outside the supported GPU surface
  fall back automatically.
- Use "Reset All" in the Brainstorm OVERHAULED tab to restore filter and
  Erratic Deck settings to defaults.

## Troubleshooting

- Missing DLL or wrong build: rerun `mise run build` and
  `mise run deploy`. If auto-detection fails, set `TARGET` to the full
  `.../Balatro/Mods/Brainstorm` path.
- Lua and `Hex.dll` must come from the same release; older DLLs do not
  provide the experimental CUDA control ABI used by version 4.
