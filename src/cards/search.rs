use std::path::Path;

use aurorix_render::CanvasKit;

use super::table;

pub const SEARCH_MAX: usize = 300;

pub const SEARCH_HARD_ROWS: usize = 300;

pub const SEARCH_NAME: &str = "SEARCH";

pub const SEARCH_DIFF_ORDER: [&str; 5] = ["EZ", "HD", "IN", "AT", "LEGACY"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchLevel {
    pub diff: String,
    pub const_text: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchRow {
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub group: String,
    pub levels: Vec<SearchLevel>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchRequest {
    pub head: String,
    pub note: String,
    pub backdrop: String,
    pub rows: Vec<SearchRow>,
}

pub fn clamp_rows(rows: &[SearchRow]) -> &[SearchRow] {
    if rows.len() > SEARCH_HARD_ROWS {
        &rows[..SEARCH_HARD_ROWS]
    } else {
        rows
    }
}

pub fn search_cols(rows: &[SearchRow]) -> Vec<String> {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for row in rows {
        for level in &row.levels {
            seen.insert(level.diff.as_str());
        }
    }
    let mut out: Vec<String> = Vec::with_capacity(seen.len());
    for diff in SEARCH_DIFF_ORDER {
        if seen.remove(diff) {
            out.push(String::from(diff));
        }
    }
    for diff in seen {
        out.push(String::from(diff));
    }
    out
}

pub fn search_table_row(row: &SearchRow, cols: &[String]) -> table::TableRow {
    let mut cells = vec![table::TableCell::default(); cols.len()];
    for (index, col) in cols.iter().enumerate() {
        let Some(level) = row.levels.iter().find(|item| &item.diff == col) else {
            continue;
        };
        let Ok(constant) = level.const_text.trim().parse::<f64>() else {
            continue;
        };
        if constant <= 0.0 {
            continue;
        }
        cells[index] = table::TableCell {
            constant,
            hit: level.active,
        };
    }
    table::TableRow {
        song_id: row.song_id.clone(),
        title: row.title.clone(),
        sub: row.composer.clone(),
        cells,
    }
}

pub fn search_table(req: &SearchRequest) -> table::TableRequest {
    let rows = clamp_rows(&req.rows);
    let cols = search_cols(rows);
    let mut out = table::TableRequest {
        name: String::from(SEARCH_NAME),
        title: req.head.clone(),
        note: req.note.clone(),
        backdrop: req.backdrop.clone(),
        cols: cols.clone(),
        sections: Vec::new(),
    };
    for row in rows {
        let fresh = match out.sections.last() {
            None => true,
            Some(section) => section.label != row.group,
        };
        if fresh {
            out.sections.push(table::TableSection {
                label: row.group.clone(),
                rows: Vec::new(),
            });
        }
        let item = search_table_row(row, &cols);
        if let Some(section) = out.sections.last_mut() {
            section.rows.push(item);
        }
    }
    out
}

pub fn canvas_size(req: &SearchRequest) -> (i32, i32) {
    table::canvas_size(&search_table(req))
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SearchRequest) {
    table::draw(kit, root, &search_table(req));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    use aurorix_render::skia_safe::Image;

    const OUT: &str = "/tmp/phigros-verify/10-search.png";

    fn level(diff: &str, const_text: &str, active: bool) -> SearchLevel {
        SearchLevel {
            diff: String::from(diff),
            const_text: String::from(const_text),
            active,
        }
    }

    fn row_data(index: usize) -> SearchRow {
        let ids = [
            "000AinSophAur.Yumeji.0",
            "70MinutesFighters.かたぎり.0",
            "996.李化禹.0",
        ];
        let id = ids[index % ids.len()];
        SearchRow {
            song_id: String::from(id),
            title: format!("Song {index}"),
            composer: String::from("Composer"),
            group: String::from("15.9"),
            levels: vec![
                level("EZ", "8.6", false),
                level("HD", "9.5", false),
                level("IN", "15.9", true),
            ],
        }
    }

    fn request() -> SearchRequest {
        SearchRequest {
            head: String::from("定数 15.9 · IN"),
            note: String::from("命中 3 首 · 按定数从高到低"),
            backdrop: String::from("000AinSophAur.Yumeji.0"),
            rows: vec![row_data(0), row_data(1), row_data(2)],
        }
    }

    #[test]
    fn cols_follow_the_library_order() {
        let mut rows = vec![row_data(0)];
        rows[0].levels.push(level("EXTRA", "15.0", false));
        rows[0].levels.push(level("AT", "15.0", false));
        assert_eq!(
            search_cols(&rows),
            vec![
                String::from("EZ"),
                String::from("HD"),
                String::from("IN"),
                String::from("AT"),
                String::from("EXTRA")
            ]
        );
    }

    #[test]
    fn table_keeps_consecutive_groups_apart() {
        let mut rows = vec![row_data(0), row_data(1)];
        rows[1].group = String::from("15.8");
        let table = search_table(&SearchRequest {
            head: String::new(),
            note: String::new(),
            backdrop: String::new(),
            rows,
        });
        assert_eq!(table.name, SEARCH_NAME);
        assert_eq!(table.sections.len(), 2);
        assert_eq!(table.sections[0].label, "15.9");
        assert_eq!(table.sections[1].label, "15.8");
        assert_eq!(table.sections[0].rows.len(), 1);
    }

    #[test]
    fn cells_align_with_columns_and_carry_hit() {
        let req = request();
        let table = search_table(&req);
        let row = &table.sections[0].rows[0];
        assert_eq!(row.cells.len(), table.cols.len());
        assert_eq!(row.song_id, req.rows[0].song_id);
        assert_eq!(row.sub, req.rows[0].composer);
        let index = table.cols.iter().position(|col| col == "IN").unwrap();
        assert!(row.cells[index].hit);
        assert_eq!(row.cells[index].constant, 15.9);
    }

    #[test]
    #[ignore]
    fn render_search() {
        let root = crate::assets_root();
        crate::cards::register_fonts(&root);
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context = aurorix_render::RenderContext::new(
            aurorix_render::SkiaBackend::handle(),
            64,
            256 * 1024 * 1024,
        )
        .with_asset_loader(assets.loader());
        let req = request();
        let (width, height) = canvas_size(&req);
        let request = aurorix_render::RenderRequest::new(width, height);
        let start = Instant::now();
        let bytes = context
            .draw(request, |canvas, ctx| {
                let kit = CanvasKit::with_chain(canvas, ctx, crate::cards::sketch::font_chain());
                draw(&kit, &root, &req);
            })
            .expect("渲染失败")
            .to_vec();
        let elapsed = start.elapsed();
        if let Some(parent) = std::path::Path::new(OUT).parent() {
            std::fs::create_dir_all(parent).expect("建出图目录");
        }
        std::fs::write(OUT, &bytes).expect("写出 PNG");
        println!(
            "检索结果渲染耗时 {} ms ({} 字节, {width}x{height})",
            elapsed.as_millis(),
            bytes.len()
        );
        let image =
            Image::from_encoded(aurorix_render::skia_safe::Data::new_copy(&bytes)).expect("解码头");
        assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
        assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
        assert!(bytes.len() > 1000, "字节数必须大于 1000");
    }
}
