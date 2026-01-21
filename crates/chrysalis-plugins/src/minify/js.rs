//! JavaScript minification using SWC.

use crate::{PluginError, Result};
use swc_core::common::{sync::Lrc, FileName, SourceMap, GLOBALS};
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_core::ecma::ast::*;
use swc_core::ecma::codegen::{text_writer::JsWriter, Emitter};
use std::path::PathBuf;

/// Minify JavaScript content using SWC.
pub fn minify_js(content: &[u8]) -> Result<Vec<u8>> {
    let content_str = std::str::from_utf8(content)
        .map_err(|e| PluginError::MinificationFailed {
            file: PathBuf::from("unknown.js"),
            reason: format!("UTF-8 error: {}", e),
        })?;

    GLOBALS.set(&Default::default(), || {
        let cm: Lrc<SourceMap> = Default::default();
        
        // Parse
        let fm = cm.new_source_file(
            FileName::Anon.into(),
            content_str.to_string(),
        );
        
        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module()
            .map_err(|e| PluginError::MinificationFailed {
                file: PathBuf::from("unknown.js"),
                reason: format!("Parse error: {:?}", e),
            })?;
        
        // Minify (simple optimization)
        let program = Program::Module(module);
        
        // Code generation
        let mut buf = vec![];
        {
            let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: swc_core::ecma::codegen::Config::default().with_minify(true),
                cm: cm.clone(),
                comments: None,
                wr: writer,
            };
            
            emitter.emit_program(&program)
                .map_err(|e| PluginError::MinificationFailed {
                    file: PathBuf::from("unknown.js"),
                    reason: format!("Emit error: {}", e),
                })?;
        }
        
        Ok(buf)
    })
}
