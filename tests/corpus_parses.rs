//! Guards that every file tsv is benched on parses cleanly.
//!
//! `bench_parser::tsv::parse` ignores parse errors (mirroring `oxc::parse`), so
//! a corpus file tsv silently *rejects* would still be timed — and a parser that
//! bails early on syntax it can't handle looks artificially fast. This test
//! asserts the tsv corpus (`.ts`/`.js`, never JSX/TSX) parses with no error, so
//! the benchmark stays apples-to-apples. See CLAUDE.md.

use std::fs;

use bumpalo::Bump;

/// The files `TsvBencher` runs on — real TypeScript only (`.ts`).
///
/// tsv is scoped to `.ts` (not the `.js`/`.tsx` corpus): its strict TS parser
/// has no JSX grammar (`cal.com.tsx`) and rejects identifiers that collide with
/// TS contextual keywords, which the `.js` corpus uses — `typescript.js` names a
/// parameter `readonly` (`function createArrayType(elementType, readonly)`),
/// which tsv refuses. Either would make tsv error mid-parse and look
/// artificially fast under the bench's error-tolerant timing.
const TSV_CORPUS: &[&str] = &["files/parser.ts"];

#[test]
fn tsv_parses_its_corpus_cleanly() {
    for &file in TSV_CORPUS {
        let source = fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
        let arena = Bump::new();
        let result = tsv_ts::parse(&source, &arena);
        assert!(result.is_ok(), "tsv failed to parse {file}: {:?}", result.err());
    }
}
