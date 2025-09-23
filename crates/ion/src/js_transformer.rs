use std::path::PathBuf;

pub struct JsTransformer {
    /// The file extension of the input file handled by this transformer.
    /// Example: "ts"
    pub kind: String,
    /// The callback run to transform the input file into JavaScript
    pub transformer:
        Box<dyn Sync + Send + Fn(TransformerContext) -> crate::Result<TransformerResult>>,
}

pub struct TransformerContext {
    /// The bytes of the input file
    pub content: Vec<u8>,
    /// Path to the source file
    pub path: PathBuf,
    /// The extension of the file being processed
    pub kind: String,
}

#[derive(Debug)]
pub struct TransformerResult {
    /// The transformed JavaScript to run
    pub code: String,
}
