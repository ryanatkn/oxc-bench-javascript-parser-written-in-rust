pub mod oxc {
    use std::path::Path;

    use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

    pub fn parse(path: &Path, source: &str) -> Allocator {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        _ = Parser::new(&allocator, source, source_type).parse();
        allocator
    }
}

pub mod swc {
    use std::path::Path;

    use swc_common::BytePos;
    use swc_ecma_ast::Module;
    use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax};

    pub fn parse(path: &Path, source: &str) -> Module {
        let syntax = match path.extension().unwrap().to_str().unwrap() {
            "js" => Syntax::Es(EsSyntax::default()),
            "ts" => Syntax::Typescript(TsSyntax::default()),
            "tsx" => Syntax::Typescript(TsSyntax {
                tsx: true,
                ..TsSyntax::default()
            }),
            _ => panic!("need to define syntax for swc"),
        };
        let input = StringInput::new(source, BytePos(0), BytePos(source.len() as u32));
        Parser::new(syntax, input, None).parse_module().unwrap()
    }
}

pub mod tsv {
    use bumpalo::Bump;

    /// Parse `source` with tsv's TypeScript parser. tsv is always a strict
    /// TypeScript parser (module goal) with no JSX grammar, so the file path is
    /// irrelevant — the caller scopes the corpus to real `.ts` (the bench skips
    /// JSX and the `.js` files, which use TS contextual keywords as identifiers
    /// that tsv rejects; see `TsvBencher::supports`).
    ///
    /// The AST is allocated into a per-call `bumpalo::Bump`, which is returned
    /// so the `no-drop` bench variant measures arena teardown — the same shape
    /// as `oxc::parse` returning its `Allocator`. The borrowed `Program` (and
    /// its string interner, which lives outside the arena) is dropped before
    /// the arena is returned, so only the arena's drop is deferred under
    /// `iter_with_large_drop`. Parse errors are ignored, mirroring `oxc::parse`;
    /// the corpus is expected to parse cleanly (see CLAUDE.md).
    pub fn parse(source: &str) -> Bump {
        let arena = Bump::new();
        let _ = tsv_ts::parse(source, &arena);
        arena
    }
}

// pub mod biome {
// use std::path::Path;

// use biome_js_parser::{JsParserOptions, Parse};
// use biome_js_syntax::{AnyJsRoot, JsFileSource};

// pub fn parse(path: &Path, source: &str) -> Parse<AnyJsRoot> {
// let options = JsParserOptions::default();
// let source_type = JsFileSource::try_from(path).unwrap();
// biome_js_parser::parse(source, source_type, options)
// }
// }
