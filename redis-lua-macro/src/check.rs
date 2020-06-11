use crate::{file::as_path, proc_macro::Span, script::Script};
use full_moon::{
    ast::{owned::Owned, AstError},
    tokenizer::Token,
    Error as ParseError,
};
use proc_macro_error::{Diagnostic as PDiagnostic, Level as PLevel};
use selene_lib::{
    rules::Severity, standard_library::StandardLibrary, Checker as SeleneChecker, CheckerConfig,
    CheckerDiagnostic,
};
use std::include_str;

fn convert_level(l: Severity) -> PLevel {
    match l {
        Severity::Error => PLevel::Error,
        #[cfg(unstable)]
        Severity::Warning => PLevel::Warning,
        #[cfg(not(unstable))]
        Severity::Warning => PLevel::Error,
    }
}

fn emit_diag_one(span: Vec<Span>, cd: CheckerDiagnostic) {
    let d = cd.diagnostic;
    let msg = format!("in lua: {} ({})", d.message, d.code);

    let pd = match span.get(0).cloned() {
        Some(span) => PDiagnostic::spanned(span.into(), convert_level(cd.severity), msg),
        None => PDiagnostic::new(convert_level(cd.severity), msg),
    };
    let pd = d
        .notes
        .iter()
        .fold(pd, |pd, note| pd.note(note.to_string()));
    pd.emit()
}

fn emit_parse_err(script: &Script, msg: &str, token: Option<&Token>) {
    let range = match token {
        Some(token) => (token.start_position().bytes(), token.end_position().bytes()),
        None => (0, script.script().len()),
    };
    let spans = script.range_to_span(range);

    let msg = format!("in lua: {} (parse_error)", msg);

    let pd = match spans.get(0).cloned() {
        Some(span) => PDiagnostic::spanned(span.into(), PLevel::Error, msg),
        None => PDiagnostic::new(PLevel::Error, msg),
    };
    pd.emit();
}

fn emit_diag(script: &Script, diags: Vec<CheckerDiagnostic>) {
    for d in diags {
        let label = d.diagnostic.primary_label.range;
        let spans = script.range_to_span((label.0 as usize, label.1 as usize));
        emit_diag_one(spans.clone(), d);
    }
}

fn make_cfg(args: &[String]) -> String {
    let cfg = include_str!("redis.toml").to_string();

    let cfg = args.iter().fold(cfg, |cfg, arg| {
        let new_rule = format!(
            r#"[{}]
property = true"#,
            arg
        );

        format!("{}\n{}", cfg, new_rule)
    });

    cfg
}

pub struct Checker {
    defined: Vec<String>,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            defined: Vec::new(),
        }
    }

    pub fn define(&mut self, s: &str) -> &mut Self {
        self.defined.push(s.into());
        self
    }

    pub fn defines(&mut self, s: Vec<String>) -> &mut Self {
        self.defined.extend(s);
        self
    }

    pub fn check(&self, script: &Script) {
        let ast = match full_moon::parse(script.script()) {
            Ok(ast) => ast.owned(),
            Err(ParseError::AstError(AstError::UnexpectedToken {
                token,
                additional: _,
            })) => {
                return emit_parse_err(
                    script,
                    &format!("unexpected token `{}`", token),
                    Some(&token),
                );
            }
            Err(_) => {
                return emit_parse_err(script, "cannot tokenize lua script", None);
            }
        };

        let std = StandardLibrary::from_file(&as_path(&make_cfg(&self.defined))).unwrap();
        let cfg: CheckerConfig<toml::value::Value> =
            toml::from_str(include_str!("selene.toml")).unwrap();

        // Create a linter
        let checker = SeleneChecker::new(cfg, std.unwrap()).unwrap();

        // Run the linter
        let mut diags = checker.test_on(&ast);
        diags.sort_by_key(|d| d.diagnostic.start_position());

        // Emit results as compiler messages
        emit_diag(script, diags);
    }
}
