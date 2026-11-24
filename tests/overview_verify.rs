use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, overview, player::Player, sketch};
use aurorix_phigros_plugin::domain::catalog::Catalog;
use aurorix_phigros_plugin::domain::overview as board;
use aurorix_phigros_plugin::model::{grade_of, Difficulty, Grade, CLEAR_SCORE, MAX_SCORE};
use aurorix_phigros_plugin::repo::{LeaderBoard, LeaderRow, RksDaily, RksPoint, ScoreEvent};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/05-overview.png";

const DAY: i64 = 86_400;

const CURVE_BASE: i64 = 1_735_689_600;

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

fn rks_points() -> Vec<RksPoint> {
    (0..10)
        .map(|index| RksPoint {
            observed_at: CURVE_BASE + index as i64 * 7 * DAY,
            rks: 15.40 + index as f64 * 0.11 + (index % 3) as f64 * 0.02,
            computed: 15.40 + index as f64 * 0.11,
            records: 120 + index as i64 * 7,
            phi: index as i64,
            challenge: index as i64 / 3,
        })
        .collect()
}

fn rks_daily() -> Vec<RksDaily> {
    (0..10)
        .map(|index| RksDaily {
            day: CURVE_BASE / DAY + index as i64 * 7,
            observed_at: CURVE_BASE + index as i64 * 7 * DAY,
            rks: 15.40 + index as f64 * 0.11 + (index % 3) as f64 * 0.02,
            records: 120 + index as i64 * 7,
            phi: index as i64,
            challenge: index as i64 / 3,
        })
        .collect()
}

struct Sample {
    score: i32,
    grade: Grade,
}

fn samples(catalog: &Catalog, difficulty: Difficulty) -> Vec<Sample> {
    let mut out: Vec<Sample> = Vec::new();
    let index = difficulty.index();
    for (order, song) in catalog.songs().iter().enumerate() {
        let chart = &song.charts[index];
        if !chart.known() {
            continue;
        }
        if order % 4 == 3 {
            continue;
        }
        let raw = 700_000 + ((order * 7_919 + index * 104_729) % 300_000) as i32;
        let score = raw.min(MAX_SCORE);
        let fc = score >= 960_000 && order % 2 == 0;
        out.push(Sample {
            score,
            grade: grade_of(score, fc),
        });
    }
    out
}

fn diffs(catalog: &Catalog) -> Vec<overview::OverviewDiff> {
    Difficulty::playable_all()
        .iter()
        .copied()
        .map(|difficulty| {
            let total = catalog
                .songs()
                .iter()
                .filter(|song| song.chart(difficulty).is_some())
                .count() as i32;
            let played = samples(catalog, difficulty);
            let cleared = played
                .iter()
                .filter(|sample| sample.score >= CLEAR_SCORE)
                .count() as i32;
            let full_combo = played
                .iter()
                .filter(|sample| sample.grade >= Grade::Fc)
                .count() as i32;
            let phi = played
                .iter()
                .filter(|sample| sample.score >= MAX_SCORE)
                .count() as i32;
            let best = played.iter().map(|sample| sample.score).max().unwrap_or(0);
            let worst = played
                .iter()
                .map(|sample| sample.score)
                .min()
                .unwrap_or(0);
            overview::OverviewDiff {
                label: difficulty.name().to_string(),
                played: played.len() as i32,
                total,
                cleared,
                full_combo,
                phi,
                best,
                worst,
            }
        })
        .collect()
}

fn leader_board(place: i64) -> LeaderBoard {
    let top: Vec<LeaderRow> = (0..10)
        .map(|index| LeaderRow {
            user_id: format!("u{index}"),
            nickname: format!("Player {index}"),
            avatar: String::from("Introduction"),
            rks: 16.80 - index as f64 * 0.05,
            challenge: (index as i64 * 137) % 600,
        })
        .collect();
    LeaderBoard {
        total: 51_912,
        me_place: place,
        me: Some(LeaderRow {
            user_id: String::from("me"),
            nickname: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rks: 16.4231,
            challenge: 505,
        }),
        top,
    }
}

fn score_events() -> Vec<ScoreEvent> {
    vec![
        ScoreEvent {
            observed_at: CURVE_BASE + 2 * DAY,
            song_id: String::from("Glaciaxion"),
            difficulty: Difficulty::In,
            has_old: true,
            old_score: 950_000,
            old_acc: 99.0,
            old_fc: true,
            new_score: MAX_SCORE,
            new_acc: 100.0,
            new_fc: true,
            old_rks: 0.0,
            new_rks: 0.0,
        },
        ScoreEvent {
            observed_at: CURVE_BASE + 3 * DAY,
            song_id: String::from("Spasmodic"),
            difficulty: Difficulty::At,
            has_old: true,
            old_score: 900_000,
            old_acc: 96.0,
            old_fc: false,
            new_score: 985_000,
            new_acc: 98.0,
            new_fc: true,
            old_rks: 0.0,
            new_rks: 0.0,
        },
    ]
}

fn build_request(catalog: &Catalog) -> overview::OverviewRequest {
    let diffs = diffs(catalog);
    let total_charts: i32 = diffs.iter().map(|diff| diff.total).sum();
    let total_songs = catalog.songs().len() as i32;
    let unlocked_songs = catalog
        .songs()
        .iter()
        .enumerate()
        .filter(|(order, song)| order % 5 != 4 && !song.difficulties().is_empty())
        .count() as i32;
    let backdrop = catalog
        .songs()
        .iter()
        .find(|song| song.chart(Difficulty::In).is_some())
        .map(|song| song.id.clone())
        .unwrap_or_default();
    overview::OverviewRequest {
        backdrop,
        player: Player {
            has_player: true,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rank: 99,
            rank_level: 5,
            rks: 16.4231,
        },
        server: String::from("国服"),
        self_intro: String::from("摸鱼中，偶尔推分。喜欢 15 上下的谱面，ACC 靠慢速背谱磨出来的。"),
        curve: overview::curve_from_points(&rks_points()),
        diffs,
        collection: overview::OverviewCollection {
            unlocked_charts: (total_charts * 4) / 5,
            total_charts,
            unlocked_songs,
            total_songs,
        },
        rank: board::rank_rows(&leader_board(12), false, ""),
        milestones: board::milestones(
            &score_events(),
            &rks_points(),
            CURVE_BASE - 400 * DAY,
            CURVE_BASE + 63 * DAY,
        ),
        synced: String::from("2026-03-07 21:30"),
    }
}

fn render(req: &overview::OverviewRequest) -> Vec<u8> {
    let root = assets_root();
    let fonts = cards::register_fonts(&root);
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = overview::canvas_size();
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            overview::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn layout_matches_go_overview_page() {
    assert_eq!(overview::CANVAS_W, 1440.0);
    assert_eq!(overview::canvas_size(), (1440, 1518));
    assert_eq!(overview::profile_card().left(), 48.0);
    assert_eq!(overview::profile_card().top(), 121.0);
    assert_eq!(overview::profile_card().width(), 1344.0);
    assert_eq!(overview::rank_card().left(), 48.0);
    assert_eq!(overview::diff_card().left(), overview::RIGHT_X);
    assert_eq!(overview::diff_card().left(), 744.0);
    assert_eq!(overview::rank_card().width(), overview::COL_W);
    assert_eq!(overview::rank_card().width(), 648.0);
    assert_eq!(overview::diff_card().width(), 648.0);
    assert_eq!(overview::curve_plot().left(), 120.0);
    assert_eq!(overview::curve_plot().width(), 844.0);
    assert_eq!(overview::curve_plot().height(), 240.0);
    assert_eq!(overview::milestone_col().width(), 360.0);
    assert_eq!(overview::rank_card().top(), 431.0);
    assert_eq!(overview::diff_card().top(), 431.0);
    assert_eq!(overview::rank_card().height(), 562.0);
    assert_eq!(overview::curve_card().top(), 1017.0);

    assert_eq!(overview::RANK_BADGE_W, 61.0);
    assert_eq!(overview::RANK_BADGE_H, 20.0);
    assert!((overview::RANK_BADGE_TEXT - 14.64).abs() < 0.01);
    assert_eq!(overview::BADGE_W, overview::RANK_BADGE_W);
    assert_eq!(overview::BADGE_H, overview::RANK_BADGE_H);
    let profile = overview::card_content(overview::profile_card());
    let badge = overview::profile_badge(profile, 120.0);
    assert_eq!(badge.width(), overview::RANK_BADGE_W);
    assert_eq!(badge.height(), overview::RANK_BADGE_H);
    let (_, name_base) = overview::profile_name_at(profile);
    assert!(
        (badge.top() + badge.height() / 2.0 - (name_base - overview::NAME_SIZE * overview::MID_RATIO))
            .abs()
            < 0.01
    );

    let points = overview::curve_from_points(&rks_points());
    assert_eq!(points.len(), 10);
    assert_eq!(points[0].at, CURVE_BASE);
    assert_eq!(
        overview::curve_from_daily(&rks_daily()),
        points,
        "日结与逐次采样必须给出同一条曲线"
    );
    let (lo, hi) = overview::curve_bounds(&points);
    assert!(lo <= points[0].rks && hi >= points[points.len() - 1].rks);
    assert_eq!(overview::peak_index(&points), Some(9));
    let (viewport_w, viewport_h) = overview::canvas_size();
    assert_eq!(viewport_w, 1440);
    assert_eq!(viewport_h, 1518);

    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let req = build_request(&catalog);
    assert_eq!(req.diffs.len(), 4);
    assert!(req.diffs.iter().all(|diff| diff.total > 0));
    assert_eq!(req.rank.total, 51_912);
    assert_eq!(req.rank.top.len(), 10);
    assert_eq!(req.rank.me.place, 12);
    assert_eq!(req.rank.omitted, 1);
    assert!(req.rank.has_data);
    assert_eq!(req.milestones.len(), 6);
    assert!(req.milestones.iter().all(|item| !item.day.is_empty()));
    assert!(req.collection.total_charts > req.collection.unlocked_charts);
}

#[test]
#[ignore]
fn render_overview() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let request = build_request(&catalog);
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_FILE, &bytes).expect("写出 PNG");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!(
        "{OUT_FILE} {width}x{height} 渲染耗时 {} ms ({} 字节, 曲线 {} 点, 榜 {} 行, 里程碑 {} 条)",
        elapsed.as_millis(),
        bytes.len(),
        request.curve.len(),
        request.rank.top.len(),
        request.milestones.len()
    );
    assert_eq!(width, 1440, "画布宽必须与 Go 的 OvW 一致");
    assert_eq!(height, 1518, "画布高必须等于 OvH");

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

    let plot = overview::curve_plot();
    let mut accent = 0usize;
    let mut y = plot.top().max(0.0) as u32;
    while (y as f32) < plot.bottom() && y < height {
        let mut x = plot.left().max(0.0) as u32;
        while (x as f32) < plot.right() && x < width {
            let pixel = pixel_at(&data, width, x, y);
            if pixel[1] > 140 && pixel[2] > 140 && pixel[0] < pixel[1] {
                accent += 1;
            }
            x += 1;
        }
        y += 1;
    }
    println!("绘图区强调色像素 {accent}");
    assert!(accent >= 200, "RKS 折线没有画进绘图区: {accent}");
    let card = overview::rank_card();
    let column = overview::milestone_col();
    let mut rank_min = 255u8;
    let mut rank_max = 0u8;
    let mut rank_accent = 0usize;
    let mut y = card.top().max(0.0) as u32;
    while (y as f32) < card.bottom() && y < height {
        let mut x = card.left().max(0.0) as u32;
        while (x as f32) < card.right() && x < width {
            let pixel = pixel_at(&data, width, x, y);
            let value = luma(pixel);
            rank_min = rank_min.min(value);
            rank_max = rank_max.max(value);
            if pixel[1] > 150 && pixel[2] > 150 && pixel[0] < pixel[1] {
                rank_accent += 1;
            }
            x += 1;
        }
        y += 1;
    }
    let mut column_min = 255u8;
    let mut column_max = 0u8;
    let mut y = column.top().max(0.0) as u32;
    while (y as f32) < column.bottom() && y < height {
        let mut x = column.left().max(0.0) as u32;
        while (x as f32) < column.right() && x < width {
            let value = luma(pixel_at(&data, width, x, y));
            column_min = column_min.min(value);
            column_max = column_max.max(value);
            x += 1;
        }
        y += 1;
    }
    let mut flags = [false; 12];
    flags[10] = true;
    let shares = overview::rank_shares(&flags);
    let me_slot = overview::rank_slot(&shares, 11);
    let top_slot = overview::rank_slot(&shares, 0);
    let mut me_luma = 0u64;
    let mut me_count = 0u64;
    let mut y = me_slot.0.max(0.0) as u32;
    while (y as f32) < me_slot.1 && y < height {
        let mut x = card.left().max(0.0) as u32;
        while (x as f32) < card.right() && x < width {
            me_luma += u64::from(luma(pixel_at(&data, width, x, y)));
            me_count += 1;
            x += 1;
        }
        y += 1;
    }
    let mut top_luma = 0u64;
    let mut top_count = 0u64;
    let mut y = top_slot.0.max(0.0) as u32;
    while (y as f32) < top_slot.1 && y < height {
        let mut x = card.left().max(0.0) as u32;
        while (x as f32) < card.right() && x < width {
            top_luma += u64::from(luma(pixel_at(&data, width, x, y)));
            top_count += 1;
            x += 1;
        }
        y += 1;
    }
    let me_mean = me_luma as f64 / me_count.max(1) as f64;
    let top_mean = top_luma as f64 / top_count.max(1) as f64;
    let band = overview::rank_band(me_slot.0, me_slot.1);
    let bar = overview::rank_me_bar(band);
    let bar_pixel = pixel_at(
        &data,
        width,
        (bar.left() + bar.width() / 2.0) as u32,
        (bar.top() + bar.height() / 2.0) as u32,
    );
    println!(
        "榜单卡亮度 {rank_min}~{rank_max} 强调色像素 {rank_accent} 我那一行均值 {me_mean:.1} 榜首均值 {top_mean:.1} 高亮条 {bar_pixel:?} 里程碑栏亮度 {column_min}~{column_max}"
    );
    assert!(
        rank_max.saturating_sub(rank_min) >= 30,
        "榜单卡几乎是平的: {}",
        rank_max.saturating_sub(rank_min)
    );
    assert!(rank_accent >= 40, "榜单卡没有画出段位徽章: {rank_accent}");
    assert!(
        me_mean > top_mean + 1.5,
        "我那一行的高亮带没有画出来: {me_mean:.1} vs {top_mean:.1}"
    );
    assert!(
        bar_pixel[1] > 150 && bar_pixel[2] > 150 && bar_pixel[0] < bar_pixel[1],
        "高亮带左缘的强调竖条没有画出来: {bar_pixel:?}"
    );
    assert!(
        column_max.saturating_sub(column_min) >= 30,
        "里程碑栏几乎是平的: {}",
        column_max.saturating_sub(column_min)
    );

    let content = overview::card_content(overview::diff_card());
    for index in 0..overview::DIFF_ROWS {
        let slot = overview::diff_bar_rect(content, index, content.right());
        let left = slot.left().round() as u32;
        let top = slot.top().round() as u32;
        let bottom = slot.bottom().round() as u32;
        let base_y = top.saturating_sub(6);
        let mid = (top + bottom) / 2;
        let mut run = 0u32;
        let mut end = left + 2;
        let mut x = left + 2;
        while x + 4 < width && run < 12 {
            let here = luma(pixel_at(&data, width, x, mid));
            let base = luma(pixel_at(&data, width, x, base_y));
            if here.abs_diff(base) >= 16 {
                run = 0;
                end = x;
            } else {
                run += 1;
            }
            x += 1;
        }
        assert!(
            end > left + 120,
            "第 {index} 档的完成度条太短, 找不到可验的那一段: {left}..{end}"
        );
        let mut gaps = 0usize;
        let mut scanned = 0usize;
        for y in (top + 1)..bottom.saturating_sub(1) {
            let mut x = left + 2;
            while x + 10 < end {
                let here = u32::from(luma(pixel_at(&data, width, x, y)));
                let mut around = [0u8; 2];
                for step in 1..=8u32 {
                    let at = x - step;
                    around[0] = around[0].max(luma(pixel_at(&data, width, at, y)));
                    let at = x + step;
                    around[1] = around[1].max(luma(pixel_at(&data, width, at, y)));
                }
                scanned += 1;
                if here + 12 <= u32::from(around[0]) && here + 12 <= u32::from(around[1]) {
                    gaps += 1;
                }
                x += 1;
            }
        }
        println!("第 {index} 档完成度条: 扫描 {scanned} 点, 段间凹口 {gaps} 点");
        assert!(
            scanned > 4000,
            "第 {index} 档的完成度条没画出实心: {scanned}"
        );
        assert_eq!(
            gaps, 0,
            "第 {index} 档的完成度条在段与段之间露了底色, 应当是一个圆角槽里铺平的几段"
        );
    }
    assert_eq!(pixel_at(&data, width, 4, 4)[3], 255, "画布必须不透明");
}
