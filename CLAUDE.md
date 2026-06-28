# CLAUDE.md

Working map for this repo, including the tsv integration this fork adds over
upstream ([oxc-project/bench-javascript-parser-written-in-rust](https://github.com/oxc-project/bench-javascript-parser-written-in-rust)).

## What this is

A **Rust / criterion** microbenchmark comparing JS/TS **parsers** on raw parse
throughput (CPU instructions via CodSpeed locally/CI, plus peak RSS via
`memory.sh`). Parsers compared:

- **oxc** (`oxc::parser`)
- **swc** (`swc_ecma_parser`)
- **biome** — present but **commented out** (deps, `src/biome.rs`, and the
  bencher are all `//`-disabled upstream; the published README tables still show
  biome numbers from when it was enabled)
- **tsv** (`tsv_ts::parse`) — native Rust, TypeScript-only. This is the fork's
  addition. It runs on the `.ts` corpus only (see [The tsv integration](#the-tsv-integration)).

Each parser is driven two ways: the **criterion bench** (`benches/parser.rs`,
three measurements per file) and a **standalone bin** (`src/{oxc,swc,tsv}.rs`,
one parse-and-exit, driven by `memory.sh` for peak RSS).

## Working in this repo

- **Don't modify git.** Make your edits and stop — leave committing, pushing,
  branching, tags, and any history changes to the maintainer. (This is a public
  fork; treat its git state as not yours to touch.)
- **The tsv parser needs a sibling `../tsv` checkout.** tsv is wired in as a
  `../tsv/crates/tsv_ts` path dependency, so `cargo build`/`bench`/`test` fail
  unless the public [tsv repo](https://github.com/fuzdev/tsv) sits next to this
  one: `git clone https://github.com/fuzdev/tsv.git ../tsv`. This is a **local
  requirement** — tsv benchmarking is run locally, not in CI (see [CI](#ci)).
- A complementary fork, `../oxc-bench-formatter`, benchmarks the whole **tsv
  CLI** (process spawn + multi-file parallel formatting + RSS) against
  prettier/biome/oxfmt. This repo is the in-process **parser** half — see that
  repo's `CLAUDE.md` for the tsv CLI, directory-discovery, and binary-setup
  details that don't apply here.

## Dependencies & upstream

This is a fork; its `main` tracks upstream
([oxc-project/bench-javascript-parser-written-in-rust](https://github.com/oxc-project/bench-javascript-parser-written-in-rust)),
and the tsv integration lives on the `tsv` branch. The three parsers reach the
build two different ways:

- **oxc, swc** — published **crates.io** dependencies (`oxc`,
  `swc_ecma_parser`, `swc_ecma_ast`, `swc_common`). Cargo downloads the resolved
  versions (frozen in `Cargo.lock`) and compiles them from source — no checkout
  needed, reproducible anywhere with registry access. Version bumps come from
  **upstream** (renovate PRs that land on `main`), so they stay current; to bench
  a different version locally, edit the version in `Cargo.toml` and run
  `cargo update`.
- **tsv** — the fork's own addition, a `../tsv/crates/tsv_ts` **path** dependency
  (plus `bumpalo`). It reads your local `../tsv` working tree directly, so it is
  *not* pinned by `Cargo.lock` and *not* renovate-managed — it tracks whatever you
  have checked out there. To bench newer tsv, just update `../tsv`.

All three compile from source into the bench binary; the only difference is where
the source comes from (registry vs local path) and whether it's version-pinned.

## Layout

```
.
├── Cargo.toml              # bench + bins + deps (tsv_ts path dep, oxc, swc)
├── benches/parser.rs       # the criterion benchmark (TheBencher trait, 3 measurements)
├── src/lib.rs              # parse() per parser: oxc / swc / tsv (biome commented out)
├── src/{oxc,swc,tsv}.rs    # standalone parse-and-exit bins for memory.sh
├── tests/corpus_parses.rs  # guards that tsv's corpus parses cleanly
├── files/                  # corpus, committed directly (not fetched)
│   ├── cal.com.tsx         #   ~1MB JSX  — oxc, swc (tsv can't: no JSX grammar)
│   ├── typescript.js       #   ~8MB JS   — oxc, swc (tsv can't: `readonly` as a param ident)
│   └── parser.ts           #   ~540KB TS — oxc, swc, tsv  (added by this fork)
├── memory.sh               # peak-RSS measurement via GNU time
├── table.mjs               # scrapes target/criterion/** into a markdown table (`pnpm run table`)
└── .github/workflows/      # benchmark.yml (CodSpeed), security.yml
```

## The benchmark — `benches/parser.rs`

`trait TheBencher` defines `parse(path, source) -> ParseOutput` and a `bench()`
that runs three criterion measurements per file:

- **single-thread** — `b.iter(parse)`; drop time is included.
- **no-drop** — `b.iter_with_large_drop(parse)`; defers dropping the returned
  output so AST teardown is excluded.
- **parallel** — `parse` run once per physical core via rayon, surfacing global
  resource contention.

The trait also has `fn supports(path) -> bool { true }`; `parser_benchmark`
guards each bencher with it so a parser is skipped for files it can't handle
(only `TsvBencher` overrides it). Benchers: `OxcBencher`, `SwcBencher`,
`TsvBencher` (`BiomeBencher` is commented out).

`ParseOutput` is each parser's arena/AST owner so the **no-drop** variant has
something large to defer dropping — oxc returns its `Allocator`, swc its
`Module`, tsv its `bumpalo::Bump`.

## The tsv integration

[tsv](https://github.com/fuzdev/tsv)'s TypeScript parser is benched as a third
parser. Specifics:

- **Library call:** `tsv_ts::parse(source, &arena) -> Result<Program>`, where
  `arena: &bumpalo::Bump`. `bench_parser::tsv::parse(source)` allocates a fresh
  `Bump`, parses into it, drops the borrowed `Program`, and returns the `Bump` —
  the direct analog of `oxc::parse` returning its `Allocator`, so the no-drop
  variant measures arena teardown. Parse errors are ignored (as `oxc::parse`
  does); cleanliness is enforced separately by `tests/corpus_parses.rs`.
- **Dependency:** `tsv_ts = { path = "../tsv/crates/tsv_ts", default-features =
  false }` (the `convert` JSON layer is off — parse only) plus `bumpalo`. tsv
  builds only the crates it needs (`tsv_ts` + `tsv_lang` + a few small deps), not
  the whole tsv workspace.
- **`.ts`-only scoping (load-bearing):** tsv's parsers are TypeScript/Svelte/CSS
  and it is benched on **real TypeScript** only. Two reasons it is *not* run on
  the existing corpus:
  - `cal.com.tsx` — tsv has **no JSX/TSX grammar** at all.
  - `typescript.js` — plain JS that names a parameter `readonly`
    (`function createArrayType(elementType, readonly)`, line 58104); tsv's strict
    TS parser treats `readonly` as a reserved modifier there and **rejects** it.
  Either would make tsv *error mid-parse*. That matters because the bench is
  error-tolerant (`parse` ignores the `Result`), so a parser that bails early on
  syntax it can't handle is still timed and looks **artificially fast**. So
  `parser.ts` (the TypeScript compiler's `src/compiler/parser.ts`, v5.9.2) was
  added as a clean, real-TS file all three parsers accept. `TsvBencher::supports`
  encodes the rule: extension `== "ts"`.
- **Keeping it honest:** `tests/corpus_parses.rs` asserts every file in tsv's
  corpus parses with no error (`cargo test --test corpus_parses`). If you add a
  `.ts` file tsv can't parse, that test fails loudly instead of silently skewing
  the numbers. Run it before trusting a new tsv result.
- **The `.bak` heritage:** `files/parser.ts` is the pristine v5.9.2 download (the
  same file `../oxc-bench-formatter` uses for its single-file scenario).

## Running

```bash
git clone https://github.com/fuzdev/tsv.git ../tsv   # one-time: the path dep
cargo bench                                          # all parsers, all files
pnpm i && pnpm run table                             # scrape target/criterion → markdown table
./memory.sh                                          # peak RSS (needs GNU time)
```

- **CPU:** `cargo bench` writes criterion estimates under `target/criterion/`;
  `table.mjs` (`pnpm run table`) renders them. tsv appears only in the
  `parser.ts` group.
- **Memory:** `memory.sh` builds the release bins and runs each under **GNU
  time** (`gtime`, or a GNU `/usr/bin/time` — not the BSD/macOS built-in;
  `apt install time` / `brew install gnu-time`), averaging peak RSS over 10 runs.
  It scopes parsers per file the same way the CPU bench does (tsv on `parser.ts`
  only). It was rewritten from upstream's macOS-only `/usr/bin/time -al` script
  to be portable; it also drops biome (whose bin isn't built).
- **Filter to one parser/file:** `cargo bench --bench parser -- tsv` (criterion
  treats the arg as a regex over IDs like `parser.ts/tsv/single-thread`).
- **Toolchain:** pinned by `rust-toolchain.toml` to Rust 1.96.0 (minimal
  profile), so cargo uses that version regardless of your system default.

## CI

**tsv is benchmarked locally only — not in CI, by choice.**
`.github/workflows/benchmark.yml` is left as upstream's CodSpeed workflow
(`cargo codspeed build/run` on push to `main` and on PRs); it is **not** wired up
to build or measure tsv, and there is no CodSpeed account/token/OIDC to set up
here. Local `cargo bench` (criterion) is fully self-sufficient and needs none of
that — see [Running](#running). `security.yml` is the upstream security scan,
unchanged.

Caveat for anyone tempted to run that workflow with the tsv integration present:
the tsv parser is a `../tsv` **path** dependency, so `cargo codspeed build` would
**fail to resolve** in a runner that has no sibling `../tsv` checkout. That's the
reason tsv stays a local concern. The workflow only triggers on push to `main`
and on PRs, so keep the tsv integration off `main` (or expect that build to fail)
— or, if you ever do want tsv in CI, add a step that clones `fuzdev/tsv` to
`"$GITHUB_WORKSPACE/../tsv"` before the build and connect a CodSpeed project.

## Adding a parser

1. Add the dep(s) to `Cargo.toml` and a `parse()` to `src/lib.rs`.
2. Add a `<Name>Bencher` to `benches/parser.rs` (set `ParseOutput` to the
   parser's arena/AST owner so no-drop is meaningful) and call its `bench()` in
   `parser_benchmark`. If it can't handle every file, override `supports()` and
   keep the call guarded by it.
3. Add a `src/<name>.rs` bin (copy `src/oxc.rs`) and a `[[bin]]` entry, then add
   the parser to `memory.sh`'s per-file `APPS` lists.
4. If it's a native parser that may reject some corpus syntax, add it to
   `tests/corpus_parses.rs` so a silent partial-parse can't masquerade as speed.
```
