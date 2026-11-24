use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use aurorix_phigros_plugin::cards::{self, b30, player::Player, sketch};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{CanvasKit, RenderContext, SkiaBackend};

const OUT_DIR: &str = "/tmp/phigros-verify";

const OUT_B30: &str = "/tmp/phigros-verify/07-b30.png";

const OUT_B50: &str = "/tmp/phigros-verify/08-b50.png";

const PARITY_OUT: &str = "/tmp/phigros-verify/rust-b30-parity.png";

const SONGS: [(&str, &str, f64); 5] = [
    ("Spasmodic.姜米條颶風元力上人.0", "Spasmodic", 15.7),
    ("ExoplanetaryMirage.かめりあ.0", "Exoplanetary Mirage", 15.9),
    ("Glaciaxion.SunsetRay.0", "Glaciaxion", 14.6),
    ("Igallta.SeURa.0", "Igallta", 16.2),
    ("Rrharil.TeamGrimoire.0", "Rrhar'il", 16.0),
];

const GRADES: [&str; 5] = ["phi", "V", "FC", "S", "A"];

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
        let luma =
            (f64::from(pixel[0]) * 3.0 + f64::from(pixel[1]) * 5.0 + f64::from(pixel[2]) * 2.0)
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

fn row_variance(data: &[u8], width: u32, height: u32, area: aurorix_render::skia_safe::Rect) -> f64 {
    let x0 = area.left().max(0.0) as u32;
    let y0 = area.top().max(0.0) as u32;
    let x1 = (area.right() as u32).min(width.saturating_sub(1));
    let y1 = (area.bottom() as u32).min(height.saturating_sub(1));
    let mut count = 0.0_f64;
    let mut sum = 0.0_f64;
    let mut square = 0.0_f64;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let at = (y as usize * width as usize + x as usize) * 4;
            let luma = (f64::from(data[at]) * 3.0
                + f64::from(data[at + 1]) * 5.0
                + f64::from(data[at + 2]) * 2.0)
                / 10.0;
            count += 1.0;
            sum += luma;
            square += luma * luma;
        }
    }
    if count <= 0.0 {
        return 0.0;
    }
    (square / count - (sum / count).powi(2)).max(0.0)
}

fn area_mean(data: &[u8], width: u32, height: u32, area: aurorix_render::skia_safe::Rect) -> f64 {
    let x0 = area.left().max(0.0) as u32;
    let y0 = area.top().max(0.0) as u32;
    let x1 = (area.right() as u32).min(width.saturating_sub(1));
    let y1 = (area.bottom() as u32).min(height.saturating_sub(1));
    let mut count = 0.0_f64;
    let mut sum = 0.0_f64;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let at = (y as usize * width as usize + x as usize) * 4;
            sum += (f64::from(data[at]) * 3.0
                + f64::from(data[at + 1]) * 5.0
                + f64::from(data[at + 2]) * 2.0)
                / 10.0;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        return 0.0;
    }
    sum / count
}

fn entry(index: i32, podium: bool) -> b30::B30Entry {
    let (id, title, constant) = SONGS[index as usize % SONGS.len()];
    b30::B30Entry {
        rank: index + 1,
        song_id: String::from(id),
        slot: false,
        title: String::from(title),
        diff: String::from("IN"),
        constant,
        known: true,
        score: 999_500 - index * 731,
        acc: 99.89 - f64::from(index) * 0.07,
        rks: (constant + 2.0 - f64::from(index) * 0.01).clamp(0.0, 99.0),
        suggest: 99.93,
        suggest_ok: index % 3 == 0,
        grade: String::from(GRADES[index as usize % GRADES.len()]),
        podium,
        cover_luma: 120,
    }
}

fn build_request(count: i32, entries: i32, podium: i32) -> b30::B30Request {
    let mut items = Vec::with_capacity(entries as usize);
    for index in 0..entries {
        items.push(entry(index, index < podium));
    }
    let wide = count > 30;
    b30::B30Request {
        player: Player {
            has_player: true,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rank: 99,
            rank_level: 5,
            rks: 16.4231,
        },
        count,
        entries: items,
        rks: 16.4231,
        computed: 15.8821,
        records: 187,
        avg: 15.4211,
        avg_label: String::from("B27 平均"),
        phi_avg: 15.0,
        best_avg: 15.42,
        best_count: 27,
        phi: 12,
        unknown: 2,
        suspect: 1,
        synced: String::from("2026-01-04 12:30"),
        title: if wide {
            String::from("BEST 50")
        } else {
            String::from("BEST 30")
        },
        list_noun: if wide {
            String::from("谱面")
        } else {
            String::new()
        },
        no_tier: false,
        backdrop: String::from("Spasmodic.姜米條颶風元力上人.0"),
        diffs: vec![
            b30::B30DiffStat {
                label: String::from("EZ"),
                played: 41,
                total: 62,
            },
            b30::B30DiffStat {
                label: String::from("HD"),
                played: 58,
                total: 62,
            },
            b30::B30DiffStat {
                label: String::from("IN"),
                played: 61,
                total: 62,
            },
            b30::B30DiffStat {
                label: String::from("AT"),
                played: 7,
                total: 19,
            },
        ],
    }
}

fn render(req: &b30::B30Request) -> Vec<u8> {
    render_as(req, aurorix_render::RenderFormat::Png, 90)
}

fn render_delivered(req: &b30::B30Request) -> Vec<u8> {
    render_as(
        req,
        aurorix_render::RenderFormat::Jpeg,
        aurorix_render::RenderRequest::DEFAULT_QUALITY,
    )
}

fn render_as(
    req: &b30::B30Request,
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
    let (width, height) = b30::canvas_size(req);
    let request = aurorix_render::RenderRequest::new(width, height)
        .with_format(format)
        .with_quality(quality);
    context
        .draw(request, |canvas, ctx| {
            let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
            b30::draw(&kit, &root, req);
        })
        .expect("渲染失败")
        .to_vec()
}

#[test]
#[ignore]
fn render_b30() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let request = build_request(30, 30, 3);
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_B30, &bytes).expect("写出 PNG");
    println!(
        "B30 渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    let delivered = render_delivered(&request);
    let delivered_path = OUT_B30.replace(".png", ".jpg");
    fs::write(&delivered_path, &delivered).expect("写出 JPEG");
    println!("交付图 {delivered_path} {} 字节", delivered.len());
    assert!(PathBuf::from(OUT_B30).is_file(), "PNG 必须落到 {OUT_B30}");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!("{OUT_B30} {width}x{height}");
    assert_eq!(width, 1500, "画布宽必须与 Go 的 B30OutW 一致");
    assert_eq!(height, 2690, "画布高必须与 Go 的 B30OutSize 一致");
    let spread = variance(&data);
    println!("像素方差 {spread:.2}");
    assert!(spread > 100.0, "画布不该是一片纯色, 方差 {spread:.2}");
    let first = b30::card_rect(0, 30, 10);
    let card = row_variance(&data, width, height, first);
    println!(
        "首卡 {}x{} @ ({}, {}) 方差 {card:.2}",
        first.width(),
        first.height(),
        first.left(),
        first.top()
    );
    assert!(card > 60.0, "首卡里没有内容: 方差 {card:.2}");
    let probe = |tag: &str, area: aurorix_render::skia_safe::Rect| {
        let mean = area_mean(&data, width, height, area);
        let flat = row_variance(&data, width, height, area);
        println!("{tag}: 均亮度 {mean:.1} 方差 {flat:.1}");
        assert!(flat > 100.0, "{tag} 没有画出内容: 方差 {flat:.1}");
    };
    probe(
        "标题块",
        aurorix_render::skia_safe::Rect::from_xywh(74.0, 40.0, 640.0, 76.0),
    );
    probe(
        "进度条",
        aurorix_render::skia_safe::Rect::from_xywh(74.0, 160.0, 1118.4, 46.0),
    );
    probe(
        "玩家条",
        aurorix_render::skia_safe::Rect::from_xywh(1250.0, 20.0, 240.0, 110.0),
    );
    probe(
        "页脚签名",
        aurorix_render::skia_safe::Rect::from_xywh(700.0, 2620.0, 700.0, 40.0),
    );
}

#[test]
#[ignore]
fn render_b50() {
    fs::create_dir_all(OUT_DIR).expect("建出图目录");
    let request = build_request(50, 50, 3);
    let start = Instant::now();
    let bytes = render(&request);
    let elapsed = start.elapsed();
    fs::write(OUT_B50, &bytes).expect("写出 PNG");
    let delivered = render_delivered(&request);
    let delivered_path = OUT_B50.replace(".png", ".jpg");
    fs::write(&delivered_path, &delivered).expect("写出 JPEG");
    println!("交付图 {delivered_path} {} 字节", delivered.len());
    println!(
        "B50 渲染耗时 {} ms ({} 字节)",
        elapsed.as_millis(),
        bytes.len()
    );
    assert!(PathBuf::from(OUT_B50).is_file(), "PNG 必须落到 {OUT_B50}");
    let (width, height, data) = decode_png(&bytes).expect("解码头");
    println!("{OUT_B50} {width}x{height}");
    assert_eq!(width, 1920, "画布宽必须与 Go 的 B30OutW 全宽一致");
    assert_eq!(height, 2170, "画布高必须与 Go 的 B30OutSize 一致");
    let spread = variance(&data);
    println!("像素方差 {spread:.2}");
    assert!(spread > 100.0, "画布不该是一片纯色, 方差 {spread:.2}");
    let first = b30::card_rect(0, 50, 0);
    let card = row_variance(&data, width, height, first);
    println!(
        "首卡 {}x{} @ ({}, {}) 方差 {card:.2}",
        first.width(),
        first.height(),
        first.left(),
        first.top()
    );
    assert!(card > 60.0, "首卡里没有内容: 方差 {card:.2}");
}

const GO_IDS: [&str; 30] = [
    "DesultorySignals.technoplanet.0",
    "ExoplanetaryMirage.かめりあ.0",
    "ハテ.rNFrums.0",
    "EntrancetotheChaos.打打だいずvssiromaru.0",
    "Rrharil.TeamGrimoire.0",
    "DerSchneid.Ωμεγα.0",
    "Petrichor.voidMournfinale.0",
    "DistortedFate.Sakuzyo.0",
    "Igallta.SeURa.0",
    "QZKagoRequiem.tpazolite.0",
    "DESTRUCTION321.Normal1zervsBrokenNerdz.0",
    "ERABYECONNEC10N.かめりあ.0",
    "AbsoluTedisoRdeR.AcuteDisarray.0",
    "StardustRAY.kanonevsBlackY.0",
    "Hydra.JamesLandinoXAkiraComplex.0",
    "幻影鬼魅PLEASE.R300K.0",
    "玩具狂奏曲終焉.きくお.0",
    "Archidoxen.Se_IRA.0",
    "DerRichter.Ωμεγα.0",
    "INFiNiTEENERZYOverdoze.RekuMochizuki.0",
    "ReEndofaDream.umavsモリモリあつし.0",
    "BANGINGSTRIKE.DewPleiades.0",
    "slips.rintarosoma.0",
    "Cuvism.Fl00tvsHalv.0",
    "DiamondDust.MasahiroGodspeedAoki.0",
    "Spasmodic.姜米條颶風元力上人.0",
    "ATHAZA.LeaF.0",
    "AvataarReincarnationofKalpa.ScarletteakaCrYmson.0",
    "PANICPARADISE.DJSHIONY.0",
    "volcanic.DETROakaルゼ.0",
];

const GO_TITLES: [&str; 30] = [
    "Desultory Signals",
    "Exoplanetary Mirage",
    "ハテ",
    "Entrance to the Chaos",
    "Rrhar'il",
    "Der Schneid",
    "Petrichor",
    "Distorted Fate",
    "Igallta",
    "QZKago Requiem",
    "DESTRUCTION 3,2,1",
    "+ERABY+E CONNEC+10N",
    "AbsoluTe disoRdeR",
    "Stardust:RAY",
    "Hydra",
    "幻影鬼魅 (PLEASE)",
    "玩具狂奏曲 -終焉-",
    "Archidoxen",
    "Der Richter",
    "INFiNiTE ENERZY -Overdoze-",
    "Re：End of a Dream",
    "BANGING STRIKE",
    "sølips",
    "Cuvism³",
    "Diamond Dust",
    "Spasmodic",
    "ATHAZA",
    "Avataar ~Reincarnation of Kalpa~",
    "PANIC PARADISE",
    "volcanic",
];

const GO_CONSTANTS: [f64; 30] = [
    18.0, 17.9, 17.9, 17.6, 17.6, 17.5, 17.5, 17.4, 17.4, 17.4, 17.3, 17.3, 17.2, 17.2, 17.1,
    17.0, 17.0, 16.9, 16.9, 16.9, 16.9, 16.8, 16.8, 16.7, 16.7, 16.7, 16.6, 16.6, 16.6, 16.6,
];

fn go_sample_request() -> b30::B30Request {
    let mut entries = Vec::with_capacity(36);
    for index in 0..36_i64 {
        let slot = (index as usize) % GO_IDS.len();
        let mut item = b30::B30Entry {
            rank: index as i32 + 1,
            song_id: String::from(GO_IDS[slot]),
            slot: false,
            title: String::from(GO_TITLES[slot]),
            diff: String::from("AT"),
            constant: GO_CONSTANTS[slot],
            known: true,
            score: 1_000_000 - index as i32 * 4_211,
            acc: ((99.87 - index as f64 * 0.13) * 100.0).trunc() / 100.0,
            rks: GO_CONSTANTS[slot] + 2.15 - index as f64 * 0.05,
            suggest: 0.0,
            suggest_ok: false,
            grade: String::from("V"),
            podium: false,
            cover_luma: 120,
        };
        if index < 3 {
            item.score = 1_000_000;
            item.acc = 100.0;
            item.podium = true;
            item.grade = String::from("PHI");
        }
        entries.push(item);
    }
    b30::B30Request {
        player: Player {
            has_player: true,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rank: 46,
            rank_level: 3,
            rks: 15.87,
        },
        count: 30,
        entries,
        rks: 15.87,
        computed: 15.83,
        records: 87,
        avg: 15.23,
        avg_label: String::from("B27 平均"),
        phi_avg: 16.42,
        best_avg: 15.23,
        best_count: 27,
        phi: 6,
        unknown: 0,
        suspect: 0,
        synced: String::from("2026-10-01 15:04"),
        title: String::from("BEST 30"),
        list_noun: String::new(),
        no_tier: false,
        backdrop: String::from("DesultorySignals.technoplanet.0"),
        diffs: vec![
            b30::B30DiffStat { label: String::from("EZ"), played: 298, total: 300 },
            b30::B30DiffStat { label: String::from("HD"), played: 173, total: 312 },
            b30::B30DiffStat { label: String::from("IN"), played: 244, total: 320 },
            b30::B30DiffStat { label: String::from("AT"), played: 31, total: 152 },
        ],
    }
}

#[test]
#[ignore]
fn render_b30_go_sample() {
    let req = go_sample_request();
    let started = Instant::now();
    let bytes = render(&req);
    let elapsed = started.elapsed().as_millis();
    fs::write(PARITY_OUT, &bytes).expect("写入失败");
    let size = b30::canvas_size(&req);
    println!("{PARITY_OUT} {size:?} 耗时 {elapsed} ms {} 字节", bytes.len());
    assert_eq!(size, (1500, 3220));
}

#[test]
#[ignore]
fn bench_b30_b50() {
    let root = assets_root();
    cards::register_fonts(&root);
    let assets = Arc::new(aurorix_phigros_plugin::assets_in(&root));
    let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
        .with_asset_loader(assets.loader());
    let cases = [
        (String::from("B30"), build_request(30, 30, 3)),
        (String::from("B50"), build_request(50, 50, 3)),
    ];
    let mut total = 0.0;
    for (label, request) in cases {
        let (width, height) = b30::canvas_size(&request);
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
                        b30::draw(&kit, &root, &request);
                    },
                )
                .expect("渲染失败")
                .to_vec();
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            if round == 0 {
                println!(
                    "{label} 冷启动首帧 {ms:.0} ms / {} 字节 ({width}x{height})",
                    bytes.len()
                );
            }
            samples.push(ms);
        }
        samples.remove(0);
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = samples[samples.len() / 2];
        total += median;
        println!(
            "{label} 热态 min {:.0} / 中位 {median:.0} / max {:.0} ms",
            samples[0],
            samples[samples.len() - 1]
        );
    }
    println!("B30+B50 单轮中位和 {total:.0} ms");
}
