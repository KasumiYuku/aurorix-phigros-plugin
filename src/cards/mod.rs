use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use aurorix_plugin::RenderCard;
use aurorix_render::RenderRequest;

pub mod b30;
pub mod chapter;
pub mod chapterdetail;
pub mod guess;
pub mod list;
pub mod memo;
pub mod overview;
pub mod player;
pub mod score;
pub mod search;
pub mod settlement;
pub mod sketch;
pub mod song;
pub mod songinfo;
pub mod songselect;
pub mod stats;
pub mod suggest;
pub mod sync;
pub mod table;

pub const CARD_WIDTH: i32 = 1046;

pub const ILLUST_DIR: &str = "illust";

pub const BLUR_DIR: &str = "blur";

pub const THUMB_DIR: &str = "thumb";

pub const THUMB_SMALL_DIR: &str = "thumb_s";

pub const AVATAR_DIR: &str = "avatar";

pub const UI_DIR: &str = "ui";

pub const FONT_DIR: &str = "font";

pub const SONG_DETAIL_LABEL: &str = "曲目详情";

#[derive(Debug, Default, serde::Deserialize)]
struct UiEntry {
    #[serde(default)]
    file: String,
}

#[derive(Debug, Default, serde::Deserialize)]
struct FontEntry {
    #[serde(default)]
    file: String,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Manifest {
    #[serde(default)]
    illust: HashMap<String, String>,
    #[serde(default)]
    blur: HashMap<String, String>,
    #[serde(default)]
    avatar: HashMap<String, String>,
    #[serde(default)]
    ui: HashMap<String, UiEntry>,
    #[serde(default)]
    font: HashMap<String, FontEntry>,
}

static MANIFEST_TABLE: OnceLock<Mutex<HashMap<String, Arc<Manifest>>>> = OnceLock::new();

fn manifest_table_lock() -> MutexGuard<'static, HashMap<String, Arc<Manifest>>> {
    MANIFEST_TABLE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn load_manifest(root: &Path) -> Arc<Manifest> {
    let key = root.to_string_lossy().to_string();
    if let Some(hit) = manifest_table_lock().get(&key).cloned() {
        return hit;
    }
    let loaded = std::fs::read_to_string(root.join("manifest.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Manifest>(&raw).ok())
        .unwrap_or_default();
    let fresh = Arc::new(loaded);
    let mut table = manifest_table_lock();
    table
        .entry(key)
        .or_insert_with(|| Arc::clone(&fresh))
        .clone()
}

pub fn convention_key(dir: &str, name: &str, ext: &str) -> String {
    format!("{dir}/{name}.{ext}")
}

pub fn swap_extension(key: &str) -> Option<String> {
    if let Some(stem) = key.strip_suffix(".jpg") {
        return Some(format!("{stem}.png"));
    }
    if let Some(stem) = key.strip_suffix(".png") {
        return Some(format!("{stem}.jpg"));
    }
    None
}

pub fn locate(root: &Path, key: &str) -> String {
    let (dir, name) = match key.split_once('/') {
        Some((kind, name)) => (kind, name),
        None => (key, ""),
    };
    let manifest = load_manifest(root);
    let mut relative = match dir {
        ILLUST_DIR => manifest.illust.get(name).cloned().unwrap_or_default(),
        BLUR_DIR => manifest.blur.get(name).cloned().unwrap_or_default(),
        AVATAR_DIR => manifest.avatar.get(name).cloned().unwrap_or_default(),
        UI_DIR => manifest
            .ui
            .get(name)
            .map(|item| item.file.clone())
            .unwrap_or_default(),
        _ => String::new(),
    };
    if relative.is_empty() {
        relative = convention_key(dir, name, "jpg");
    }
    if root.join(&relative).is_file() {
        return relative;
    }
    if let Some(alt) = swap_extension(&relative) {
        if root.join(&alt).is_file() {
            return alt;
        }
    }
    relative
}

pub fn illust_key(root: &Path, id: &str) -> String {
    locate(root, &format!("{ILLUST_DIR}/{id}"))
}

pub fn blur_key(root: &Path, id: &str) -> String {
    locate(root, &format!("{BLUR_DIR}/{id}"))
}

pub fn thumb_key(root: &Path, id: &str) -> String {
    locate(root, &format!("{THUMB_DIR}/{id}"))
}

pub fn thumb_small_key(root: &Path, id: &str) -> String {
    locate(root, &format!("{THUMB_SMALL_DIR}/{id}"))
}

pub fn avatar_asset(root: &Path, name: &str) -> String {
    locate(root, &format!("{AVATAR_DIR}/{name}"))
}

pub fn ui_asset(root: &Path, name: &str) -> String {
    locate(root, &format!("{UI_DIR}/{name}"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontItem {
    pub name: String,
    pub path: PathBuf,
}

pub fn font_list(root: &Path) -> Vec<FontItem> {
    let manifest = load_manifest(root);
    let mut out: Vec<FontItem> = Vec::new();
    if !manifest.font.is_empty() {
        for (name, item) in manifest.font.iter() {
            let path = root.join(&item.file);
            if item.file.is_empty() || !path.is_file() {
                continue;
            }
            out.push(FontItem {
                name: name.clone(),
                path,
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        return out;
    }
    let Ok(entries) = std::fs::read_dir(root.join(FONT_DIR)) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let extension = path
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if !matches!(extension.as_str(), "ttf" | "otf" | "ttc") {
            continue;
        }
        let Some(stem) = path.file_stem() else {
            continue;
        };
        out.push(FontItem {
            name: stem.to_string_lossy().to_string(),
            path,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

pub fn register_fonts(root: &Path) -> usize {
    let mut registered = 0;
    for item in font_list(root) {
        if aurorix_render::has_font_family(&item.name) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&item.path) else {
            continue;
        };
        if aurorix_render::register_font_family(&item.name, bytes, None).is_ok() {
            registered += 1;
        }
    }
    registered
}

pub const MAX_OUTPUT_WIDTH: i32 = 1680;

pub const MAX_OUTPUT_PIXELS: f64 = 13_500_000.0;

pub const QUALITY_CARD: u8 = 92;

pub const QUALITY_MID: u8 = 90;

pub const QUALITY_LONG: u8 = 85;

pub fn device_scale(width: i32, height: i32) -> f32 {
    let width = width.max(1);
    let height = height.max(1);
    let by_width = MAX_OUTPUT_WIDTH as f32 / width as f32;
    let by_pixels = (MAX_OUTPUT_PIXELS / (width as f64 * height as f64)).sqrt() as f32;
    by_width.min(by_pixels).clamp(0.6, 1.0)
}

pub fn device_size(width: i32, height: i32) -> (i32, i32) {
    let factor = device_scale(width, height);
    if (factor - 1.0).abs() < 1e-4 {
        return (width.max(1), height.max(1));
    }
    (
        ((width as f32 * factor).round() as i32).max(1),
        ((height as f32 * factor).round() as i32).max(1),
    )
}

pub fn quality_for(width: i32, height: i32) -> u8 {
    let pixels = width.max(1) as f64 * height.max(1) as f64;
    if pixels <= 6_000_000.0 {
        QUALITY_CARD
    } else if pixels <= 10_000_000.0 {
        QUALITY_MID
    } else {
        QUALITY_LONG
    }
}

pub fn with_device_scale(
    kit: &aurorix_render::CanvasKit<'_>,
    width: i32,
    height: i32,
    body: impl FnOnce(),
) {
    let factor = device_scale(width, height);
    if (factor - 1.0).abs() < 1e-4 {
        body();
        return;
    }
    let canvas = kit.canvas();
    canvas.save();
    canvas.scale((factor, factor));
    body();
    canvas.restore();
}

pub fn card_request(width: i32, height: i32) -> RenderRequest {
    let (out_width, out_height) = device_size(width, height);
    RenderRequest::new(out_width, out_height).with_quality(quality_for(out_width, out_height))
}

pub fn render_request(req: &songinfo::SongInfoRequest) -> RenderRequest {
    let (width, height) = songinfo::canvas_size(req);
    card_request(width, height)
}

pub fn songinfo_card(root: &Path, req: &songinfo::SongInfoRequest) -> RenderCard {
    let (width, height) = songinfo::canvas_size(req);
    let request = card_request(width, height);
    let root = root.to_path_buf();
    let owned = req.clone();
    RenderCard::new(request, SONG_DETAIL_LABEL, move |kit| {
        with_device_scale(kit, width, height, || songinfo::draw(kit, &root, &owned));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Instant;
    #[test]
    fn asset_kinds_match_directories() {
        let root = crate::assets_root();
        let ui = ui_asset(&root, "grade_phi");
        assert_eq!(ui, "ui/grade_phi.png");
        assert!(root.join(&ui).is_file());
        let avatar = avatar_asset(&root, "Introduction");
        assert!(avatar.starts_with("avatar/"));
        assert!(root.join(&avatar).is_file());
    }

    #[test]
    fn convention_key_and_swap_extension_agree() {
        assert_eq!(convention_key("illust", "Glaciaxion", "jpg"), "illust/Glaciaxion.jpg");
        assert_eq!(convention_key("thumb_s", "Glaciaxion", "jpg"), "thumb_s/Glaciaxion.jpg");
        assert_eq!(swap_extension("ui/a.png").as_deref(), Some("ui/a.jpg"));
        assert_eq!(swap_extension("blur/a.jpg").as_deref(), Some("blur/a.png"));
        assert_eq!(swap_extension("blur/a.jpeg"), None);
    }

    #[test]
    fn missing_manifest_falls_back_to_convention() {
        let root = Path::new("/nonexistent-phigros-root");
        assert_eq!(blur_key(root, "Song"), "blur/Song.jpg");
        assert_eq!(thumb_small_key(root, "Song"), "thumb_s/Song.jpg");
        assert_eq!(font_list(root).len(), 0);
    }

    fn rss_kb() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find(|line| line.starts_with("VmRSS:"))
                    .map(str::to_string)
            })
            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
            .unwrap_or(0)
    }

    fn bench_player() -> player::Player {
        player::Player {
            has_player: true,
            name: String::from("Okamikko"),
            avatar: String::from("avatar_1"),
            rank: 3,
            rank_level: 4,
            rks: 16.1234,
        }
    }

    fn bench_library(path: &Path) -> Option<Vec<crate::model::Song>> {
        let catalog = crate::domain::catalog::Catalog::open(path).ok()?;
        let songs: Vec<crate::model::Song> = catalog.songs().iter().take(200).cloned().collect();
        (songs.len() == 200).then_some(songs)
    }

    fn list_page(songs: &[crate::model::Song], page: usize) -> list::ListRequest {
        let rows = (0..20)
            .map(|index| {
                let song = &songs[(page * 20 + index) % songs.len()];
                list::ListRow {
                    song_id: song.id.clone(),
                    title: song.title.clone(),
                    composer: song.composer.clone(),
                    group: match page / 2 {
                        0 => String::from("RKS 16.00+"),
                        1 => String::from("RKS 15.50 - 15.99"),
                        2 => String::from("RKS 15.00 - 15.49"),
                        3 => String::from("RKS 14.00 - 14.99"),
                        4 => String::from("RKS 13.00 - 13.99"),
                        _ => String::from("RKS < 13.00"),
                    },
                    rank: index as i32 + 1,
                    diff: String::from("IN"),
                    constant: song.charts[2].constant,
                    known: song.charts[2].known(),
                    variant: song.variant(),
                    score: 1_000_000 - index as i32 * 1000,
                    acc: 100.0 - index as f64 * 0.1,
                    grade: String::from("V"),
                    rks: song.charts[2].constant,
                    next: String::from("V"),
                    need: 12_000 - index as i32 * 100,
                }
            })
            .collect();
        list::ListRequest {
            player: bench_player(),
            head: format!("定数表 · 第 {} 页", page + 1),
            note: String::from("200 首基准"),
            backdrop: songs[(page * 20) % songs.len()].id.clone(),
            stats: vec![
                list::ListStat {
                    value: String::from("20"),
                    label: String::from("谱面"),
                },
                list::ListStat {
                    value: String::from("15.42"),
                    label: String::from("平均定数"),
                },
            ],
            rows,
        }
    }

    fn table_page(songs: &[crate::model::Song], page: usize) -> table::TableRequest {
        let labels = ["EZ", "HD", "IN", "AT"];
        let sections = (0..5)
            .map(|section| {
                let rows = (0..8)
                    .map(|row| {
                        let song = &songs[(page * 40 + section * 8 + row) % songs.len()];
                        table::TableRow {
                            song_id: song.id.clone(),
                            title: song.title.clone(),
                            sub: format!(
                                "{} · BPM {} · {}",
                                song.composer, song.bpm_text, song.length
                            ),
                            cells: song
                                .charts
                                .iter()
                                .take(4)
                                .map(|chart| table::TableCell {
                                    constant: chart.constant,
                                    hit: chart.known(),
                                })
                                .collect(),
                        }
                    })
                    .collect();
                table::TableSection {
                    label: format!("{:.1}", 15.9 - (page * 5 + section) as f64 * 0.1),
                    rows,
                }
            })
            .collect();
        table::TableRequest {
            name: String::new(),
            title: String::from("定数 14.0+ · 全部谱面"),
            note: format!("200 首基准 · 第 {} 页", page + 1),
            backdrop: songs[(page * 40) % songs.len()].id.clone(),
            cols: labels.iter().map(|label| label.to_string()).collect(),
            sections,
        }
    }

    fn search_page(songs: &[crate::model::Song], page: usize) -> search::SearchRequest {
        let labels = ["EZ", "HD", "IN", "AT"];
        let rows = (0..20)
            .map(|row| {
                let song = &songs[(page * 20 + row) % songs.len()];
                search::SearchRow {
                    song_id: song.id.clone(),
                    title: song.title.clone(),
                    composer: song.composer.clone(),
                    group: format!("{:.1}", song.charts[2].constant),
                    levels: song
                        .charts
                        .iter()
                        .take(4)
                        .enumerate()
                        .map(|(index, chart)| search::SearchLevel {
                            diff: String::from(labels[index]),
                            const_text: if chart.known() {
                                format!("{:.1}", chart.constant)
                            } else {
                                String::from("-")
                            },
                            active: chart.known(),
                        })
                        .collect(),
                }
            })
            .collect();
        search::SearchRequest {
            head: String::from("SEARCH"),
            note: format!("200 首基准 · 第 {} 页", page + 1),
            backdrop: songs[(page * 20) % songs.len()].id.clone(),
            rows,
        }
    }

    fn render_page<F: FnOnce(&aurorix_render::CanvasKit<'_>)>(
        context: &aurorix_render::RenderContext,
        width: i32,
        height: i32,
        body: F,
    ) {
        let request = card_request(width, height);
        context
            .draw(request, |canvas, kit_context| {
                let kit =
                    aurorix_render::CanvasKit::with_chain(canvas, kit_context, sketch::font_chain());
                with_device_scale(&kit, width, height, || body(&kit));
            })
            .expect("渲染失败");
    }

    fn bench_report(name: &str, frames: &[f64], pages: usize) {
        let cold = frames[0];
        let mut hot: Vec<f64> = frames[pages..].to_vec();
        hot.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let min = hot.first().copied().unwrap_or(0.0);
        let max = hot.last().copied().unwrap_or(0.0);
        let median = hot[hot.len() / 2];
        let total: f64 = hot.iter().sum();
        println!(
            "{name}: {pages} 页 × 3 轮 | 冷 {cold:.0}ms | 热 min {min:.0} / 中位 {median:.0} / max {max:.0}ms | 热均 {:+.0}ms | 热合计 {total:.0}ms",
            total / hot.len().max(1) as f64
        );
    }

    #[test]
    #[ignore]
    fn bench_pages_200_songs() {
        let root = crate::assets_root();
        register_fonts(&root);
        let db = root.join("../data/phi.db");
        let Some(songs) = bench_library(&db) else {
            println!("跳过: 曲库不足 200 首 ({})", db.display());
            return;
        };
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context = aurorix_render::RenderContext::new(
            aurorix_render::SkiaBackend::handle(),
            64,
            512 * 1024 * 1024,
        )
        .with_asset_loader(assets.loader());

        let list_pages: Vec<list::ListRequest> = (0..10).map(|page| list_page(&songs, page)).collect();
        let table_pages: Vec<table::TableRequest> =
            (0..5).map(|page| table_page(&songs, page)).collect();
        let search_pages: Vec<search::SearchRequest> =
            (0..10).map(|page| search_page(&songs, page)).collect();
        let mut list_ms = Vec::new();
        let mut table_ms = Vec::new();
        let mut search_ms = Vec::new();
        let rss_start = rss_kb();
        for _ in 0..3 {
            for request in &list_pages {
                let (width, height) = list::canvas_size(request);
                let start = Instant::now();
                render_page(&context, width, height, |kit| list::draw(kit, &root, request));
                list_ms.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            for request in &table_pages {
                let (width, height) = table::canvas_size(request);
                let start = Instant::now();
                render_page(&context, width, height, |kit| table::draw(kit, &root, request));
                table_ms.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            for request in &search_pages {
                let (width, height) = search::canvas_size(request);
                let start = Instant::now();
                render_page(&context, width, height, |kit| search::draw(kit, &root, request));
                search_ms.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        bench_report("list", &list_ms, list_pages.len());
        bench_report("table", &table_ms, table_pages.len());
        bench_report("search", &search_ms, search_pages.len());
        println!("RSS {rss_start} → {} KB", rss_kb());
    }

    fn budget_report<F>(
        context: &aurorix_render::RenderContext,
        name: &str,
        width: i32,
        height: i32,
        rounds: usize,
        body: F,
    ) where
        F: Fn(&aurorix_render::CanvasKit<'_>),
    {
        let mut frames = Vec::new();
        let mut bytes = 0usize;
        let mut dims = (0, 0);
        for _ in 0..rounds {
            let request = card_request(width, height);
            dims = (request.width, request.height);
            let start = Instant::now();
            let out = context
                .draw(request, |canvas, kit_context| {
                    let kit = aurorix_render::CanvasKit::with_chain(
                        canvas,
                        kit_context,
                        sketch::font_chain(),
                    );
                    with_device_scale(&kit, width, height, || body(&kit));
                })
                .expect("渲染失败");
            frames.push(start.elapsed().as_secs_f64() * 1000.0);
            bytes = out.len();
        }
        let quality = quality_for(dims.0, dims.1);
        println!(
            "{name}: {}x{} q{quality} {:.2}MB | {:.0} / {:.0} / {:.0} ms",
            dims.0,
            dims.1,
            bytes as f64 / (1024.0 * 1024.0),
            frames[0],
            frames[frames.len() / 2],
            frames[frames.len() - 1]
        );
    }

    #[test]
    #[ignore]
    fn bench_render_budget() {
        let root = crate::assets_root();
        register_fonts(&root);
        let db = root.join("../data/phi.db");
        let Some(songs) = bench_library(&db) else {
            println!("跳过: 曲库不足 200 首 ({})", db.display());
            return;
        };
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context =
            aurorix_render::RenderContext::new(aurorix_render::SkiaBackend::handle(), 64, 512 * 1024 * 1024)
                .with_asset_loader(assets.loader());
        let first = list_page(&songs, 0);
        let long = list::ListRequest {
            rows: (0..200)
                .map(|index| first.rows[index % first.rows.len()].clone())
                .collect(),
            ..first.clone()
        };
        let longer = list::ListRequest {
            rows: (0..300)
                .map(|index| first.rows[index % first.rows.len()].clone())
                .collect(),
            ..first.clone()
        };
        let table_req = table_page(&songs, 0);
        let table_long = table::TableRequest {
            sections: (0..6)
                .map(|index| table::TableSection {
                    label: format!("Lv.{}", 16 - index),
                    rows: (0..50)
                        .map(|slot| {
                            table_req.sections[0].rows
                                [(index * 50 + slot) % table_req.sections[0].rows.len()]
                                .clone()
                        })
                        .collect(),
                })
                .collect(),
            ..table_req.clone()
        };
        let search_req = search_page(&songs, 0);
        let search_long = search::SearchRequest {
            rows: (0..300)
                .map(|index| search_req.rows[index % search_req.rows.len()].clone())
                .collect(),
            ..search_req.clone()
        };
        let bare = list::ListRequest {
            rows: Vec::new(),
            ..first.clone()
        };
        for (name, request) in [
            ("list-0", &bare),
            ("list-20", &first),
            ("list-200", &long),
            ("list-300", &longer),
        ] {
            let (width, height) = list::canvas_size(request);
            budget_report(&context, name, width, height, 3, |kit| {
                list::draw(kit, &root, request)
            });
        }
        let (width, height) = table::canvas_size(&table_req);
        budget_report(&context, "table", width, height, 3, |kit| {
            table::draw(kit, &root, &table_req)
        });
        let (width, height) = table::canvas_size(&table_long);
        budget_report(&context, "table-300", width, height, 3, |kit| {
            table::draw(kit, &root, &table_long)
        });
        let (width, height) = search::canvas_size(&search_long);
        budget_report(&context, "search-300", width, height, 3, |kit| {
            search::draw(kit, &root, &search_long)
        });
        for (name, request) in [
            ("/tmp/phigros-table300-preview.png", &table_long),
        ] {
            let (width, height) = table::canvas_size(request);
            let png = context
                .draw(
                    card_request(width, height).with_format(aurorix_render::RenderFormat::Png),
                    |canvas, kit_context| {
                        let kit = aurorix_render::CanvasKit::with_chain(
                            canvas,
                            kit_context,
                            sketch::font_chain(),
                        );
                        with_device_scale(&kit, width, height, || {
                            table::draw(&kit, &root, request)
                        });
                    },
                )
                .expect("渲染失败");
            std::fs::write(name, &png).expect("写出 PNG");
        }
        let (width, height) = search::canvas_size(&search_long);
        let png = context
            .draw(
                card_request(width, height).with_format(aurorix_render::RenderFormat::Png),
                |canvas, kit_context| {
                    let kit =
                        aurorix_render::CanvasKit::with_chain(canvas, kit_context, sketch::font_chain());
                    with_device_scale(&kit, width, height, || {
                        search::draw(&kit, &root, &search_long)
                    });
                },
            )
            .expect("渲染失败");
        std::fs::write("/tmp/phigros-search300-preview.png", &png).expect("写出 PNG");
        let (width, height) = list::canvas_size(&long);
        let preview =
            card_request(width, height).with_format(aurorix_render::RenderFormat::Png);
        let png = context
            .draw(preview, |canvas, kit_context| {
                let kit =
                    aurorix_render::CanvasKit::with_chain(canvas, kit_context, sketch::font_chain());
                with_device_scale(&kit, width, height, || list::draw(&kit, &root, &long));
            })
            .expect("渲染失败");
        std::fs::write("/tmp/phigros-list200-preview.png", &png).expect("写出 PNG");
        let (sw, sh) = search::canvas_size(&search_req);
        let sreq = card_request(sw, sh).with_format(aurorix_render::RenderFormat::Png);
        let spng = context
            .draw(sreq, |canvas, kit_context| {
                let kit =
                    aurorix_render::CanvasKit::with_chain(canvas, kit_context, sketch::font_chain());
                with_device_scale(&kit, sw, sh, || search::draw(&kit, &root, &search_req));
            })
            .expect("渲染失败");
        std::fs::write("/tmp/phigros-search-preview.png", &spng).expect("写出 PNG");
        for (name, source) in [
            ("/tmp/phigros-list20-preview.png", &first),
            ("/tmp/phigros-list300-preview.png", &longer),
        ] {
            let (width, height) = list::canvas_size(source);
            let request =
                card_request(width, height).with_format(aurorix_render::RenderFormat::Png);
            let png = context
                .draw(request, |canvas, kit_context| {
                    let kit = aurorix_render::CanvasKit::with_chain(
                        canvas,
                        kit_context,
                        sketch::font_chain(),
                    );
                    with_device_scale(&kit, width, height, || list::draw(&kit, &root, source));
                })
                .expect("渲染失败");
            std::fs::write(name, &png).expect("写出 PNG");
        }
        for (name, source) in [("/tmp/phigros-table-preview.png", &table_req)] {
            let (width, height) = table::canvas_size(source);
            let request =
                card_request(width, height).with_format(aurorix_render::RenderFormat::Png);
            let png = context
                .draw(request, |canvas, kit_context| {
                    let kit = aurorix_render::CanvasKit::with_chain(
                        canvas,
                        kit_context,
                        sketch::font_chain(),
                    );
                    with_device_scale(&kit, width, height, || {
                        table::draw(&kit, &root, source)
                    });
                })
                .expect("渲染失败");
            std::fs::write(name, &png).expect("写出 PNG");
        }
        let (width, height) = search::canvas_size(&search_req);
        budget_report(&context, "search", width, height, 3, |kit| {
            search::draw(kit, &root, &search_req)
        });
    }
}
