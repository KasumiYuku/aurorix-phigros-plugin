use std::sync::Arc;

use aurorix_plugin::PluginAssets;
use aurorix_render::RenderContext;

pub const CORE_TEXTURES: [&str; 14] = [
    "ui/grade_f.png",
    "ui/grade_c.png",
    "ui/grade_b.png",
    "ui/grade_a.png",
    "ui/grade_s.png",
    "ui/grade_v.png",
    "ui/grade_phi.png",
    "ui/grade_fc.png",
    "ui/back_in_result.png",
    "ui/bar_avatar_default.png",
    "ui/bar_icon_avatar.png",
    "ui/bar_icon_illust.png",
    "ui/rank_white.png",
    "ui/rank_colorful.png",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Warmup {
    pub fonts: usize,
    pub templates: usize,
    pub textures: usize,
    pub missing: usize,
}

impl Warmup {
    pub fn summary(&self) -> String {
        format!(
            "字体 {} · 模板 {} · 贴图 {} · 缺失 {}",
            self.fonts, self.templates, self.textures, self.missing
        )
    }
}

pub fn warmup(assets: &PluginAssets, context: &RenderContext) -> Warmup {
    let registry = Arc::new(aurorix_template::Registry::new());
    let report = assets.bootstrap(&registry);
    let templates = crate::templates::register_templates(&registry);
    let mut textures = 0;
    let mut missing = 0;
    for key in CORE_TEXTURES {
        match context.asset(key) {
            Ok(_) => textures += 1,
            Err(_) => missing += 1,
        }
    }
    Warmup {
        fonts: report.fonts.len(),
        templates,
        textures,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmup_summary_is_human_readable() {
        let warmup = Warmup {
            fonts: 4,
            templates: 66,
            textures: 14,
            missing: 0,
        };
        assert!(warmup.summary().contains("模板 66"));
        assert_eq!(Warmup::default(), Warmup::default());
    }
}
