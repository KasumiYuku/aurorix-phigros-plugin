use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, player::Player, sketch, songinfo};
use aurorix_phigros_plugin::domain::catalog::Catalog;
use aurorix_phigros_plugin::model::{grade_of, Difficulty};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/01-songinfo.png";

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

fn timestamp(text: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(text)
        .expect("时间戳")
        .with_timezone(&chrono::Utc)
}

fn make_level(diff: Difficulty, constant: f64, score: i32, acc: f64, rks: f64) -> songinfo::SongInfoLevel {
    let played = score > 0;
    let history = if played {
        vec![
            songinfo::SongInfoPoint {
                at: timestamp("2026-01-04T12:30:00Z"),
                score,
                acc,
                rks,
                grade: grade_of(score, false).name().to_string(),
                delta: 1200,
                first: false,
            },
            songinfo::SongInfoPoint {
                at: timestamp("2025-12-21T09:05:00Z"),
                score: score - 1200,
                acc: acc - 0.22,
                rks: rks - 0.02,
                grade: grade_of(score - 1200, false).name().to_string(),
                delta: 0,
                first: true,
            },
        ]
    } else {
        Vec::new()
    };
    songinfo::SongInfoLevel {
        diff: diff.name().to_string(),
        constant,
        has: constant > 0.0,
        played,
        score,
        acc,
        grade: if played {
            grade_of(score, false).name().to_string()
        } else {
            String::new()
        },
        rks,
        combo: 1200,
        changes: if history.is_empty() { 0 } else { 3 },
        history,
    }
}

fn build_request(catalog: &Catalog) -> songinfo::SongInfoRequest {
    let song = catalog
        .songs()
        .iter()
        .find(|song| {
            song.charts
                .iter()
                .filter(|chart| chart.constant > 0.0)
                .count()
                >= 3
        })
        .expect("曲库里没有三档以上的曲子");
    let mut levels = Vec::new();
    for (difficulty, score, acc, rks) in [
        (Difficulty::At, 996_233, 99.45, 16.12),
        (Difficulty::In, 982_001, 98.12, 15.31),
        (Difficulty::Hd, 0, 0.0, 0.0),
        (Difficulty::Ez, 0, 0.0, 0.0),
    ] {
        let chart = &song.charts[difficulty.index()];
        levels.push(make_level(
            difficulty,
            chart.constant,
            score,
            acc,
            rks,
        ));
    }
    songinfo::SongInfoRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        player: Player {
            has_player: true,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rank: 99,
            rank_level: 5,
            rks: 16.4231,
        },
        levels,
    }
}

fn render(req: &songinfo::SongInfoRequest) -> Vec<u8> {
    let root = assets_root();
    let fonts = cards::register_fonts(&root);
    assert!(fonts >= 1, "assets/font 下必须至少注册上一支字体");
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = songinfo::canvas_size(req);
    let request = cards::render_request(req);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            cards::with_device_scale(&kit, width, height, || songinfo::draw(&kit, &root, req));
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn layout_height_and_truncation() {
    let plan = songinfo::layout(&[1, 2]);
    assert_eq!(plan.tops, vec![142.0, 329.0]);
    assert_eq!(plan.heights, vec![167.0, 202.0]);
    assert_eq!(plan.total_height, 142.0 + 167.0 + 20.0 + 202.0 + 66.0);
    let request = songinfo::SongInfoRequest {
        song_id: String::new(),
        title: String::new(),
        composer: String::new(),
        player: Player::default(),
        levels: Vec::new(),
    };
    assert_eq!(songinfo::canvas_size(&request), (1046, 376));

    let measure_width = |text: &str| text.chars().count() as f32 * 12.0;
    assert_eq!(sketch::truncate_width("Phigros Song", 120.0, measure_width), "Phigros S…");
    assert_eq!(sketch::truncate_width("Phigros", 120.0, measure_width), "Phigros");
    assert_eq!(sketch::group_digits("1000000"), "1'000'000");
    assert_eq!(sketch::signed_score(-4200), "-4'200");
}

#[test]
#[ignore]
fn render_songinfo() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let catalog = Catalog::open(catalog_path()).expect("打开曲库");
    let request = build_request(&catalog);
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_FILE, &bytes).expect("写出 PNG");
    println!(
        "渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    assert!(
        PathBuf::from(OUT_FILE).is_file(),
        "PNG 必须落到 {OUT_FILE}"
    );

    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!("{OUT_FILE} {width}x{height}");
    assert_eq!(width, 1046, "画布宽必须与 Go 的设计宽一致");
    assert!(height > 200, "四档卡片的画布不该这么矮: {height}");

    let paper = pixel_at(&data, width, 4, 4);
    let paper_luma = u32::from(paper[0]) + u32::from(paper[1]) + u32::from(paper[2]);
    assert!(
        paper_luma > 20,
        "左上角应当是纸色而不是纯黑, 实际 {paper:?}"
    );
    assert_eq!(paper[3], 255, "画布必须不透明");

    for row in 0..28 {
        let mut line = String::from("  ");
        for col in 0..52 {
            let x = (col * (width as usize) / 52) as u32;
            let y = (row * (height as usize) / 28) as u32;
            let pixel = pixel_at(&data, width, x.min(width - 1), y.min(height - 1));
            let luma = (u32::from(pixel[0]) * 3 + u32::from(pixel[1]) * 5 + u32::from(pixel[2]) * 2) / 10;
            line.push(match luma {
                0..=32 => '.',
                33..=64 => ':',
                65..=96 => '-',
                97..=128 => '=',
                129..=170 => 'o',
                171..=210 => 'O',
                _ => '#',
            });
        }
        println!("{line}");
    }

    let mut min_v = 255u8;
    let mut max_v = 0u8;
    for x in 60..700 {
        for y in 6..22 {
            let pixel = pixel_at(&data, width, x, y);
            for channel in [pixel[0], pixel[1], pixel[2]] {
                min_v = min_v.min(channel);
                max_v = max_v.max(channel);
            }
        }
    }
    println!("页头曲绘底跨度 {}", max_v.saturating_sub(min_v));
    assert!(
        max_v.saturating_sub(min_v) >= 6,
        "页头那一片曲绘底必须有明显色差, 实际跨度 {}",
        max_v.saturating_sub(min_v)
    );

    let card_pixel = pixel_at(&data, width, 120, 330);
    let gap_pixel = pixel_at(&data, width, 120, 352);
    let luma = |pixel: [u8; 4]| {
        (u32::from(pixel[0]) * 3 + u32::from(pixel[1]) * 5 + u32::from(pixel[2]) * 2) / 10
    };
    println!("卡内 {:?} 卡缝 {:?}", card_pixel, gap_pixel);
    assert!(
        luma(gap_pixel) > luma(card_pixel),
        "第一张卡的玻璃墨必须压暗卡缝之下的底, 卡内 {} 卡缝 {}",
        luma(card_pixel),
        luma(gap_pixel)
    );

    let mut bright = 0usize;
    for x in 36..700 {
        for y in 24..64 {
            let pixel = pixel_at(&data, width, x, y);
            if pixel[0] > 200 && pixel[1] > 200 && pixel[2] > 200 {
                bright += 1;
            }
        }
    }
    assert!(bright > 40, "曲名那行白字没有画上去, 只有 {bright} 个亮点");
}
