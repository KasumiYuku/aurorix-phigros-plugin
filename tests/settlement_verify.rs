use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, player::Player, settlement, sketch};
use aurorix_phigros_plugin::domain::catalog::Catalog;
use aurorix_phigros_plugin::model::{grade_of, Difficulty};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/03-settlement.png";

fn assets_root() -> PathBuf {
    aurorix_phigros_plugin::assets_root()
}

fn catalog_path() -> PathBuf {
    let root = assets_root();
    let mut candidates = vec![
        root.join("data").join("phi.db"),
        root.join("../data").join("phi.db"),
    ];
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

fn build_request(catalog: &Catalog) -> settlement::SettlementRequest {
    let song = catalog
        .songs()
        .iter()
        .find(|song| song.difficulties().len() == 4)
        .expect("曲库里没有四档齐全的曲子");
    let diff = Difficulty::In;
    let constant = song.constant(diff).expect("IN 档定数");
    let score_value = 996_233;
    let mut scores = Vec::new();
    for difficulty in Difficulty::playable_all() {
        let chart = &song.charts[difficulty.index()];
        let played = chart.known();
        scores.push(settlement::DifficultyScore {
            diff: difficulty.name().to_string(),
            played,
            score: if played {
                900_000 + difficulty.index() as i32 * 21_000
            } else {
                0
            },
            acc: if played {
                96.0 + difficulty.index() as f64
            } else {
                0.0
            },
            grade: if played {
                grade_of(900_000, false).name().to_string()
            } else {
                String::new()
            },
            fc: played,
            phi: false,
            rks: if played {
                12.0 + difficulty.index() as f64
            } else {
                0.0
            },
            rank: if played {
                40 + difficulty.index() as i32
            } else {
                0
            },
            total: if played { 4_120 } else { 0 },
        });
    }
    settlement::SettlementRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        diff: diff.name().to_string(),
        constant,
        known: true,
        played: true,
        score: score_value,
        acc: 99.45,
        history: 990_614,
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
    }
}

fn render(req: &settlement::SettlementRequest) -> Vec<u8> {
    let root = assets_root();
    let fonts = cards::register_fonts(&root);
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = settlement::canvas_size();
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(aurorix_render::RenderFormat::Png);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            settlement::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn layout_and_labels_match_go() {
    assert_eq!(settlement::canvas_size(), (1920, 886));
    assert_eq!(settlement::score_text(996_233), "0996233");
    assert_eq!(
        settlement::history_text(996_233, 990_614),
        "NEW BEST 0990614 +0005619"
    );
    assert_eq!(
        settlement::history_text(900_000, 995_307),
        "NEW BEST 0995307 -0095307"
    );
    assert_eq!(settlement::level_text("IN", 15.0, true), "IN Lv.15");
    assert_eq!(settlement::level_text("IN", 15.7, true), "IN Lv.15.7");
    assert_eq!(settlement::level_text("AT", 0.0, false), "AT 未收录");
    assert_eq!(settlement::acc_text(99.1234), "99.12%");
    assert_eq!(settlement::rank_place_text(3, 312), "#3 / 312");
    assert_eq!(settlement::gain_text(34_120), "+34'120");
    let cards = settlement::settle_cards();
    assert_eq!(cards.backs.len(), 3);
    assert!(cards.score.bottom() < cards.combo_and_acc.top());
    assert!(cards.combo_and_acc.bottom() < cards.more_info.top());
    assert!(cards.ratings.left() > cards.score.left());
    let slots = settlement::stat_text(cards.combo_and_acc);
    assert!(slots.right_value.x < cards.combo_and_acc.right());
    assert!(slots.left_value.size > slots.left_label.size);
    let bar = settlement::gained();
    assert!(bar.top() > cards.more_info.bottom());
    assert!(settlement::gained_curve_rect().height() > 0.0);
}

#[test]
#[ignore]
fn render_settlement() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let request = build_request(&catalog);
    println!("曲目 {} / {}", request.song_id, request.title);
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
}
