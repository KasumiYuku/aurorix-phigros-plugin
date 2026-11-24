use std::future::Future;
use std::sync::Arc;

use aurorix_plugin::{
    async_handler, CommandHandler, CommandSpec, Invocation, Outgoing, PluginCommands, PluginError,
};
use aurorix_template::{Arg, Args};

use crate::service::PhigrosService;

const HELP_WORDS: [&str; 2] = ["help", "帮助"];

const GLUE: [&str; 21] = [
    "challenge", "suggest", "search", "score", "alias", "stats", "table", "ap30", "fc30", "song",
    "play", "rand", "list", "chap", "b30", "b50", "x30", "his", "aim", "cal", "ill",
];

pub fn plugin(service: Arc<PhigrosService>) -> PluginCommands {
    PluginCommands::new(crate::PLUGIN_ID, "Phigros 音游助手")
        .describe("Phigros 成绩查询、曲目检索、账号绑定与图片渲染")
        .command(root(&service))
        .with_config(crate::settings::fields())
}

fn root(service: &Arc<PhigrosService>) -> CommandSpec {
    let mut subs: Vec<CommandSpec> = Vec::new();
    subs.extend(crate::cmd_account::subcommands(service));
    subs.extend(crate::cmd_score::subcommands(service));
    subs.extend(crate::cmd_song::subcommands(service));
    subs.extend(crate::cmd_query::subcommands(service));
    subs.extend(crate::cmd_games::subcommands(service));
    subs.extend(crate::cmd_admin::subcommands(service));
    let glue = glue_targets(&subs);
    CommandSpec::new("p")
        .alias("phi")
        .describe("Phigros 指令总入口")
        .fallback(fallback(service, glue))
        .subcommands(subs)
}

fn glue_targets(subs: &[CommandSpec]) -> Vec<(String, Arc<dyn CommandHandler>)> {
    let mut out: Vec<(String, Arc<dyn CommandHandler>)> = subs
        .iter()
        .filter(|spec| GLUE.contains(&spec.name.as_str()))
        .filter_map(|spec| {
            spec.handler
                .clone()
                .map(|handler| (spec.name.clone(), handler))
        })
        .collect();
    out.sort_by(|left, right| {
        right
            .0
            .len()
            .cmp(&left.0.len())
            .then_with(|| left.0.cmp(&right.0))
    });
    out
}

fn fallback(
    service: &Arc<PhigrosService>,
    glue: Vec<(String, Arc<dyn CommandHandler>)>,
) -> Arc<dyn CommandHandler> {
    on(service, move |service, inv| {
        let glue = glue.clone();
        async move {
            let text = inv.text.trim();
            if text.is_empty() || HELP_WORDS.contains(&text) {
                return Ok(Some(crate::buttons::menu(&service)));
            }
            for (name, handler) in &glue {
                let Some(rest) = text.strip_prefix(name.as_str()) else {
                    continue;
                };
                if rest.is_empty() || rest.contains([' ', '\t']) {
                    continue;
                }
                let mut next = inv.clone();
                next.text = rest.to_string();
                next.command = name.clone();
                return handler.handle(next).await;
            }
            done()
        }
    })
}

pub fn on<F, Fut>(service: &Arc<PhigrosService>, run: F) -> Arc<dyn CommandHandler>
where
    F: Fn(Arc<PhigrosService>, Invocation) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<Outgoing>, PluginError>> + Send + 'static,
{
    let service = Arc::clone(service);
    async_handler(move |inv: Invocation| run(Arc::clone(&service), inv))
}

pub fn ok(out: Outgoing) -> Result<Option<Outgoing>, PluginError> {
    Ok(Some(out))
}

pub fn done() -> Result<Option<Outgoing>, PluginError> {
    Ok(None)
}

pub fn args() -> Args {
    Args::new()
}

pub fn put(args: &mut Args, key: &str, value: impl Into<String>) {
    args.insert(key.to_string(), Arg::Str(value.into()));
}

pub fn with_at(out: Outgoing, inv: &Invocation) -> Outgoing {
    Outgoing { at: false, ..out }.mention_user(inv.user.clone())
}
