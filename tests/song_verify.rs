use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, player::Player, sketch, song};
use aurorix_phigros_plugin::domain::catalog::Catalog;
use aurorix_phigros_plugin::model::{Difficulty, Song};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/04-song.png";

const MIN_ALIASES: usize = 20;

fn assets_root() -> PathBuf {
    aurorix_phigros_plugin::assets_root()
}

fn catalog_path() -> PathBuf {
    let root = assets_root();
    let mut candidates = vec![root.join("data").join("phi.db")];
    let real = root.canonicalize().unwrap_or_else(|_| root.clone());
    candidates.push(real.join("data").join("phi.db"));
    if let Some(parent) = real.parent() {
        candidates.push(parent.join("data").join("phi.db"));
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .expect("找不到 assets/data/phi.db")
}

fn decode_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let image = Image::from_encoded(Data::new_copy(bytes))?;
    let width = image.width() as u32;
    let height = image.height() as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut data = vec![0u8; width as usize * height as usize * 4];
    if !image.read_pixels(
        &info,
        &mut data,
        width as usize * 4,
        (0, 0),
        CachingHint::Allow,
    ) {
        return None;
    }
    Some((width, height, data))
}

fn pixel_at(data: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let at = (y * width + x) as usize * 4;
    [data[at], data[at + 1], data[at + 2], data[at + 3]]
}

fn luma(pixel: [u8; 4]) -> u8 {
    ((u32::from(pixel[0]) * 3 + u32::from(pixel[1]) * 6 + u32::from(pixel[2])) / 10) as u8
}

fn history_rows(song: &Song) -> usize {
    song.all_difficulties()
        .iter()
        .map(|difficulty| song.charts[difficulty.index()].history.len())
        .sum()
}

fn pick_song(catalog: &Catalog) -> &Song {
    catalog
        .songs()
        .iter()
        .find(|song| {
            song.difficulties().len() >= 3
                && history_rows(song) >= 1
                && song.aliases.len() >= MIN_ALIASES
        })
        .expect("曲库里没有\"多档 + 有定数变迁 + 别名够多\"的曲子")
}

fn version_label(change: &aurorix_phigros_plugin::model::ConstChange) -> String {
    let label = change.version_label.trim();
    if !label.is_empty() {
        return label.to_string();
    }
    if change.version > 0 {
        return format!("v{}", change.version);
    }
    String::new()
}

fn levels_of(song: &Song) -> Vec<song::SongLevel> {
    let mut levels: Vec<song::SongLevel> = Vec::new();
    for difficulty in song.all_difficulties() {
        let chart = &song.charts[difficulty.index()];
        levels.push(song::SongLevel {
            diff: difficulty.name().to_string(),
            constant: chart.constant,
            known: chart.known(),
            const_text: String::new(),
            notes: chart.notes,
            charter: chart.charter.clone(),
        });
    }
    for extra in &song.extra {
        levels.push(song::SongLevel {
            diff: extra.display(),
            constant: 0.0,
            known: true,
            const_text: extra.label.clone(),
            notes: extra.notes,
            charter: extra.charter.clone(),
        });
    }
    levels
}

fn history_of(song: &Song) -> Vec<song::SongConstChange> {
    let mut history: Vec<song::SongConstChange> = Vec::new();
    for difficulty in song.all_difficulties() {
        for change in &song.charts[difficulty.index()].history {
            history.push(song::SongConstChange {
                diff: difficulty.name().to_string(),
                version: version_label(change),
                from: change.from,
                to: change.to,
            });
        }
    }
    history
}

fn ranking_of(song: &Song, player: &Player) -> song::SongRanking {
    let mut rows: Vec<song::SongRankRow> = Vec::new();
    let mut total = 0;
    for difficulty in song.all_difficulties() {
        let chart = &song.charts[difficulty.index()];
        if !chart.known() {
            continue;
        }
        for place in 0..3_i32 {
            let rank = place + 1;
            let score = 999_000 - place * 713 - difficulty.index() as i32 * 101;
            let acc = 99.9 - f64::from(place) * 0.11 - difficulty.index() as f64 * 0.01;
            total += 1;
            let self_row = place == 2 && difficulty == Difficulty::Hd;
            rows.push(song::SongRankRow {
                diff: difficulty.name().to_string(),
                rank,
                name: if self_row {
                    player.name.clone()
                } else {
                    format!("Player{}{}", diff_seed(difficulty), rank)
                },
                avatar: String::from("Introduction"),
                score,
                acc,
                dan_level: place % 6,
                dan_progress: 10 + place,
                self_row,
            });
            if self_row {
                total += 1;
            }
        }
    }
    let own = song::SongRankRow {
        diff: String::from("IN"),
        rank: 24,
        name: player.name.clone(),
        avatar: player.avatar.clone(),
        score: 967_431,
        acc: 96.71,
        dan_level: 4,
        dan_progress: 7,
        self_row: true,
    };
    total += 1;
    rows.push(own);
    song::SongRanking { total, rows }
}

fn diff_seed(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Ez => "Ez",
        Difficulty::Hd => "Hd",
        Difficulty::In => "In",
        Difficulty::At => "At",
        Difficulty::Legacy => "Lg",
        Difficulty::Sp => "Sp",
    }
}

fn build_request(catalog: &Catalog) -> song::SongRequest {
    let picked = pick_song(catalog);
    let player = Player {
        has_player: true,
        name: String::from("Aurorix"),
        avatar: String::from("Introduction"),
        rank: 99,
        rank_level: 5,
        rks: 16.4231,
    };
    song::SongRequest {
        song_id: picked.id.clone(),
        title: picked.title.clone(),
        composer: picked.composer.clone(),
        illustrator: picked.illustrator.clone(),
        chapter: catalog.chapter_display(&picked.chapter),
        length: picked.length.clone(),
        bpm: picked.bpm_text.clone(),
        version: picked.version_label.clone(),
        aliases: picked.aliases.clone(),
        variant_of: picked.variant_of.clone(),
        sp: picked.sp.clone(),
        ranking: ranking_of(picked, &player),
        history: history_of(picked),
        levels: levels_of(picked),
    }
}

fn render(req: &song::SongRequest) -> Vec<u8> {
    let root = assets_root();
    cards::register_fonts(&root);
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = song::canvas_size(req);
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            song::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn layout_matches_go_song_page() {
    assert_eq!(song::CANVAS_W, 1920.0);
    assert_eq!(song::canvas_height(1, 0, 0, 0).round(), 1200.0);
    assert_eq!(song::canvas_height(2, 0, 0, 0).round(), 1272.0);
    assert_eq!(song::canvas_height(3, 0, 0, 0).round(), 1344.0);
    assert_eq!(song::canvas_height(4, 0, 0, 0).round(), 1416.0);
    assert_eq!(song::main_card().left(), 100.0);
    assert_eq!(song::main_card().width(), 1720.0);
    assert_eq!(song::cover_rect().left(), 128.0);
    assert_eq!(song::cover_rect().width(), 900.0);
    assert!((song::cover_rect().height() - 474.609_38).abs() < 0.01);
    assert_eq!(song::cover_title_at().0, 164.0);
    assert_eq!(song::rank_content().left(), 1056.0);
    assert_eq!(song::rank_score_right(), 1792.0);
    assert_eq!(song::info_card(4, 0, 0).left(), 100.0);
    assert_eq!(song::info_value_w(4), 392.0);

    let fonts = cards::register_fonts(&assets_root());
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let req = build_request(&catalog);
    assert!(
        req.levels.len() >= 3,
        "真实曲目至少要有三档谱面, 实际 {} 档",
        req.levels.len()
    );
    assert!(
        !req.aliases.is_empty(),
        "选中的曲目必须有别名, 否则别名整行画不出来"
    );
    assert!(
        !req.history.is_empty(),
        "选中的曲目必须有定数变迁, 否则变迁那块画不出来"
    );
    let lines = song::alias_lines(&req);
    let notes = song::note_lines(&req);
    assert!(!lines.is_empty(), "别名应当多到一格装不下");
    assert!(lines.len() <= song::INFO_ALIAS_MAX_LINES);
    let (width, height) = song::canvas_size(&req);
    assert_eq!(width, 1920);
    assert_eq!(
        height,
        song::canvas_height(
            req.levels.len(),
            req.history.len(),
            lines.len(),
            notes.len()
        )
        .round() as i32
    );
    assert!(
        height > 1200 + 72 * (req.levels.len() as i32 - 1),
        "带变迁与别名整行的页面必须比只有难度矩阵时更高: {height}"
    );
    assert_eq!(song::rank_label(&req.ranking.rows[0]), "#1");
    assert_eq!(song::rank_badge(0).width(), 61.0);
    assert!((song::rank_badge(0).height() - 20.0).abs() < 0.01);
    assert!((song::rank_badge_text_size() - 14.64).abs() < 0.01);
    let badge_mid = song::rank_badge(0).top() + song::rank_badge(0).height() / 2.0;
    assert!((song::rank_row_base(0) - 16.0 * 0.36 - badge_mid).abs() < 0.01);
}

#[test]
#[ignore]
fn render_song() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let fonts = cards::register_fonts(&assets_root());
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let request = build_request(&catalog);
    let alias_lines = song::alias_lines(&request);
    let note_lines = song::note_lines(&request);
    println!(
        "曲目 {} / {} 档 变迁 {} 行 别名 {} 条折 {} 行 说明 {} 行 排行 {} 行",
        request.song_id,
        request.levels.len(),
        request.history.len(),
        request.aliases.len(),
        alias_lines.len(),
        note_lines.len(),
        request.ranking.rows.len()
    );
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_FILE, &bytes).expect("写出 PNG");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!(
        "{OUT_FILE} {width}x{height} 渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    assert_eq!(width, 1920, "画布宽必须与 Go 的 SongW 一致");
    let want = song::canvas_height(
        request.levels.len(),
        request.history.len(),
        alias_lines.len(),
        note_lines.len(),
    )
    .round() as u32;
    assert_eq!(height, want, "画布高必须等于这一份数据的 SongHeight");
    assert!(height > 1000, "四档曲目页不该这么矮: {height}");

    let mut min_luma = 255u8;
    let mut max_luma = 0u8;
    let mut seen: HashSet<u8> = HashSet::new();
    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            let value = luma(pixel_at(&data, width, x, y));
            min_luma = min_luma.min(value);
            max_luma = max_luma.max(value);
            seen.insert(value);
        }
    }
    println!("亮度跨度 {} 灰阶 {}", max_luma - min_luma, seen.len());
    assert!(
        max_luma.saturating_sub(min_luma) >= 40,
        "亮度跨度太小, 可能是一张空画布: {}",
        max_luma.saturating_sub(min_luma)
    );
    assert!(seen.len() >= 40, "灰阶太少, 画布基本是平的: {}", seen.len());

    let cover = pixel_at(&data, width, 200, 300);
    let outside = pixel_at(&data, width, 20, 300);
    println!("曲绘 {cover:?} 页边 {outside:?}");
    assert_ne!(cover, outside, "曲绘与页面底不该是同一种颜色");
    assert_eq!(pixel_at(&data, width, 4, 4)[3], 255, "画布必须不透明");

    let mut corner_max = 0u8;
    let mut corner_sum = 0u64;
    let mut corner_count = 0u64;
    let mut y = 28u32;
    while y < 124 && y < height {
        let mut x = 1700u32;
        while x + 4 < width {
            let value = luma(pixel_at(&data, width, x, y));
            corner_max = corner_max.max(value);
            corner_sum += u64::from(value);
            corner_count += 1;
            x += 3;
        }
        y += 3;
    }
    let corner_mean = corner_sum as f64 / corner_count.max(1) as f64;
    let corner_spread = f64::from(corner_max) - corner_mean;
    println!("右上角均值 {corner_mean:.1} 峰值 {corner_max} 起伏 {corner_spread:.1}");
    assert!(
        corner_max < 96,
        "曲目页右上角应当是页面底色, 玩家信息条不该再画: 峰值 {corner_max}"
    );
    assert!(
        corner_spread < 40.0,
        "曲目页右上角应当是平的页面底色, 玩家信息条还在: 起伏 {corner_spread:.1}"
    );
}
