use std::path::{Path, PathBuf};

use aurorix_plugin::{PluginAssets, PluginCommands};

pub mod backend;
pub mod cards;
pub mod domain;
pub mod model;
pub mod history;
pub mod rank;
pub mod repo;
pub mod templates;
pub mod warm;
pub mod commands;
pub mod cmd_account;
pub mod cmd_admin;
pub mod cmd_games;
pub mod cmd_query;
pub mod cmd_score;
pub mod cmd_song;
pub mod games_buttons;
pub mod videoclip;
pub mod buttons;
pub mod service;
pub mod settings;
pub mod webui;

pub const PLUGIN_ID: &str = "phigros";

pub const PLUGIN_NAME: &str = "Phigros";

pub const ASSETS_DIR: &str = "assets";

pub fn assets_root() -> PathBuf {
    let local = Path::new(ASSETS_DIR).join(PLUGIN_ID);
    if local.is_dir() {
        return local;
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join(ASSETS_DIR)
}

pub fn assets() -> PluginAssets {
    assets_in(assets_root())
}

pub fn assets_in(root: impl Into<PathBuf>) -> PluginAssets {
    PluginAssets::new(PLUGIN_ID, root)
}

pub async fn install(
    db: std::sync::Arc<aurorix_db::PluginDb>,
    backend: std::sync::Arc<crate::backend::SaveClient>,
) -> Result<PluginCommands, String> {
    install_with_report(db, backend)
        .await
        .map(|(commands, _, _)| commands)
}
pub async fn install_with_report(

    db: std::sync::Arc<aurorix_db::PluginDb>,
    backend: std::sync::Arc<crate::backend::SaveClient>,
) -> Result<(PluginCommands, crate::webui::WebuiService, String), String> {

    crate::repo::Repo::open(db.clone())
        .await
        .map_err(|error| format!("插件数据库迁移失败：{error}"))?;
    let mut notes: Vec<String> = Vec::new();
    match crate::repo::legacy::import_once(&db).await {
        Ok(report) => notes.push(report.summary()),
        Err(error) => notes.push(format!("旧数据迁移跳过：{error}")),
    }
    let registry = aurorix_template::Registry::global();
    let templates = crate::templates::register_templates(registry);
    notes.push(format!("模板 {templates} 个"));
    let assets = std::sync::Arc::new(assets());
    let fonts = crate::cards::register_fonts(assets.root());
    notes.push(format!("字体 {fonts} 支"));
    let context = aurorix_render::RenderContext::new(
        aurorix_render::SkiaBackend::handle(),
        16,
        64 * 1024 * 1024,
    )
    .with_asset_loader(assets.loader());
    let warm = crate::warm::warmup(&assets, &context);
    notes.push(format!("预热 {}", warm.summary()));
    let path = crate::service::catalog_path(assets.root());
    let catalog = std::sync::Arc::new(
        crate::domain::catalog::Catalog::open(&path)
            .map_err(|error| format!("曲库打开失败 {}：{error}", path.display()))?,
    );
    notes.push(format!("曲库 {} 首", catalog.songs().len()));
    let service = crate::service::PhigrosService::new(
        catalog,
        assets.clone(),
        context,
        backend,
        db,
    );
    match service.refresh_aliases().await {
        Ok(()) => notes.push(String::from("别名索引已合并")),
        Err(error) => notes.push(format!("别名索引跳过：{error}")),
    }
    crate::history::spawn(service.clone());
    let commands = crate::commands::plugin(service.clone());
    let mut callbacks = crate::buttons::install(&service);
    callbacks += crate::cmd_admin::install(&service);
    notes.push(format!("按钮回调 {callbacks} 个"));
    let webui = crate::webui::WebuiService::new(service.clone());
    Ok((commands, webui, notes.join("；")))
}
