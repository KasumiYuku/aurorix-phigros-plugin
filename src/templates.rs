use aurorix_template::{Registry, TemplateKind};

pub const MARKDOWN: [(&str, &str); 66] = [
    ("account.bind.ok", include_str!("../templates/markdown/account/account.bind.ok.md")),
    ("account.bind.ok.image", include_str!("../templates/markdown/account/account.bind.ok.image.md")),
    ("account.clean.confirm", include_str!("../templates/markdown/account/account.clean.confirm.md")),
    ("account.clean.ok", include_str!("../templates/markdown/account/account.clean.ok.md")),
    ("account.login.ok", include_str!("../templates/markdown/account/account.login.ok.md")),
    ("account.login.ok.image", include_str!("../templates/markdown/account/account.login.ok.image.md")),
    ("account.login.ticket", include_str!("../templates/markdown/account/account.login.ticket.md")),
    ("account.me", include_str!("../templates/markdown/account/account.me.md")),
    ("account.pull.ok", include_str!("../templates/markdown/account/account.pull.ok.md")),
    ("account.unbind.confirm", include_str!("../templates/markdown/account/account.unbind.confirm.md")),
    ("account.unbind.ok", include_str!("../templates/markdown/account/account.unbind.ok.md")),
    ("admin.block.confirm", include_str!("../templates/markdown/admin/admin.block.confirm.md")),
    ("admin.block.list", include_str!("../templates/markdown/admin/admin.block.list.md")),
    ("admin.block.ok", include_str!("../templates/markdown/admin/admin.block.ok.md")),
    ("admin.block.pick", include_str!("../templates/markdown/admin/admin.block.pick.md")),
    ("admin.block.unblock.ok", include_str!("../templates/markdown/admin/admin.block.unblock.ok.md")),
    ("admin.stats", include_str!("../templates/markdown/admin/admin.stats.md")),
    ("admin.usage", include_str!("../templates/markdown/admin/admin.usage.md")),
    ("challenge", include_str!("../templates/markdown/challenge/challenge.md")),
    ("challenge.image", include_str!("../templates/markdown/challenge/challenge.image.md")),
    ("chapter.detail.image", include_str!("../templates/markdown/chapter/chapter.detail.image.md")),
    ("chapter.progress", include_str!("../templates/markdown/chapter/chapter.progress.md")),
    ("chapter.progress.image", include_str!("../templates/markdown/chapter/chapter.progress.image.md")),
    ("clip.hint", include_str!("../templates/markdown/clip/clip.hint.md")),
    ("clip.over", include_str!("../templates/markdown/clip/clip.over.md")),
    ("clip.over.plain", include_str!("../templates/markdown/clip/clip.over.plain.md")),
    ("clip.start", include_str!("../templates/markdown/clip/clip.start.md")),
    ("clip.step", include_str!("../templates/markdown/clip/clip.step.md")),
    ("clip.step.plain", include_str!("../templates/markdown/clip/clip.step.plain.md")),
    ("fail", include_str!("../templates/markdown/shared/fail.md")),
    ("guess.hint", include_str!("../templates/markdown/guess/guess.hint.md")),
    ("guess.over", include_str!("../templates/markdown/guess/guess.over.md")),
    ("guess.start", include_str!("../templates/markdown/guess/guess.start.md")),
    ("guess.step", include_str!("../templates/markdown/guess/guess.step.md")),
    ("guess.step.plain", include_str!("../templates/markdown/guess/guess.step.plain.md")),
    ("help", include_str!("../templates/markdown/help/help.md")),
    ("letter.over", include_str!("../templates/markdown/letter/letter.over.md")),
    ("letter.start", include_str!("../templates/markdown/letter/letter.start.md")),
    ("letter.step", include_str!("../templates/markdown/letter/letter.step.md")),
    ("notice", include_str!("../templates/markdown/shared/notice.md")),
    ("pick", include_str!("../templates/markdown/shared/pick.md")),
    ("query.list", include_str!("../templates/markdown/query/query.list.md")),
    ("query.push", include_str!("../templates/markdown/query/query.push.md")),
    ("query.stats", include_str!("../templates/markdown/query/query.stats.md")),
    ("score.b30", include_str!("../templates/markdown/score/score.b30.md")),
    ("score.b30.image", include_str!("../templates/markdown/score/score.b30.image.md")),
    ("score.cal", include_str!("../templates/markdown/score/score.cal.md")),
    ("score.extra", include_str!("../templates/markdown/score/score.extra.md")),
    ("score.song", include_str!("../templates/markdown/score/score.song.md")),
    ("snipe", include_str!("../templates/markdown/snipe/snipe.md")),
    ("song.alias", include_str!("../templates/markdown/song/song.alias.md")),
    ("song.detail", include_str!("../templates/markdown/song/song.detail.md")),
    ("song.ill", include_str!("../templates/markdown/song/song.ill.md")),
    ("song.image", include_str!("../templates/markdown/song/song.image.md")),
    ("song.pending", include_str!("../templates/markdown/song/song.pending.md")),
    ("song.play", include_str!("../templates/markdown/song/song.play.md")),
    ("song.rand", include_str!("../templates/markdown/song/song.rand.md")),
    ("song.rand.image", include_str!("../templates/markdown/song/song.rand.image.md")),
    ("song.search.image", include_str!("../templates/markdown/song/song.search.image.md")),
    ("songinfo", include_str!("../templates/markdown/songinfo/songinfo.md")),
    ("songinfo.image", include_str!("../templates/markdown/songinfo/songinfo.image.md")),
    ("table.sheet", include_str!("../templates/markdown/table/table.sheet.md")),
    ("table.sheet.image", include_str!("../templates/markdown/table/table.sheet.image.md")),
    ("unbound", include_str!("../templates/markdown/shared/unbound.md")),
    ("video.info", include_str!("../templates/markdown/video/video.info.md")),
    ("warn", include_str!("../templates/markdown/shared/warn.md")),
];

pub fn register_templates(registry: &Registry) -> usize {
    let mut registered = 0;
    for (id, body) in MARKDOWN {
        registry.register(TemplateKind::Markdown, crate::PLUGIN_ID, id, body);
        registered += 1;
    }
    registered
}

pub fn template_ids() -> Vec<&'static str> {
    MARKDOWN.iter().map(|(id, _)| *id).collect()
}
