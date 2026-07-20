# Brainstorm OVERHAULED Audit

Audit date: 2026-07-18

## Bottom line

Two small improvements are worth considering before the search interface grows:
make mod-root discovery independent of the installation directory name, and add
an explicit native ABI/model-version handshake. One CPU optimization question
worth a narrow prototype is fixed-width scalar interleaving of independent RNG
recurrences. The larger design opportunity is to make multi-ante search
simulate one chronological route rather than a set of independent ante
predicates.

All mechanics statements below were checked against the local
`BalatroSource/` tree. `BalatroSource/` and the source-faithful Rust `Instance`
model remain the correctness authorities.

## Future work before expanding search

### Stop assuming the install directory is literally `Mods/Brainstorm`

`Brainstorm.init()` currently assigns
`Brainstorm.PATH = lovely.mod_dir .. "/Brainstorm"`. Deriving the running mod's
directory from loader context or a unique shipped marker would support renamed
and nested installations without scanning the entire Mods directory and
risking selection of an unintended duplicate installation.

Cover renamed and nested layouts in `tests/lua_lifecycle_smoke.lua`. Once this
is reliable, the README would no longer need to make the exact folder name a
runtime requirement.

### Add a native ABI/model-version handshake

Lua currently checks that native exports exist, but a stale DLL can expose the
same function name with an incompatible FFI signature or search model. Before
multi-ante parameters enlarge `brainstorm_search`, add a minimal
`hex_abi_version()` export or introduce a versioned search-function name.
Mismatches should fail closed with a clear error.

### Benchmark fixed-width scalar RNG interleaving

The experiment is whether advancing a small fixed batch of independent
candidates in lockstep lets the processor overlap the serial `f64` pseudohash
and first-stream recurrences. Keep the existing seed traversal, adjacent-seed
cache, compiled predicates, and parallel block scheduler, and evaluate
candidate lanes in seed order.

Treat this as a benchmark experiment until native-Windows measurements improve
representative current-DLL cases. Require bit-exact scalar parity across every
lane, partial batches, carry and seed-space wrap boundaries, plus identical
result and scanned-count semantics in single-thread and parallel searches. Do
not use `fast-math`, host-specific release flags, or transformations that alter
Lua-number rounding. Consider explicit SIMD only if profiling leaves a uniform
hot recurrence, with runtime dispatch and the scalar implementation retained as
the portable correctness path.

## Multi-ante search design

### Simulate a chronological route

A reproducible result needs more than a seed. It needs the ante and blind,
which blinds are skipped, whether the source is a shop or an immediate tag
reward, which packs are opened, and any required voucher purchases. Small,
Big, after-Boss, Charm, and Ethereal sources are distinct physical locations
even when some of their RNG keys use the same ante number.

Local source confirms why this matters:

- `functions/button_callbacks.lua` (`G.FUNCS.skip_blind`) grants the tag and
  advances the blind without playing a round or opening a shop.
- `functions/state_events.lua` advances the ante when the Boss round ends, so
  the shop after that Boss uses the next ante's keyed streams.
- `functions/common_events.lua` (`get_pack`) forces the run's first generated
  shop pack to a normal Buffoon without consuming `shop_pack<ante>`.
- `tag.lua` (`Tag:apply_to_run`) opens Charm and Ethereal reward packs
  immediately. Those `Card:open()` calls consume current-ante card and Soul
  streams before a later shop pack.

The native result should eventually return structured location and route data
with the seed. Human-facing ante/blind/source information should be distinct
from the RNG ante and stream advance that produced the match.

### Compose tag effects explicitly

Tag effects can change the route and the streams consumed by later filters:

- Uncommon and Rare Tags use `store_joker_create` and replace a future shop
  Joker draw. Sources: `functions/UI_definitions.lua`
  (`create_card_for_shop`); `tag.lua`.
- Voucher Tag adds another voucher draw. Source: `tag.lua` (`voucher_add`).
- Double Tag can duplicate a later tag reward. Source: `tag.lua` (`tag_add`).
- Charm and Ethereal open packs before the next reachable shop. Source:
  `tag.lua` (`new_blind_choice`).

Combined tag, Joker, pack, Soul, and voucher filters therefore need one shared
route simulation. The first implementation should either support each
composition or reject unsupported combinations statically and explain why.

### Treat hidden special cards as live-card state

Soul and Black Hole availability is controlled by live-card state:

- `functions/common_events.lua` (`create_card`) gates Soul and Black Hole
  against `G.GAME.used_jokers` unless Showman permits duplicates.
- `card.lua` (`Card:set_ability`) marks a generated pack card as used while it
  exists, preventing a duplicate later in the same live pack.
- `card.lua` (`Card:remove`) and `cardarea.lua` (`CardArea:remove`) clear that
  gate once no matching live card remains, allowing a later pack to roll it.
- A later Soul advances the global bare `Joker4` stream. A legendary retained
  in the Joker area remains unavailable and can trigger `_resampleN`.

Any route involving another Soul must state whether the earlier Soul was used
and whether the resulting legendary remains owned. Tests must also cover the
same-card Spectral Soul/Black Hole overwrite behavior and their asymmetric ban
handling recorded in `BalatroSource_Guide.md`.

### Make Omen Globe and voucher searches purchase-aware

`card.lua` (`Card:open`) checks the global `omen_globe` stream for every Arcana
pack card while `v_omen_globe` is in `used_vouchers`. Buying Omen Globe before
opening a pack in the same shop affects that pack because its contents are
generated when opened, not when the booster appears.

Voucher state is also route-dependent:

- `functions/common_events.lua` (`get_current_pool`) excludes redeemed
  vouchers and locks upgrades behind their prerequisites.
- `card.lua` (`Card:redeem`) changes `used_vouchers` before subsequently
  opened packs.
- Hieroglyph and Petroglyph can call `ease_ante(-1)`, revisiting ante-keyed
  streams at later advances rather than moving monotonically through antes.

Choose one explicit initial contract: either buy nothing before the target or
accept a purchase plan as input. RNG availability and route feasibility should
remain distinct from affordability.

### Simulate each stream once for multiple targets

When several targets inspect the same shop or pack sequence, generate that
sequence once and evaluate all targets against the recorded events. Walking a
mutable stream independently for every target risks checking later advances
for the second and subsequent targets. Keep each result's first matching
physical location so the displayed route is reproducible.

### Keep vanilla seed spaces distinct

Local source establishes three relevant cases:

- Natural rerolls use fixed eight-character seeds over 34 symbols, excluding
  both `0` and `O`.
- Vanilla settable seeds use lengths 1-8 over 35 symbols, including `O` but
  excluding `0`; typed `0` is normalized to `O`.
- Empty seeded input is accepted by vanilla and is included in the native
  settable-seed space.

Sources: `misc_functions.lua` (`random_string`),
`functions/UI_definitions.lua` (`create_text_input`), and
`functions/button_callbacks.lua` (`text_input_key`, `paste_seed`). Native
`SEED_SPACE` represents `2,318,107,019,761` settable states including the empty
seed (`Hex/src/seed.rs`).

## Additional exploration avenues

### Expand the typed search model

- Edition-aware targets could constrain ordinary or Legendary Jokers by
  edition. Define the exact RNG timing, eligibility, locks, and resampling from
  `BalatroSource/` before extending the request, then require source-oracle
  coverage and either CPU/GPU parity for a supported CUDA shape or
  unsupported-path fallback coverage.
- General pack-content targets could search for selected Tarot, Planet,
  Spectral, playing-card, or Joker outcomes at supported physical locations.
  Generate each pack once, return its exact route and location, and reject
  impossible targets or compositions statically.
- Compound scenario presets could combine cards, editions, tags, vouchers,
  ante ranges, skips, purchases, and deterministic resource requirements.
  Presets should compile into the shared chronological route model rather than
  add one-off native filter modes. Availability must remain distinct from
  affordability and route feasibility.

### Improve search and result UX

- Replace long cyclic option lists with a searchable, paged,
  controller-friendly card selector when the target catalog warrants it.
  Build it from the model-supported catalog, preserve explicit `Any` and `None`
  choices, and keep input callbacks local to Brainstorm's UI.
- Keep an optional result history containing the seed, typed request, route,
  model version, and verification status. Store it as user data in the Love
  save directory; do not fabricate game save states or ship generated history.
- Localize display text through stable internal keys with an English fallback.
  Never pass translated labels through the FFI or use them as persistent
  identifiers.
- Expose remappable modifiers and hotkeys, including left/right modifier
  alternatives. Preserve current defaults, validate persisted values against
  the current schema, and reset obsolete values instead of adding migrations.
- If the filter surface grows, group controls by route stage and source and use
  responsive columns or paging instead of extending one long cycling form.
- A job-style native API with progress and cooperative cancellation is worth
  exploring if in-game profiling demonstrates visible synchronous hitches. It
  must preserve earliest-match semantics, result lifetime, and clean DLL
  teardown; it is a responsiveness feature, not a throughput shortcut.

### Define compatibility and portability explicitly

- An opt-in mod-compatibility mode could consume an identified runtime catalog
  plus explicit eligibility adapters. A live pool snapshot alone is not a
  mechanics model: mods may alter generation, locks, rarity, editions, or
  resampling. Keep vanilla mode unchanged and label compatibility results with
  the exact catalog and model identity.
- Cross-platform native CPU builds could use OS- and architecture-aware
  loading while retaining the same Rust model and source-oracle suite. Preserve
  a one-zip installation experience, keep optional CUDA transparent, and
  require identical result and scanned-count semantics on every supported
  platform.

### Support offline and shareable workflows

- Reusable exhaustive or partially filtered seed pools may suit repeated rare
  and multi-ante searches better than rescanning. Keep this as a separate
  offline workflow sharing the Rust oracle. Formats must be checksummed and
  versioned by seed space, model, catalog, and filter semantics; support
  resumable non-overlapping ranges; preserve route metadata; and reverify
  loaded hits against the current model before presenting them.
- Exportable typed requests and verified result bundles could support sharing
  and distributed work without coupling the in-game UI to pool construction.

### Profile further GPU work

- Expand experimental CUDA coverage only for high-value filter shapes backed
  by representative native-Windows measurements. Unsupported shapes must keep
  transparent CPU fallback and exact result and scanned-count parity.
- Overlapped or double-buffered GPU windows are worth testing only if profiling
  identifies launch or readback idle time. Resolve windows in order so
  speculative later work cannot change earliest-match semantics.

## Regression cases to add with multi-ante search

Use the local `Instance` model as the oracle:

1. Exact Small, Big, and after-Boss shop coordinates.
2. One skipped blind, both ante-1 blinds skipped, and forced-Buffoon migration
   to the first shop that opens.
3. Charm and Ethereal reward packs consuming streams before shop packs.
4. Uncommon, Rare, Voucher, and Double Tag effects in combined filters.
5. Omen Globe absent, already owned, and bought before a same-shop Arcana pack.
6. Spectral Soul-then-Black-Hole overwrite, including banned Black Hole.
7. Pack-card lifetime, Showman, a later Soul, and legendary resampling.
8. Voucher prerequisites, purchases, exclusions, and ante decreases.
9. Multiple Joker targets evaluated against one simulated sequence.
10. Returned location and route data reproducing the match.
11. Single-thread and parallel searches returning the earliest matching seed.
12. Natural and settable seed-space boundary and wraparound cases.

A small fixture captured from Balatro's shipped Windows LuaJIT could provide
additional cross-platform evidence if RNG code changes. Runtime calibration is
not currently necessary because the Rust CPU engine is the native correctness
oracle and experimental CUDA must match its result and scanned-count semantics.

## Implementation patterns to avoid

- A second native search engine or a duplicated Lua mechanics oracle would add
  another implementation to keep synchronized. Extend the Rust `Instance`
  model and its optimized predicates instead.
- Do not add hard-coded scenario switches or mutually exclusive custom filter
  modes. Prefer composable typed requests and shared chronological route
  simulation; a generic text query language is still premature.
- Do not post-filter only the first native hit. If a candidate fails a later
  clause, searching must continue through the original ordered window.
- Do not return the first wall-clock worker result, hard-code thread counts, or
  replace the earliest-offset scheduler with a competing multicore engine.
- Do not use an external process or file IPC for in-game search if it duplicates
  mechanics or weakens teardown, packaging, cancellation, or earliest-result
  guarantees. Offline tools may share the Rust oracle without becoming a
  second mechanics implementation.
- Do not use `fast-math`, altered RNG arithmetic, host-specific release tuning,
  or SIMD without runtime dispatch and a bit-exact scalar fallback.
- Do not treat a dynamic card pool as sufficient mod compatibility. Generation
  rules and stateful eligibility need an explicit adapter and model identity.
- Avoid global card or input hooks for a local selector, and never use localized
  strings as config, ABI, or catalog identifiers.
- Fabricated or marker-based save slots for unplayed seeds are unnecessary for
  multi-ante correctness and should not be coupled to result history.
- Do not extend searches to inputs Balatro cannot accept. Preserve the vanilla
  seed-space definitions above.
- Do not re-enable runtime logging, commit generated search data, or bundle a
  helper executable as part of ordinary in-game search.
- Keep the existing immutable, version-checked, checksum-verified release
  process rather than adding another packaging or updater path.

## Verification record

Mechanics were traced through these local source entry points:

- `functions/button_callbacks.lua`: `G.FUNCS.skip_blind`, seed input handling
- `functions/state_events.lua`: Boss completion and `ease_ante(1)`
- `functions/common_events.lua`: `get_current_pool`, `get_pack`, `create_card`,
  `get_next_tag_key`
- `functions/UI_definitions.lua`: `create_card_for_shop`, seed text input
- `tag.lua`: `Tag:apply_to_run`
- `card.lua`: `Card:open`, `Card:redeem`, `Card:apply_to_run`,
  `Card:set_ability`, `Card:remove`, `Card:use_consumeable`
- `cardarea.lua`: `CardArea:remove`
- `misc_functions.lua`: `random_string`

Validation completed during the audit:

- `cargo test --manifest-path Hex/Cargo.toml seed_`: 9 tests passed
  across the library and DLL harness, including wraparound and parallel
  earliest-result coverage.
- `cargo test --manifest-path Hex/Cargo.toml source_oracle`: 5
  source-oracle suites passed.
- `mise run lint-lua`: 0 warnings and 0 errors; version check passed.
- `git diff --check`: passed.

No runtime code was changed. `BalatroSource_Guide.md` was updated only with
mechanics independently verified in `BalatroSource/`.
