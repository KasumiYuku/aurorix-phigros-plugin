use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, player::Player, sketch, sync};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_FILE: &str = "/tmp/phigros-verify/06-sync.png";

fn assets_root() -> PathBuf {
    aurorix_phigros_plugin::assets_root()
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

fn player() -> Player {
    Player {
        has_player: true,
        name: String::from("Aurorix"),
        avatar: String::from("Introduction"),
        rank: 99,
        rank_level: 5,
        rks: 16.4231,
    }
}

struct Bench {
    song_id: &'static str,
    title: &'static str,
    diff: &'static str,
    constant: f64,
    old_score: i32,
    new_score: i32,
    old_acc: f64,
    new_acc: f64,
    rks: f64,
    grade: &'static str,
    in_best: bool,
    rks_delta: f64,
}

const BENCH: [Bench; 10] = [
    Bench {
        song_id: "Spasmodic.姜米條颶風元力上人.0",
        title: "Spasmodic",
        diff: "AT",
        constant: 16.4,
        old_score: 1_001_233,
        new_score: 1_002_916,
        old_acc: 99.53,
        new_acc: 99.71,
        rks: 16.31,
        grade: "PHI",
        in_best: true,
        rks_delta: 0.0241,
    },
    Bench {
        song_id: "ExoplanetaryMirage.かめりあ.0",
        title: "Exoplanetary Mirage",
        diff: "IN",
        constant: 15.8,
        old_score: 986_204,
        new_score: 994_871,
        old_acc: 98.11,
        new_acc: 99.02,
        rks: 15.74,
        grade: "V",
        in_best: true,
        rks_delta: 0.0312,
    },
    Bench {
        song_id: "Glaciaxion.SunsetRay.0",
        title: "Glaciaxion",
        diff: "IN",
        constant: 14.6,
        old_score: 974_331,
        new_score: 974_331,
        old_acc: 97.42,
        new_acc: 97.42,
        rks: 14.12,
        grade: "S",
        in_best: false,
        rks_delta: 0.0,
    },
    Bench {
        song_id: "Cthugha.USAO.0",
        title: "Cthugha",
        diff: "IN",
        constant: 16.1,
        old_score: 963_880,
        new_score: 1_001_334,
        old_acc: 96.88,
        new_acc: 99.62,
        rks: 16.02,
        grade: "V",
        in_best: true,
        rks_delta: 0.0584,
    },
    Bench {
        song_id: "Igallta.SeURa.0",
        title: "Igallta",
        diff: "AT",
        constant: 16.3,
        old_score: 955_127,
        new_score: 961_744,
        old_acc: 95.41,
        new_acc: 96.12,
        rks: 15.38,
        grade: "V",
        in_best: false,
        rks_delta: 0.0092,
    },
    Bench {
        song_id: "Rrharil.TeamGrimoire.0",
        title: "Rrhar'il",
        diff: "IN",
        constant: 15.9,
        old_score: 1_001_001,
        new_score: 1_001_820,
        old_acc: 99.61,
        new_acc: 99.68,
        rks: 15.88,
        grade: "PHI",
        in_best: true,
        rks_delta: 0.0071,
    },
    Bench {
        song_id: "Aphasia.闫东炜.0",
        title: "Aphasia",
        diff: "AT",
        constant: 15.4,
        old_score: 942_666,
        new_score: 938_512,
        old_acc: 94.28,
        new_acc: 93.87,
        rks: 14.44,
        grade: "S",
        in_best: false,
        rks_delta: -0.0048,
    },
    Bench {
        song_id: "Stasis.Maozon.0",
        title: "Stasis",
        diff: "IN",
        constant: 14.4,
        old_score: 931_412,
        new_score: 941_006,
        old_acc: 93.11,
        new_acc: 94.02,
        rks: 13.72,
        grade: "A",
        in_best: false,
        rks_delta: 0.0117,
    },
    Bench {
        song_id: "DistortedFate.Sakuzyo.0",
        title: "Distorted Fate",
        diff: "HD",
        constant: 12.7,
        old_score: 900_411,
        new_score: 912_338,
        old_acc: 90.04,
        new_acc: 91.22,
        rks: 11.94,
        grade: "A",
        in_best: false,
        rks_delta: 0.0062,
    },
    Bench {
        song_id: "KhronostasisKatharsis.Halv.0",
        title: "Khronostasis Katharsis",
        diff: "EZ",
        constant: 8.3,
        old_score: 862_004,
        new_score: 866_771,
        old_acc: 86.20,
        new_acc: 86.68,
        rks: 8.11,
        grade: "B",
        in_best: false,
        rks_delta: 0.0009,
    },
];

fn build_item(bench: &Bench) -> sync::SyncItem {
    let delta = bench.new_score - bench.old_score;
    sync::SyncItem {
        song_id: String::from(bench.song_id),
        title: String::from(bench.title),
        difficulty: String::from(bench.diff),
        constant: bench.constant,
        known: true,
        old_score: bench.old_score,
        new_score: bench.new_score,
        old_acc: bench.old_acc,
        new_acc: bench.new_acc,
        delta,
        rks: bench.rks,
        grade: String::from(bench.grade),
        in_best: bench.in_best,
        rks_delta: bench.rks_delta,
    }
}

fn build_request() -> sync::SyncRequest {
    let mut first = Vec::new();
    for bench in &BENCH[..6] {
        first.push(build_item(bench));
    }
    let mut second = Vec::new();
    for bench in &BENCH[6..] {
        second.push(build_item(bench));
    }
    sync::SyncRequest {
        player: player(),
        backdrop: String::from(BENCH[0].song_id),
        at: Some(timestamp("2026-01-04T12:30:00Z")),
        save_modified: Some(timestamp("2026-01-03T21:04:00Z")),
        first_sync: false,
        changed: true,
        rks_before: 15.9842,
        rks_after: 16.0725,
        groups: vec![
            sync::SyncGroup {
                at: None,
                label: String::from(""),
                items: first,
                total: 14,
            },
            sync::SyncGroup {
                at: Some(timestamp("2026-01-03T21:04:00Z")),
                label: String::from(""),
                items: second,
                total: 4,
            },
        ],
        total: 349,
        played: 286,
        charts: 349,
        recent7: 0.0413,
        has_recent: true,
    }
}

fn render(req: &sync::SyncRequest) -> Vec<u8> {
    render_as(req, aurorix_render::RenderFormat::Png, 90)
}

fn render_delivered(req: &sync::SyncRequest) -> Vec<u8> {
    render_as(
        req,
        aurorix_render::RenderFormat::Jpeg,
        aurorix_render::RenderRequest::DEFAULT_QUALITY,
    )
}

fn render_as(
    req: &sync::SyncRequest,
    format: aurorix_render::RenderFormat,
    quality: u8,
) -> Vec<u8> {
    let root = assets_root();
    let fonts = cards::register_fonts(&root);
    assert!(
        fonts >= 1 || aurorix_render::has_font_family(sketch::FONT_BODY),
        "assets/font 下必须至少注册上一支字体"
    );
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let (width, height) = sync::canvas_size(req);
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(format)
        .with_quality(quality);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            sync::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
fn canvas_size_follows_the_group_plan() {
    let shaped = |counts: &[usize]| -> sync::SyncRequest {
        let mut request = build_request();
        request.groups = counts
            .iter()
            .map(|count| sync::SyncGroup {
                at: None,
                label: String::new(),
                items: (0..*count).map(|_| build_item(&BENCH[0])).collect(),
                total: *count as i32,
            })
            .collect();
        request
    };
    let (width, height) = sync::canvas_size(&shaped(&[2, 2]));
    println!("2 组 x 2/2 张 -> {width}x{height}");
    assert_eq!((width, height), (1440, 634));
    let (width, height) = sync::canvas_size(&shaped(&[10, 10]));
    println!("2 组 x 10/10 张 -> {width}x{height}");
    assert_eq!((width, height), (1440, 960));
    let (width, height) = sync::canvas_size(&shaped(&[24, 10]));
    println!("2 组 x 24/10 张 (第二组截到 10) -> {width}x{height}");
    assert_eq!((width, height), (1440, 960));
    let (width, height) = sync::canvas_size(&sync::SyncRequest::default());
    println!("空请求 -> {width}x{height}");
    assert_eq!((width, height), (1440, 292));
    let (width, height) = sync::canvas_size(&build_request());
    println!("样板请求 2 组 x 6/4 张 -> {width}x{height}");
    assert_eq!(width, 1440);
    assert!(height > 634 && height < 960);
}

#[test]
#[ignore]
fn render_sync() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let request = build_request();
    println!(
        "分组 {} 组 / 卡片 {} 张",
        request.groups.len(),
        request
            .groups
            .iter()
            .map(|group| group.items.len())
            .sum::<usize>()
    );
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_FILE, &bytes).expect("写出 PNG");
    let delivered = render_delivered(&request);
    let delivered_path = OUT_FILE.replace(".png", ".jpg");
    fs::write(&delivered_path, &delivered).expect("写出 JPEG");
    println!("交付图 {delivered_path} {} 字节", delivered.len());
    println!(
        "渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    assert!(PathBuf::from(OUT_FILE).is_file(), "PNG 必须落到 {OUT_FILE}");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!("{OUT_FILE} {width}x{height}");
    let (want_w, want_h) = sync::canvas_size(&request);
    assert_eq!(width, want_w as u32, "画布宽必须与 canvas_size 一致");
    assert_eq!(height, want_h as u32, "画布高必须与 canvas_size 一致");
    let spread = variance(&data);
    println!("像素方差 {spread:.2}");
    assert!(spread > 100.0, "画布不该是一片纯色, 方差 {spread:.2}");
}

#[test]
#[ignore]
fn bench_sync() {
    let root = assets_root();
    cards::register_fonts(&root);
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let request = build_request();
    let (width, height) = sync::canvas_size(&request);
    let mut samples = Vec::new();
    for round in 0..7 {
        let started = Instant::now();
        let bytes = context
            .draw(
                aurorix_render::RenderRequest::new(width, height)
                    .with_format(aurorix_render::RenderFormat::Jpeg)
                    .with_quality(aurorix_render::RenderRequest::DEFAULT_QUALITY),
                |canvas, ctx| {
                    let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
                    sync::draw(&kit, &root, &request);
                },
            )
            .expect("渲染失败")
            .to_vec();
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        if round == 0 {
            println!(
                "SYNC 冷启动首帧 {ms:.0} ms / {} 字节 ({width}x{height})",
                bytes.len()
            );
        }
        samples.push(ms);
    }
    samples.remove(0);
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    println!(
        "SYNC 热态 min {:.0} / 中位 {:.0} / max {:.0} ms",
        samples[0],
        samples[samples.len() / 2],
        samples[samples.len() - 1]
    );
}
