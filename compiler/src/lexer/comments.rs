use super::{Diagnostic, Span, Token, TokenKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Documentation {
    pub module: bool,
    pub bars: usize,
    pub open: Span,
    pub body: Span,
    pub close: Span,
}

impl Token {
    pub fn documentation(&self) -> Option<Documentation> {
        let TokenKind::Doc { module, bars } = self.kind else {
            return None;
        };
        if bars == 0 || self.span.end.checked_sub(self.span.start)? != self.text.len() {
            return None;
        }
        let width = bars.checked_add(1 + usize::from(module))?;
        let start = self.span.start.checked_add(width)?;
        let end = self.span.end.checked_sub(width)?;
        (start <= end).then_some(Documentation {
            module,
            bars,
            open: Span::new(self.span.start, start),
            body: Span::new(start, end),
            close: Span::new(end, self.span.end),
        })
    }
}

pub(crate) fn scan(source: &str, start: usize, errors: &mut Vec<Diagnostic>) -> (usize, TokenKind) {
    let bytes = source.as_bytes();
    let module = bytes.get(start + 1) == Some(&b'!') && bytes.get(start + 2) == Some(&b'|');
    let first = start + 1 + usize::from(module);
    let mut pos = first;
    while bytes.get(pos) == Some(&b'|') {
        pos += 1;
    }
    let bars = pos - first;
    if bars == 0 {
        pos = start + 1;
        while pos < bytes.len() && bytes[pos] != b'#' {
            pos += 1;
        }
        if pos == bytes.len() {
            errors.push(Diagnostic::new(
                "E002",
                "unclosed comment",
                Span::new(start, pos),
            ));
        } else {
            pos += 1;
        }
        return (pos, TokenKind::Comment);
    }
    let open = Span::new(start, pos);
    let kind = TokenKind::Doc { module, bars };
    while pos < bytes.len() {
        if bytes[pos] != b'|' {
            pos += 1;
            continue;
        }
        let first = pos;
        while bytes.get(pos) == Some(&b'|') {
            pos += 1;
        }
        if pos - first != bars {
            continue;
        }
        if module {
            if bytes.get(pos) == Some(&b'!') && bytes.get(pos + 1) == Some(&b'#') {
                return (pos + 2, kind);
            }
        } else if bytes.get(pos) == Some(&b'#') {
            return (pos + 1, kind);
        }
    }
    let suffix = if module { "!#" } else { "#" };
    errors.push(Diagnostic::new(
        "E002",
        format!("unclosed documentation comment; expected {bars} bar(s) followed by `{suffix}`"),
        open,
    ));
    (pos, kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    pub(crate) fn doc_fences_preserve_bytes_kinds_and_payload_spans() {
        let source = "#||\r\n# Heading \u{e9}\r\n|# short\r\n||#\r\n#!||| module ||!# short |||!#\r\n##\"#| literal |#\"";
        let tokens = lex(source).unwrap();
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<String>(),
            source
        );
        for token in &tokens {
            assert_eq!(&source[token.span.start..token.span.end], token.text);
        }
        let docs = tokens
            .iter()
            .filter_map(Token::documentation)
            .collect::<Vec<_>>();
        assert_eq!(docs.len(), 2);
        assert!(!docs[0].module);
        assert_eq!(docs[0].bars, 2);
        assert_eq!(docs[0].open, Span::new(0, 3));
        assert_eq!(
            &source[docs[0].body.start..docs[0].body.end],
            "\r\n# Heading \u{e9}\r\n|# short\r\n"
        );
        assert_eq!(&source[docs[0].close.start..docs[0].close.end], "||#");
        assert!(docs[1].module);
        assert_eq!(docs[1].bars, 3);
        assert_eq!(&source[docs[1].close.start..docs[1].close.end], "|||!#");
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Newline)
                .count(),
            2
        );
    }

    #[test]
    pub(crate) fn doc_fences_require_exact_maximal_closers() {
        for module in [false, true] {
            for bars in 1..=8 {
                let mark = if module { "!" } else { "" };
                let body = format!(
                    " short {}{mark}# long {}{mark}# done ",
                    "|".repeat(bars - 1),
                    "|".repeat(bars + 1)
                );
                let comment = format!(
                    "#{mark}{}{body}{}{mark}#",
                    "|".repeat(bars),
                    "|".repeat(bars)
                );
                let tokens = lex(&format!("{comment};x:1")).unwrap();
                assert_eq!(tokens[0].text, comment);
                assert_eq!(tokens[0].kind, TokenKind::Doc { module, bars });
                let doc = tokens[0].documentation().unwrap();
                assert_eq!(&comment[doc.body.start..doc.body.end], body);
                assert_eq!(tokens[1].text, ";");
            }
        }
    }

    #[test]
    pub(crate) fn doc_fences_keep_quotes_and_other_openers_inert() {
        let source = r###"#|| "#| text |#" # ordinary # { } #!| module |!# ||#;x:1"###;
        let tokens = lex(source).unwrap();
        assert_eq!(tokens[0].span.end, source.find(";x").unwrap());
        assert_eq!(tokens[0].documentation().unwrap().bars, 2);
        let source = "#| \" |#";
        assert_eq!(lex(source).unwrap()[0].text, source);
    }

    #[test]
    pub(crate) fn doc_fences_report_unclosed_openers() {
        for (source, open, bars, suffix) in [
            ("#|", 2, 1, "#"),
            ("#!|", 3, 1, "!#"),
            ("#||#", 3, 2, "#"),
            ("#| wrong |!#", 2, 1, "#"),
            ("#!|| wrong |!#", 4, 2, "!#"),
            ("#|| wrong |||#", 3, 2, "#"),
        ] {
            let errors = lex(source).unwrap_err();
            assert_eq!(errors[0].code, "E002", "{source}");
            assert_eq!(errors[0].span, Span::new(0, open));
            assert!(errors[0].message.contains(&format!("{bars} bar(s)")));
            assert!(errors[0].message.contains(&format!("`{suffix}`")));
        }
    }

    #[test]
    pub(crate) fn doc_fences_leave_ordinary_comments_and_strings_unchanged() {
        let source = "# | old | # #! old |# ## \"#| quoted |#\"";
        let tokens = lex(source).unwrap();
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Comment)
                .count(),
            3
        );
        assert!(tokens.iter().all(|token| token.documentation().is_none()));
        assert_eq!(
            tokens
                .iter()
                .find(|token| token.kind == TokenKind::String)
                .unwrap()
                .text,
            "\"#| quoted |#\""
        );
        let error = lex("# old").unwrap_err().remove(0);
        assert_eq!(error.message, "unclosed comment");
        assert_eq!(error.span, Span::new(0, 5));
    }

    #[test]
    pub(crate) fn doc_fences_are_scanned_inside_interpolation() {
        let source = r###""before {{#|| } " #| inner |# ||# x:7;->x}} after""###;
        let tokens = lex(source).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].text, source);
        let source = "\"before {#|| unfinished\"";
        let error = lex(source).unwrap_err().remove(0);
        assert_eq!(error.code, "E002");
        let start = source.find("#||").unwrap();
        assert_eq!(error.span, Span::new(start, start + 3));
    }

    #[test]
    pub(crate) fn doc_fence_spans_follow_rebased_tokens() {
        let mut token = lex("#| body |#").unwrap().remove(0);
        token.span.start += 100;
        token.span.end += 100;
        let doc = token.documentation().unwrap();
        assert_eq!(doc.open, Span::new(100, 102));
        assert_eq!(doc.body, Span::new(102, token.span.end - 2));
        assert_eq!(doc.close, Span::new(token.span.end - 2, token.span.end));
        token.kind = TokenKind::Doc {
            module: true,
            bars: usize::MAX,
        };
        assert!(token.documentation().is_none());
    }

    #[test]
    pub(crate) fn doc_fences_scan_long_near_matches_without_expanding_diagnostics() {
        let bars = "|".repeat(64);
        let wrong = format!("{}#{}#", "|".repeat(63), "|".repeat(65)).repeat(1024);
        let source = format!("#{bars} {wrong}{bars}#");
        let token = lex(&source).unwrap().remove(0);
        assert_eq!(token.span.end, source.len());
        assert_eq!(token.documentation().unwrap().bars, 64);
        let source = format!("#{} unfinished", "|".repeat(4096));
        let error = lex(&source).unwrap_err().remove(0);
        assert!(error.message.len() < 120);
        assert_eq!(error.span, Span::new(0, 4097));
    }
}
