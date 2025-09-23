use crate::JsResolver;
use crate::ResolverContext;
use crate::ResolverResult;

pub async fn run_resolvers(
    resolvers: &Vec<JsResolver>,
    ctx: ResolverContext,
) -> crate::Result<Option<ResolverResult>> {
    for resolver in resolvers {
        match resolver(ResolverContext {
            fs: ctx.fs.clone(),
            specifier: ctx.specifier.clone(),
            from: ctx.from.clone(),
        })
        .await?
        {
            Some(result) => {
                return Ok(Some(result));
            }
            None => continue,
        }
    }

    // Always fall back to resolving relative paths
    crate::resolvers::relative()(ctx).await
}
