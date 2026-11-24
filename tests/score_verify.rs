use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, player, player::Player, score, settlement, sketch};
use aurorix_phigros_plugin::domain::catalog::Catalog;
use aurorix_phigros_plugin::model::{grade_of, Difficulty};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{images, AlphaType, ColorType, Data, Image, ImageInfo, Rect};
use aurorix_render::{CanvasKit, Color, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/02-score.png";

fn assets_root() -> PathBuf {
    aurorix_phigros_plugin::assets_root()
}

fn catalog_path() -> PathBuf {
    let root = assets_root();
    let mut candidates = vec![root.join("data").join("phi.db"), root.join("../data").join("phi.db")];
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

fn variance(data: &[u8]) -> f64 {
    let mut count = 0.0_f64;
    let mut sum = 0.0_f64;
    let mut square = 0.0_f64;
    for pixel in data.as_chunks::<4>().0 {
        let luma = (f64::from(pixel[0]) * 3.0
            + f64::from(pixel[1]) * 5.0
            + f64::from(pixel[2]) * 2.0)
            / 10.0;
        count += 1.0;
        sum += luma;
        square += luma * luma;
    }
    if count <= 0.0 {
        return 0.0;
    }
    (square / count - (sum / count).powi(2)).max(0.0)
}

fn timestamp(text: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(text)
        .expect("时间戳")
        .with_timezone(&chrono::Utc)
}

fn four_diff_song(catalog: &Catalog) -> &aurorix_phigros_plugin::model::Song {
    catalog
        .songs()
        .iter()
        .find(|song| song.difficulties().len() == 4)
        .expect("曲库里没有四档齐全的曲子")
}

fn build_request(catalog: &Catalog) -> score::ScoreRequest {
    let song = four_diff_song(catalog);
    let diff = Difficulty::In;
    let constant = song.constant(diff).expect("IN 档定数");
    let score_value = 996_233;
    let acc = 99.45;
    let history = 990_614;
    let mut scores = Vec::new();
    for difficulty in Difficulty::playable_all() {
        let chart = &song.charts[difficulty.index()];
        let played = chart.known();
        scores.push(settlement::DifficultyScore {
            diff: difficulty.name().to_string(),
            played,
            score: if played { 900_000 + difficulty.index() as i32 * 21_000 } else { 0 },
            acc: if played { 96.0 + difficulty.index() as f64 } else { 0.0 },
            grade: if played {
                grade_of(900_000, false).name().to_string()
            } else {
                String::new()
            },
            fc: played,
            phi: false,
            rks: if played { 12.0 + difficulty.index() as f64 } else { 0.0 },
            rank: if played { 40 + difficulty.index() as i32 } else { 0 },
            total: if played { 4_120 } else { 0 },
        });
    }
    let settled = settlement::SettlementRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        diff: diff.name().to_string(),
        constant,
        known: true,
        played: true,
        score: score_value,
        acc,
        history,
        fc: true,
        grade: grade_of(score_value, false).name().to_string(),
        player: Player {
            has_player: true,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rank: 99,
            rank_level: 5,
            rks: 16.4231,
        },
        standing: settlement::Standing {
            played: true,
            rks: 15.62,
            rank: 3,
            total: 312,
            suggest: 99.87,
            suggest_ok: true,
        },
        scores,
        chart_history: settlement::ChartHistory {
            points: vec![
                settlement::HistoryPoint {
                    at: timestamp("2025-09-04T12:00:00Z"),
                    score: 962_113,
                },
                settlement::HistoryPoint {
                    at: timestamp("2025-10-18T12:00:00Z"),
                    score: 978_402,
                },
                settlement::HistoryPoint {
                    at: timestamp("2025-12-21T09:05:00Z"),
                    score: 990_614,
                },
                settlement::HistoryPoint {
                    at: timestamp("2026-01-04T12:30:00Z"),
                    score: score_value,
                },
            ],
            gain: 34_120,
        },
        gained: String::from("768.00 KB"),
    };
    let mut rows = Vec::new();
    for index in 0..7 {
        let self_row = index == 3;
        rows.push(score::ScoreRankRow {
            rank: index + 1,
            name: if self_row {
                String::from("Aurorix")
            } else {
                format!("Player {}", index + 1)
            },
            avatar: String::from("Introduction"),
            rks: 16.5 - index as f64 * 0.11,
            dan_level: (index % 6).abs(),
            dan_progress: 10 + index,
            score: 996_233 - index * 911,
            acc: 99.45 - index as f64 * 0.13,
            self_row,
            hidden: false,
        });
    }
    score::ScoreRequest {
        settlement: settled,
        ranking: score::ScoreRanking {
            diff: diff.name().to_string(),
            total: 12_345,
            global: false,
            rows,
        },
    }
}
fn render(req: &score::ScoreRequest) -> Vec<u8> {
    let root = assets_root();
    let fonts = cards::register_fonts(&root);
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = score::canvas_size();
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            score::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn layout_and_labels_match_go() {
    assert_eq!(score::canvas_size(), (1920, 886));
    assert_eq!(score::SC_BOARD_ROWS, 7);
    assert!(score::board_row_h() > 60.0);
    assert!(score::title_max_width() > 400.0);
    assert_eq!(score::rank_label(&score::ScoreRankRow { rank: 7, ..Default::default() }), "#7");
    let ranking = score::ScoreRanking {
        diff: String::from("IN"),
        total: 12_345,
        global: true,
        rows: Vec::new(),
    };
    assert_eq!(score::score_board_count(&ranking), "全服共 12'345 人");
    let request = settlement::SettlementRequest {
        score: 996_233,
        played: true,
        history: 990_614,
        ..settlement::SettlementRequest::default()
    };
    assert_eq!(score::cards_data(&request).score_line, "996'233");
    assert_eq!(
        score::cards_data(&request).history_line,
        "NEW BEST 990'614 +5'619"
    );
    let cards = score::score_cards();
    assert_eq!(cards.backs.len(), 3);
    assert!(cards.score.left() < settlement::score_panel().left());
}

#[test]
#[ignore]
fn render_score() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let request = build_request(&catalog);
    println!("曲目 {} / {}", request.settlement.song_id, request.settlement.title);
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_FILE, &bytes).expect("写出 PNG");
    println!(
        "渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    assert!(PathBuf::from(OUT_FILE).is_file(), "PNG 必须落到 {OUT_FILE}");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!("{OUT_FILE} {width}x{height}");
    assert_eq!(width, 1920, "画布宽必须与 Go 的 W 一致");
    assert_eq!(height, 886, "画布高必须与 Go 的 H 一致");
    let spread = variance(&data);
    println!("像素方差 {spread:.2}");
    assert!(spread > 100.0, "画布不该是一片纯色, 方差 {spread:.2}");
    let avatar = player::shift_rect(
        player::avatar_window(),
        1.0,
        0.0,
        score::SC_PLAYER_BAR_DY.round(),
    );
    println!(
        "头像窗口 {}x{} @ ({}, {})",
        avatar.width(),
        avatar.height(),
        avatar.left(),
        avatar.top()
    );
    let contrast = max_gradient(&data, width, height, avatar);
    println!("头像区最大梯度 {contrast:.1}");
    assert!(contrast > 150.0, "头像区糊成一片: 梯度 {contrast:.1}");
}

#[test]
fn board_geometry_matches_go_device_pixels() {
    let area = score::board();
    println!(
        "board x={} y={} w={} h={} right={}",
        area.left(),
        area.top(),
        area.width(),
        area.height(),
        area.right()
    );
    assert!((area.left() - 1393.84).abs() < 0.05, "{}", area.left());
    assert!((area.right() - 1838.55).abs() < 0.05, "{}", area.right());
    assert!((area.top() - 152.96).abs() < 0.01);
    assert!((area.bottom() - 731.185).abs() < 0.01);
    let skewed = area.right() + score::LEAN * area.height() / 2.0;
    println!("board max right including skew {skewed:.3}");
    assert!((skewed - 1916.01).abs() < 0.05, "{skewed}");
    assert!(skewed <= 1920.0, "排行板右缘越出画布: {skewed}");
    let intercept = score::SC_BOARD_EDGE_LEFT + score::SC_BOARD_WIDTH;
    println!("unskewed intercept {intercept:.3}");
    assert!((intercept - 1957.0).abs() < 0.01);
    assert!((intercept - 1920.0 - 37.0).abs() < 0.01);
    let cards = score::score_cards();
    for (name, right) in [
        ("score", cards.score.right()),
        ("combo_and_acc", cards.combo_and_acc.right()),
        ("more_info", cards.more_info.right()),
        ("ratings", cards.ratings.right()),
    ] {
        println!("{name} right={right:.3}");
        assert!(right <= 1920.0, "{name} 越出画布: {right}");
    }
    assert!((cards.score.right() - 1404.456).abs() < 0.05);
    assert!((cards.ratings.right() - 1380.342).abs() < 0.05);
    assert!((cards.combo_and_acc.right() - 1340.893).abs() < 0.05);
    assert!((cards.more_info.right() - 1302.284).abs() < 0.05);
    let rows = score::SC_BOARD_ROWS - 1;
    let last = score::board_row_at(rows);
    let last_right = last.right()
        + sketch::skew_offset(
            score::board(),
            last.top() + last.height() / 2.0,
            score::LEAN,
        );
    assert!(last_right <= 1920.0, "{last_right}");
}

fn white_image(side: i32) -> Image {
    let info = ImageInfo::new(
        (side, side),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row = side as usize * 4;
    let data = vec![0xFFu8; row * side as usize];
    images::raster_from_data(&info, Data::new_copy(&data), row).expect("sample image")
}

fn luma_of(pixel: &[u8]) -> f64 {
    (f64::from(pixel[0]) * 3.0 + f64::from(pixel[1]) * 5.0 + f64::from(pixel[2]) * 2.0) / 10.0
}

fn row_run(data: &[u8], width: u32, y: u32, x0: u32, x1: u32) -> Option<(f32, f32)> {
    let mut left = None;
    let mut right = None;
    for x in x0..=x1.min(width - 1) {
        let at = (y as usize * width as usize + x as usize) * 4;
        if luma_of(&data[at..at + 4]) > 128.0 {
            left.get_or_insert(x);
            right = Some(x);
        }
    }
    match (left, right) {
        (Some(left), Some(right)) => Some((left as f32, right as f32 + 1.0)),
        _ => None,
    }
}

fn assert_slant_edges(data: &[u8], width: u32, area: Rect, lean: f32, tag: &str) {
    let cut = sketch::skew_cut(area, lean);
    let x0 = (area.left() - cut - 4.0).max(0.0) as u32;
    let x1 = (area.right() + cut + 4.0) as u32;
    let mut rows = Vec::new();
    let mut y = area.top() + 8.0;
    while y <= area.bottom() - 8.0 {
        let (left, right) = row_run(data, width, y.round() as u32, x0, x1)
            .unwrap_or_else(|| panic!("{tag}: row {y} has no pixels"));
        let shift = sketch::skew_offset(area, y, lean);
        let left_edge = area.left() + shift;
        let right_edge = area.right() + shift;
        assert!(
            (left - left_edge).abs() <= 2.5,
            "{tag}: left edge {left} at y {y} should be {left_edge}"
        );
        assert!(
            (right - right_edge).abs() <= 2.5,
            "{tag}: right edge {right} at y {y} should be {right_edge}"
        );
        assert!(
            ((right - left) - area.width()).abs() <= 3.0,
            "{tag}: run {} at y {y} should be {} wide",
            right - left,
            area.width()
        );
        rows.push((left, right));
        y += 4.0;
    }
    assert!(rows.len() >= 8, "{tag}: only {} sampled rows", rows.len());
    let first = rows[0];
    let last = rows[rows.len() - 1];
    let span = lean * 4.0 * (rows.len() - 1) as f32;
    assert!(
        first.0 - last.0 >= span * 0.7,
        "{tag}: left edge only moved {} across the slant",
        first.0 - last.0
    );
    assert!(
        first.1 - last.1 >= span * 0.7,
        "{tag}: right edge only moved {} across the slant",
        first.1 - last.1
    );
}
fn checker_image(side: i32) -> Image {
    let info = ImageInfo::new(
        (side, side),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row = side as usize * 4;
    let mut data = vec![0u8; row * side as usize];
    let cell = 8;
    let rim = 3;
    for y in 0..side {
        for x in 0..side {
            let border = x < rim || y < rim || x >= side - rim || y >= side - rim;
            let bright = border || (x / cell + y / cell) % 2 == 0;
            let value = if bright { 0xFFu8 } else { 0x00u8 };
            let at = y as usize * row + x as usize * 4;
            data[at..at + 4].copy_from_slice(&[value, value, value, 0xFF]);
        }
    }
    images::raster_from_data(&info, Data::new_copy(&data), row).expect("checker image")
}

fn luma_at(data: &[u8], width: u32, x: u32, y: u32) -> f64 {
    let at = (y as usize * width as usize + x as usize) * 4;
    luma_of(&data[at..at + 4])
}

fn content_box(data: &[u8], width: u32, height: u32, threshold: f64) -> (u32, u32, u32, u32) {
    let mut x0 = width;
    let mut y0 = height;
    let mut x1 = 0;
    let mut y1 = 0;
    let mut found = false;
    for y in 0..height {
        for x in 0..width {
            if luma_at(data, width, x, y) <= threshold {
                continue;
            }
            found = true;
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    assert!(found, "图上没有亮像素");
    (x0, y0, x1, y1)
}

fn max_gradient(data: &[u8], width: u32, height: u32, area: Rect) -> f64 {
    let x0 = area.left().max(0.0) as u32;
    let y0 = area.top().max(0.0) as u32;
    let x1 = (area.right().max(0.0) as u32).min(width - 1);
    let y1 = (area.bottom().max(0.0) as u32).min(height - 1);
    let mut best = 0.0_f64;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let here = luma_at(data, width, x, y);
            if x > x0 {
                best = best.max((here - luma_at(data, width, x - 1, y)).abs());
            }
            if y > y0 {
                best = best.max((here - luma_at(data, width, x, y - 1)).abs());
            }
        }
    }
    best
}

#[test]
fn avatar_sprite_draws_square_and_sharp() {
    let (width, height) = (420u32, 380u32);
    let window = Rect::from_xywh(60.0, 40.0, 210.0, 220.0);
    let image = checker_image(128);
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024);
    let request = aurorix_render::RenderRequest::new(width as i32, height as i32)
        .with_format(aurorix_render::RenderFormat::Png);
    let bytes = context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            sketch::ph_fill(
                &kit,
                Rect::from_xywh(0.0, 0.0, width as f32, height as f32),
                Color::new(0, 0, 0, 255),
            );
            sketch::image_sprite_skew(&kit, &image, window, player::SKEW);
        })
        .expect("render failed")
        .to_vec();
    let (w, h, data) = decode_png(&bytes).expect("decode");
    assert_eq!((w, h), (width, height));
    let (x0, y0, x1, y1) = content_box(&data, w, h, 64.0);
    let drawn_w = f64::from(x1 - x0 + 1);
    let drawn_h = f64::from(y1 - y0 + 1);
    let cut = sketch::skew_cut(window, player::SKEW);
    println!("头像绘制尺寸 {drawn_w}x{drawn_h}");
    let expect_w = f64::from(window.width()) + 2.0 * f64::from(cut);
    let expect_h = f64::from(window.height());
    assert!(
        (drawn_w - expect_w).abs() <= 3.0,
        "头像横向范围 {drawn_w} 应为 {expect_w}"
    );
    assert!(
        (drawn_h - expect_h).abs() <= 3.0,
        "头像纵向范围 {drawn_h} 应为 {expect_h}"
    );
    assert!(
        f64::from(x0) >= f64::from(window.left()) - 2.0 * f64::from(cut) - 3.0
            && f64::from(x1) <= f64::from(window.right()) + 2.0 * f64::from(cut) + 3.0
            && f64::from(y0) >= f64::from(window.top()) - 3.0
            && f64::from(y1) <= f64::from(window.bottom()) + 3.0,
        "头像超出窗口"
    );
    let mid_x = (x0 + x1) / 2;
    let mid_y = (y0 + y1) / 2;
    let mut left_step = 0.0_f64;
    let mut right_step = 0.0_f64;
    for y in (y0 + 1)..y1 {
        if let Some((left, right)) = row_run(&data, w, y, 0, w - 1) {
            let (left, right) = (left as u32, right as u32);
            left_step = left_step.max(luma_at(&data, w, left, y) - luma_at(&data, w, left - 1, y));
            right_step =
                right_step.max(luma_at(&data, w, right - 1, y) - luma_at(&data, w, right, y));
        }
    }
    let mut top_step = 0.0_f64;
    let mut bottom_step = 0.0_f64;
    for x in (x0 + 1)..x1 {
        top_step = top_step.max(luma_at(&data, w, x, y0) - luma_at(&data, w, x, y0 - 1));
        bottom_step = bottom_step.max(luma_at(&data, w, x, y1) - luma_at(&data, w, x, y1 + 1));
    }
    println!("边缘反差 左 {left_step:.1} 右 {right_step:.1} 上 {top_step:.1} 下 {bottom_step:.1}");
    let edges = [
        ("left", left_step),
        ("right", right_step),
        ("top", top_step),
        ("bottom", bottom_step),
    ];
    for (tag, step) in edges {
        assert!(step > 100.0, "{tag} 边缘发虚: 只差 {step:.1}");
    }
    let inside = Rect::from_xywh(x0 as f32, y0 as f32, drawn_w as f32, drawn_h as f32);
    let edge_diff = max_gradient(&data, w, h, inside);
    println!("头像内部最大梯度 {edge_diff:.1}");
    assert!(edge_diff > 100.0, "头像内部糊成一片: 梯度 {edge_diff:.1}");
    let span = 100u32;
    let mut across: u32 = 0;
    let mut down: u32 = 0;
    for x in (x0 + 20)..(x0 + 20 + span).min(w - 1) {
        let here = luma_at(&data, w, x, mid_y) > 128.0;
        let before = luma_at(&data, w, x - 1, mid_y) > 128.0;
        if here != before {
            across += 1;
        }
    }
    for y in (y0 + 20)..(y0 + 20 + span).min(h - 1) {
        let here = luma_at(&data, w, mid_x, y) > 128.0;
        let before = luma_at(&data, w, mid_x, y - 1) > 128.0;
        if here != before {
            down += 1;
        }
    }
    println!("棋盘格横向条纹 {across} / 纵向条纹 {down}");
    assert!(
        across.abs_diff(down) <= 1 && across >= 2,
        "头像被横向拉伸: 横向 {across} 条 / 纵向 {down} 条"
    );
}

#[test]
fn skew_clip_keeps_both_slant_edges_continuous() {
    let avatar = player::avatar_window();
    let avatar_area = sketch::ph_rect(40.0, 30.0, avatar.width(), avatar.height());
    let cover = settlement::cover();
    let cover_area = sketch::ph_rect(340.0, 30.0, cover.width(), cover.height());
    let (width, height) = (1400, 720);
    let image = white_image(256);
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024);
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    let bytes = context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            sketch::ph_fill(
                &kit,
                Rect::from_xywh(0.0, 0.0, width as f32, height as f32),
                Color::new(0, 0, 0, 255),
            );
            sketch::image_sprite_skew(&kit, &image, avatar_area, player::SKEW);
            sketch::image_cover_skew(&kit, &image, cover_area, settlement::LEAN);
        })
        .expect("render failed")
        .to_vec();
    let (w, h, data) = decode_png(&bytes).expect("decode");
    assert_eq!((w, h), (width as u32, height as u32));
    assert_slant_edges(&data, w, avatar_area, player::SKEW, "avatar");
    assert_slant_edges(&data, w, cover_area, settlement::LEAN, "cover");
}

#[test]
fn avatar_window_rows_follow_skew_slant() {
    let avatar = player::avatar_window();
    let area = sketch::ph_rect(40.0, 30.0, avatar.width(), avatar.height());
    let (width, height) = (1400, 720);
    let image = white_image(256);
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024);
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    let bytes = context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            sketch::ph_fill(
                &kit,
                Rect::from_xywh(0.0, 0.0, width as f32, height as f32),
                Color::new(0, 0, 0, 255),
            );
            sketch::image_sprite_skew(&kit, &image, area, player::SKEW);
        })
        .expect("render failed")
        .to_vec();
    let (w, h, data) = decode_png(&bytes).expect("decode");
    assert_eq!((w, h), (width as u32, height as u32));
    let cut = sketch::skew_cut(area, player::SKEW);
    let x0 = (area.left() - cut - 4.0).max(0.0) as u32;
    let x1 = ((area.right() + cut + 4.0) as u32).min(w - 1);
    let mid_y = area.top() + area.height() / 2.0;
    let mut rows: Vec<(f32, f32, f32)> = Vec::new();
    let mut y = area.top();
    while y < area.bottom() {
        let row = y.round() as u32;
        let (left, right) = row_run(&data, w, row, x0, x1)
            .unwrap_or_else(|| panic!("头像窗口第 {row} 行没有像素"));
        let shift = sketch::skew_offset(area, y, player::SKEW);
        let left_edge = area.left() + shift;
        let right_edge = area.right() + shift;
        println!(
            "y {row:>4} left {left:>6.1} / {left_edge:>7.2} right {right:>6.1} / {right_edge:>7.2} run {:>6.1}",
            right - left
        );
        assert!(
            (left - left_edge).abs() <= 2.5,
            "第 {row} 行左边缘 {left} 应为 {left_edge}"
        );
        assert!(
            (right - right_edge).abs() <= 2.5,
            "第 {row} 行右边缘 {right} 应为 {right_edge}"
        );
        assert!(
            ((right - left) - area.width()).abs() <= 3.0,
            "第 {row} 行宽度 {} 应为 {}",
            right - left,
            area.width()
        );
        rows.push((y, left, right));
        y += 4.0;
    }
    assert!(rows.len() >= 8, "只采样到 {} 行", rows.len());
    let middle = rows.iter().position(|(y, _, _)| *y >= mid_y).unwrap_or(0);
    let travel = |pick: fn(&(f32, f32, f32)) -> f32| -> f32 {
        rows.iter()
            .map(|row| (pick(row) - pick(&rows[middle])).abs())
            .fold(0.0_f32, f32::max)
    };
    let left_travel = travel(|row| row.1);
    let right_travel = travel(|row| row.2);
    println!("左边缘偏移 {left_travel:.1} / 右边缘偏移 {right_travel:.1}");
    assert!(
        left_travel > 4.0 && right_travel > 4.0,
        "两侧斜边没有连续: 左 {left_travel:.1} / 右 {right_travel:.1}"
    );
}
