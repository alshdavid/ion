use std::path::PathBuf;
use std::sync::Arc;

use normalize_path::NormalizePath;

use crate::JsResolver;
use crate::JsResolverFut;
use crate::ResolverContext;
use crate::ResolverResult;
use crate::utils::OsStringExt;

pub fn relative() -> JsResolver {
    Arc::new(|ctx: ResolverContext| -> JsResolverFut {
        Box::pin(async move {
            let mut specifier = PathBuf::from(ctx.specifier);
            if specifier.is_relative() {
                specifier = ctx.from.parent().unwrap().join(&specifier).normalize();
            } else {
                specifier = specifier.normalize()
            }

            if !ctx.fs.try_exists(&specifier).await? {
                return Ok(None);
            }

            let Some(kind) = specifier.extension().map(|v| v.try_to_string().unwrap()) else {
                return Err(crate::Error::ResolveError);
            };

            Ok(Some(ResolverResult {
                code: ctx.fs.read(&specifier).await?,
                path: specifier,
                kind,
            }))
        })
    })
}

// TODO: virtual filesystem
//
// #[cfg(test)]
// mod test {
//     use std::path::PathBuf;

//     use crate::{self as ion};

//     #[tokio::test]
//     async fn should_resolve_relative_path() -> ion::Result<()> {
//         let ctx = ion::ResolverContext {
//             fs: ion::fs::FileSystem::Virtual,
//             specifier: "./foo.js".into(),
//             from: "/index.js".into(),
//         };

//         ctx.fs.write(&PathBuf::from("/index.js"), b"").await?;
//         ctx.fs.write(&PathBuf::from("/foo.js"), b"").await?;

//         let Some(result) = ion::resolvers::relative(ctx).await? else {
//             return ion::Error::generic_err("Unable to resolve relative path")
//         };

//         let expect = PathBuf::from("/").join("foo.js");
//         if result.path != expect {
//             return ion::Error::generic_err("Invalid path resolved")
//         }

//         Ok(())
//     }
// }
