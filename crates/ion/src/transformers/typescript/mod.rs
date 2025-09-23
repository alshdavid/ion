use oxc::allocator::Allocator;
use oxc::ast::ast::SourceType;
use oxc::codegen::Codegen;
use oxc::codegen::CodegenOptions;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::transformer::JsxOptions;
use oxc::transformer::TransformOptions;
use oxc::transformer::Transformer;
use oxc::transformer::TypeScriptOptions;

use crate::JsTransformer;
use crate::TransformerContext;
use crate::TransformerResult;

pub fn ts() -> JsTransformer {
    JsTransformer {
        kind: "ts".to_string(),
        transformer: Box::new(transformer),
    }
}

pub fn tsx() -> JsTransformer {
    JsTransformer {
        kind: "tsx".to_string(),
        transformer: Box::new(transformer),
    }
}

fn transformer(ctx: TransformerContext) -> crate::Result<TransformerResult> {
    let source = String::from_utf8(ctx.content)?;

    let allocator = Allocator::default();

    let source_type = match ctx.kind.as_str() {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        _ => {
            return Err(crate::Error::TransformerError(
                "Got an invalid file type".to_string(),
            ));
        }
    };

    let parse_result = Parser::new(&allocator, &source, source_type).parse();
    if !parse_result.errors.is_empty() {
        // If we're parsing a .ts file, it is common for users to accidentally include JSX.
        // Re-parse in TSX mode to detect this case and emit a clearer error.
        if ctx.kind == "ts" {
            let allocator_tsx = Allocator::default();
            let parse_as_tsx = Parser::new(&allocator_tsx, &source, SourceType::tsx()).parse();
            // To avoid false positives (e.g. angle-bracket type assertions), also require
            // an obvious JSX marker in the source when TS parsing fails but TSX parsing succeeds.
            let looks_like_jsx =
                source.contains("</") || source.contains("/>") || source.contains("<>");
            if parse_as_tsx.errors.is_empty() && looks_like_jsx {
                return Err(crate::Error::TransformerError(format!(
                    "JSX syntax detected in a .ts file: {}. Rename the file to .tsx or remove JSX.",
                    ctx.path.display()
                )));
            }
        }

        let errors: Vec<String> = parse_result
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        return Err(crate::Error::TransformerError(format!(
            "Parse errors: {}",
            errors.join(", ")
        )));
    }
    let mut program = parse_result.program;

    let scoping_result = SemanticBuilder::new().build(&program);
    if !scoping_result.errors.is_empty() {
        let errors: Vec<String> = scoping_result
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        return Err(crate::Error::TransformerError(format!(
            "Parse errors: {}",
            errors.join(", ")
        )));
    }
    let scoping = scoping_result.semantic.into_scoping();

    let transform_options = TransformOptions {
        typescript: TypeScriptOptions::default(),
        jsx: match source_type.is_jsx() {
            true => JsxOptions::default(),
            false => JsxOptions::disable(),
        },
        ..Default::default()
    };

    let result = Transformer::new(&allocator, &ctx.path, &transform_options);
    let build_result = result.build_with_scoping(scoping, &mut program);
    if !build_result.errors.is_empty() {
        let errors: Vec<String> = build_result
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        return Err(crate::Error::TransformerError(format!(
            "Transform errors: {}",
            errors.join(", ")
        )));
    }

    let generated = Codegen::new()
        .with_options(CodegenOptions {
            minify: false,
            ..CodegenOptions::default()
        })
        .build(&program);

    Ok(TransformerResult {
        code: generated.code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsTransformer;
    use crate::TransformerContext;
    use std::path::PathBuf;

    #[test]
    fn ts_with_jsx_returns_clear_error() {
        let transformer: JsTransformer = ts();
        let ctx = TransformerContext {
            content: b"const x = <div/>;".to_vec(),
            path: PathBuf::from("test.ts"),
            kind: "ts".to_string(),
        };

        let err = (transformer.transformer)(ctx).unwrap_err();
        match err {
            crate::Error::TransformerError(msg) => {
                assert!(
                    msg.contains("JSX syntax detected in a .ts file"),
                    "unexpected message: {}",
                    msg
                );
                assert!(msg.contains("test.ts"), "path missing in message: {}", msg);
            }
            other => panic!("expected TransformerError, got {:?}", other),
        }
    }
}
