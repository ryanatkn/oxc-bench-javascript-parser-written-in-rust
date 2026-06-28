use std::path::Path;

use criterion::{measurement::WallTime, *};
use rayon::prelude::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

trait TheBencher {
    type ParseOutput;

    const ID: &'static str;

    fn parse(filename: &Path, source: &str) -> Self::ParseOutput;

    /// Whether this parser can handle the given file. Defaults to `true`;
    /// `TsvBencher` overrides it to skip JSX/TSX, which tsv has no grammar for.
    fn supports(_path: &Path) -> bool {
        true
    }

    fn bench(g: &mut BenchmarkGroup<'_, WallTime>, path: &Path, source: &str) {
        let cpus = num_cpus::get_physical();
        let id = BenchmarkId::new(Self::ID, "single-thread");
        g.bench_with_input(id, &source, |b, source| {
            b.iter(|| Self::parse(path, source))
        });

        let id = BenchmarkId::new(Self::ID, "no-drop");
        g.bench_with_input(id, &source, |b, source| {
            b.iter_with_large_drop(|| Self::parse(path, source))
        });

        let id = BenchmarkId::new(Self::ID, "parallel");
        g.bench_with_input(id, &source, |b, source| {
            b.iter(|| {
                (0..cpus).into_par_iter().for_each(|_| {
                    Self::parse(path, source);
                });
            })
        });
    }
}

struct OxcBencher;

impl TheBencher for OxcBencher {
    type ParseOutput = oxc::allocator::Allocator;

    const ID: &'static str = "oxc";

    fn parse(path: &Path, source: &str) -> Self::ParseOutput {
        bench_parser::oxc::parse(path, source)
    }
}

struct SwcBencher;

impl TheBencher for SwcBencher {
    type ParseOutput = swc_ecma_ast::Module;

    const ID: &'static str = "swc";

    fn parse(path: &Path, source: &str) -> Self::ParseOutput {
        bench_parser::swc::parse(path, source)
    }
}

// struct BiomeBencher;

// impl TheBencher for BiomeBencher {
// type ParseOutput = biome_js_parser::Parse<biome_js_syntax::AnyJsRoot>;

// const ID: &'static str = "biome";

// fn parse(path: &Path, source: &str) -> Self::ParseOutput {
// bench_parser::biome::parse(path, source)
// }
// }

struct TsvBencher;

impl TheBencher for TsvBencher {
    type ParseOutput = bumpalo::Bump;

    const ID: &'static str = "tsv";

    fn parse(_path: &Path, source: &str) -> Self::ParseOutput {
        bench_parser::tsv::parse(source)
    }

    /// tsv is benched on real TypeScript (`.ts`) only. It has no JSX grammar
    /// (`cal.com.tsx`), and its strict TS parser rejects identifiers that
    /// collide with TS contextual keywords, which the `.js` corpus uses
    /// (`typescript.js` names a parameter `readonly`) — so `.js` is out too.
    /// Either would make tsv error mid-parse and look artificially fast.
    fn supports(path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str()) == Some("ts")
    }
}

fn parser_benchmark(c: &mut Criterion) {
    let filenames = ["typescript.js", "cal.com.tsx", "parser.ts"];
    for filename in filenames {
        let path = Path::new("files").join(filename);
        let source = std::fs::read_to_string(&path).unwrap();
        let mut g = c.benchmark_group(filename);
        OxcBencher::bench(&mut g, &path, &source);
        SwcBencher::bench(&mut g, &path, &source);
        // BiomeBencher::bench(&mut g, &path, &source);
        if TsvBencher::supports(&path) {
            TsvBencher::bench(&mut g, &path, &source);
        }
        g.finish();
    }
}

criterion_group!(parser, parser_benchmark);
criterion_main!(parser);
