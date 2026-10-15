use std::sync::Arc;

use deno_ast::apply_text_changes;
use deno_ast::lex;
use deno_ast::parse_module;
use deno_ast::strip_bom;
use deno_ast::MediaType;
use deno_ast::ModuleSpecifier;
use deno_ast::ParseParams;
use deno_ast::TextChange;
use deno_ast::TokenOrComment;

fn parse_params(text: &str, capture_tokens: bool) -> ParseParams {
  ParseParams {
    specifier: ModuleSpecifier::parse("file:///src/mod.ts").unwrap(),
    text: Arc::from(text),
    media_type: MediaType::TypeScript,
    capture_tokens,
    scope_analysis: false,
    maybe_syntax: None,
  }
}

#[test]
fn parses_typescript_modules_and_preserves_metadata() {
  let source =
    parse_module(parse_params("export const answer: number = 42;", true))
      .unwrap();

  assert_eq!(source.specifier().as_str(), "file:///src/mod.ts");
  assert_eq!(source.media_type(), MediaType::TypeScript);
  assert_eq!(source.text().as_ref(), "export const answer: number = 42;");
  assert!(!source.compute_is_script());
  assert!(!source.tokens().is_empty());
}

#[test]
fn lexes_tokens_and_comments_in_source_order() {
  let items = lex("const /* value */ answer = 42;", MediaType::JavaScript);
  assert!(items
    .windows(2)
    .all(|pair| pair[0].range.start <= pair[1].range.start));
  assert!(items
    .iter()
    .any(|item| matches!(item.inner, TokenOrComment::Comment { .. })));
}

#[test]
fn applies_non_overlapping_text_changes() {
  let result = apply_text_changes(
    "const value = 1;",
    vec![
      TextChange::new(6, 11, "answer".to_string()),
      TextChange::new(14, 15, "42".to_string()),
    ],
  );
  assert_eq!(result, "const answer = 42;");
}

#[test]
fn strips_a_utf8_bom() {
  assert_eq!(strip_bom("\u{feff}source".to_string()), "source");
  assert_eq!(strip_bom("source".to_string()), "source");
}
