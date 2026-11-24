use std::sync::Arc;
use std::sync::OnceLock;

use aurorix_buttons::{only, style, Builder, ButtonStyle, Keyboard};
use aurorix_plugin::{
    role_allows, CallbackBook, CallbackFuture, CallbackHandler, CallbackRequest, CommandSpec,
    Invocation, Outgoing, PluginError, ReplyKind, RoleRequired,
};
use aurorix_template::Args;

use crate::commands::{self, ok};
use crate::repo::BlockEntry;
use crate::service::{fingerprint, now_seconds, put, PhigrosService};

pub const BTN_BLOCK_PICK: &str = "admin.block.pick";
pub const BTN_BLOCK_APPLY: &str = "admin.block.apply";
pub const BTN_BLOCK_CANCEL: &str = "admin.block.cancel";
pub const BTN_UNBLOCK_APPLY: &str = "admin.unblock.apply";

pub const CALLBACK_SUFFIXES: [&str; 4] = [
    BTN_BLOCK_PICK,
    BTN_BLOCK_APPLY,
    BTN_BLOCK_CANCEL,
    BTN_UNBLOCK_APPLY,
];

pub const LABEL_BLOCK_PICK: &str = "选中";
pub const LABEL_BLOCK_APPLY: &str = "确认屏蔽";
pub const LABEL_BLOCK_CANCEL: &str = "取消";
pub const LABEL_UNBLOCK_APPLY: &str = "撤销屏蔽";
pub const LABEL_BLOCKED: &str = "已屏蔽";

pub const ACTION_LIST: &str = "/p admin stats       查看运行概览
/p block <名字>     屏蔽作弊嫌疑玩家（不上任何排行）
/p unblock <名字>   撤销屏蔽
/p block list       查看屏蔽名单";

pub const BLOCK_USAGE: &str = "/p block <名字>      按名字模糊找人, 选中后确认屏蔽
/p unblock <名字>    把名字移出屏蔽名单
/p block list        查看当前名单";

pub const NOT_FOUND: &str = "没找到叫「%s」的玩家。只认本插件绑定过的人，名字要与他存档里的昵称对得上";

pub const TARGET_MISSING: &str = "没认出要处理的是谁，请重新发送 /p block";

pub const DENIED_ADMIN: &str = "这条指令只给管理员";

pub const DENIED_NOT_MINE: &str = "这颗按钮不是给你的，请自己发一次指令";

const SEARCH_LIMIT: usize = 10;
const SEARCH_SCAN: i64 = 200;
const PACK_SEP: char = '\u{1}';
const UNKNOWN_PLAYER: &str = "未命名玩家";

#[derive(Clone, Debug, Default, PartialEq)]
struct Candidate {
    user_id: String,
    nickname: String,
    object_id: String,
    rks: f64,
    blocked: bool,
}

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("admin")
            .alias("运维")
            .describe("运维概览")
            .role(RoleRequired::Admin)
            .handler(commands::on(service, run_admin)),
        CommandSpec::new("block")
            .alias("屏蔽")
            .describe("屏蔽作弊嫌疑玩家（不上任何排行）")
            .role(RoleRequired::Admin)
            .handler(commands::on(service, run_block)),
        CommandSpec::new("unblock")
            .alias("解禁")
            .alias("解除屏蔽")
            .describe("把玩家移出屏蔽名单")
            .role(RoleRequired::Admin)
            .handler(commands::on(service, run_unblock)),
    ]
}

fn allowed(inv: &Invocation) -> bool {
    role_allows(Some(RoleRequired::Admin), inv.role)
}

fn denied() -> Outgoing {
    Outgoing::new(ReplyKind::Denied, DENIED_ADMIN.to_string())
}

fn notice_denied() -> Outgoing {
    Outgoing::new(ReplyKind::Denied, DENIED_NOT_MINE.to_string())
}

async fn run_admin(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if !allowed(&inv) {
        return ok(commands::with_at(denied(), &inv));
    }
    let action = inv.text.trim().to_lowercase();
    let out = match action.as_str() {
        "" => usage(&service, ""),
        "stats" => stats(&service).await,
        other => usage(&service, other),
    };
    ok(commands::with_at(out, &inv))
}

async fn run_block(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if !allowed(&inv) {
        return ok(commands::with_at(denied(), &inv));
    }
    let keyword = inv.text.trim().to_string();
    let out = if keyword.eq_ignore_ascii_case("list") || keyword == "列表" {
        list(&service).await
    } else if keyword.is_empty() {
        PhigrosService::notice(BLOCK_USAGE)
    } else {
        block_entry(&service, &inv.user, &keyword).await
    };
    ok(commands::with_at(out, &inv))
}

async fn run_unblock(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if !allowed(&inv) {
        return ok(commands::with_at(denied(), &inv));
    }
    let keyword = inv.text.trim().to_string();
    let out = if keyword.is_empty() {
        PhigrosService::notice(BLOCK_USAGE)
    } else {
        unblock_by_keyword(&service, &keyword).await
    };
    ok(commands::with_at(out, &inv))
}

fn usage(service: &Arc<PhigrosService>, unknown: &str) -> Outgoing {
    let body = if unknown.is_empty() {
        ACTION_LIST.to_string()
    } else {
        format!("未知动作：「{unknown}」\n\n{ACTION_LIST}")
    };
    let mut args = Args::new();
    put(&mut args, "body", body);
    service.text("admin.usage", &args)
}

async fn stats(service: &Arc<PhigrosService>) -> Outgoing {
    let repo = service.repo();
    let totals = repo.stats().await.unwrap_or_default();
    let catalog = service.catalog();
    let mut charts = 0_usize;
    let mut aliases = 0_usize;
    for song in catalog.songs() {
        charts += song.all_difficulties().len();
        aliases += song.aliases.len();
    }
    let mut args = Args::new();
    put(&mut args, "users", totals.users.to_string());
    put(&mut args, "records", totals.records.to_string());
    put(&mut args, "synced", totals.synced.to_string());
    put(&mut args, "failed", totals.failed.to_string());
    put(&mut args, "schema", totals.schema.to_string());
    put(&mut args, "songs", catalog.songs().len().to_string());
    put(&mut args, "charts", charts.to_string());
    put(&mut args, "aliases", aliases.to_string());
    put(&mut args, "chapters", catalog.chapters().len().to_string());
    put(&mut args, "version", totals.schema.to_string());
    put(
        &mut args,
        "source",
        crate::service::catalog_path(service.root())
            .display()
            .to_string(),
    );
    service.text("admin.stats", &args)
}

async fn block_entry(
    service: &Arc<PhigrosService>,
    operator: &str,
    keyword: &str,
) -> Outgoing {
    let hits = search(service, keyword).await;
    match hits.len() {
        0 => PhigrosService::notice(NOT_FOUND.replace("%s", keyword)),
        1 => confirm_out(service, operator, &hits[0]),
        _ => pick_out(service, operator, keyword, &hits),
    }
}

fn confirm_out(service: &Arc<PhigrosService>, operator: &str, hit: &Candidate) -> Outgoing {
    let mut args = Args::new();
    put(&mut args, "nickname", hit.nickname.clone());
    put(&mut args, "rks", format!("{:.2}", hit.rks));
    let out = service.text("admin.block.confirm", &args);
    match confirm_board(operator, &hit.user_id) {
        Some(keyboard) => out.keyboard(keyboard),
        None => out,
    }
}

fn pick_out(
    service: &Arc<PhigrosService>,
    operator: &str,
    keyword: &str,
    hits: &[Candidate],
) -> Outgoing {
    let mut rows = String::new();
    for (index, hit) in hits.iter().enumerate() {
        rows.push_str(&format!(
            "{:>2}. {}  RKS {:.2}  {}\n",
            index + 1,
            dash(&hit.nickname),
            hit.rks,
            state_label(hit.blocked)
        ));
    }
    let mut args = Args::new();
    put(&mut args, "keyword", keyword.to_string());
    put(&mut args, "count", hits.len().to_string());
    put(&mut args, "rows", rows);
    let out = service.text("admin.block.pick", &args);
    match pick_board(operator, hits) {
        Some(keyboard) => out.keyboard(keyboard),
        None => out,
    }
}

async fn list(service: &Arc<PhigrosService>) -> Outgoing {
    let rows = service.repo().blocks().await.unwrap_or_default();
    if rows.is_empty() {
        return PhigrosService::notice("屏蔽名单是空的");
    }
    let mut body = String::new();
    for (index, row) in rows.iter().enumerate() {
        body.push_str(&format!(
            "{:>2}. {}  理由 {}  操作人 {}\n",
            index + 1,
            dash(&row.nickname),
            dash(&row.reason),
            dash(&row.operator)
        ));
    }
    let mut args = Args::new();
    put(&mut args, "rows", body);
    service.text("admin.block.list", &args)
}

async fn unblock_by_keyword(service: &Arc<PhigrosService>, keyword: &str) -> Outgoing {
    let rows = service.repo().blocks().await.unwrap_or_default();
    let needle = keyword.to_lowercase();
    let hit = rows.iter().find(|row| {
        row.user_id == keyword || row.nickname.to_lowercase().contains(&needle)
    });
    let Some(hit) = hit else {
        return PhigrosService::notice(format!("名单里没有叫「{keyword}」的人"));
    };
    match service.repo().remove_block(&hit.user_id).await {
        Ok(true) => unblock_ok(service, &hit.nickname),
        Ok(false) => PhigrosService::notice(format!("名单里没有叫「{keyword}」的人")),
        Err(error) => PhigrosService::fail(format!("撤销屏蔽失败：{error}")),
    }
}

fn unblock_ok(service: &Arc<PhigrosService>, nickname: &str) -> Outgoing {
    let mut args = Args::new();
    put(&mut args, "nickname", nickname.to_string());
    service.text("admin.block.unblock.ok", &args)
}

async fn search(service: &Arc<PhigrosService>, keyword: &str) -> Vec<Candidate> {
    let repo = service.repo();
    let Ok(board) = repo.leader_board("", SEARCH_SCAN).await else {
        return Vec::new();
    };
    let needle = keyword.to_lowercase();
    let mut out: Vec<Candidate> = Vec::new();
    for row in board.top {
        if out.len() >= SEARCH_LIMIT {
            break;
        }
        if row.user_id != keyword && !row.nickname.to_lowercase().contains(&needle) {
            continue;
        }
        let object_id = repo
            .binding(&row.user_id)
            .await
            .ok()
            .flatten()
            .map(|binding| binding.object_id)
            .unwrap_or_default();
        let blocked = repo.is_blocked(&row.user_id).await.unwrap_or_default();
        out.push(Candidate {
            user_id: row.user_id,
            nickname: player_name(&row.nickname),
            object_id,
            rks: row.rks,
            blocked,
        });
    }
    out
}

async fn candidate(service: &Arc<PhigrosService>, user_id: &str) -> Option<Candidate> {
    let target = user_id.trim();
    if target.is_empty() {
        return None;
    }
    let repo = service.repo();
    let binding = repo.binding(target).await.ok().flatten()?;
    let rks = repo
        .profile(target)
        .await
        .ok()
        .flatten()
        .map(|profile| profile.rks)
        .unwrap_or_default();
    let blocked = repo.is_blocked(target).await.unwrap_or_default();
    Some(Candidate {
        user_id: binding.user_id,
        nickname: player_name(&binding.nickname),
        object_id: binding.object_id,
        rks,
        blocked,
    })
}

fn player_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return UNKNOWN_PLAYER.to_string();
    }
    trimmed.to_string()
}

fn dash(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::from("-");
    }
    trimmed.to_string()
}

fn state_label(blocked: bool) -> &'static str {
    if blocked {
        return LABEL_BLOCKED;
    }
    "未屏蔽"
}

fn pack(operator: &str, target: &str) -> String {
    format!("{operator}{PACK_SEP}{target}")
}

fn unpack(data: &str) -> Option<(String, String)> {
    let (operator, target) = data.split_once(PACK_SEP)?;
    if operator.trim().is_empty() || target.trim().is_empty() {
        return None;
    }
    Some((operator.to_string(), target.to_string()))
}

fn opts(operator: &str) -> Vec<aurorix_buttons::Opt> {
    vec![
        style(ButtonStyle::GraySolid),
        only(vec![operator.to_string()]),
    ]
}

fn finish(builder: Builder) -> Option<Keyboard> {
    let keyboard = builder.build()?;
    match keyboard.validate() {
        Ok(()) => Some(keyboard),
        Err(_) => None,
    }
}

fn confirm_board(operator: &str, target: &str) -> Option<Keyboard> {
    let data = pack(operator, target);
    finish(
        Builder::new()
            .scope(crate::buttons::SCOPE)
            .callback(
                BTN_BLOCK_APPLY,
                LABEL_BLOCK_APPLY,
                &data,
                opts(operator),
            )
            .callback(
                BTN_BLOCK_CANCEL,
                LABEL_BLOCK_CANCEL,
                &data,
                opts(operator),
            ),
    )
}

fn pick_board(operator: &str, hits: &[Candidate]) -> Option<Keyboard> {
    let mut builder = Builder::new().scope(crate::buttons::SCOPE);
    for hit in hits {
        let label = if hit.blocked {
            format!("{} · {}", hit.nickname, LABEL_BLOCKED)
        } else {
            hit.nickname.clone()
        };
        builder = builder.callback(
            BTN_BLOCK_PICK,
            &label,
            &hit.user_id,
            opts(operator),
        );
    }
    finish(builder)
}

fn unblock_board(operator: &str, target: &str) -> Option<Keyboard> {
    finish(
        Builder::new()
            .scope(crate::buttons::SCOPE)
            .callback(
                BTN_UNBLOCK_APPLY,
                LABEL_UNBLOCK_APPLY,
                target,
                opts(operator),
            ),
    )
}

pub fn unblock_board_for(operator: &str, target: &str) -> Option<Keyboard> {
    unblock_board(operator, target)
}

pub fn install(service: &Arc<PhigrosService>) -> usize {
    static ONCE: OnceLock<()> = OnceLock::new();
    if ONCE.set(()).is_err() {
        return 0;
    }
    let book = CallbackBook::global();
    let mut registered = 0;
    for suffix in CALLBACK_SUFFIXES {
        let id = crate::buttons::callback_id(suffix);
        if book.register(id, handler_for(Arc::clone(service))) {
            registered += 1;
        }
    }
    registered
}

fn handler_for(service: Arc<PhigrosService>) -> CallbackHandler {
    Arc::new(move |request: CallbackRequest| {
        let service = Arc::clone(&service);
        let future: CallbackFuture = Box::pin(async move { dispatch(&service, &request).await });
        future
    })
}

async fn dispatch(
    service: &Arc<PhigrosService>,
    request: &CallbackRequest,
) -> Option<Outgoing> {
    let suffix = request
        .button_id
        .strip_prefix(crate::buttons::SCOPE)
        .and_then(|rest| rest.strip_prefix('.'))?;
    match suffix {
        BTN_BLOCK_PICK => Some(pick(service, request).await),
        BTN_BLOCK_APPLY => Some(apply(service, request).await),
        BTN_BLOCK_CANCEL => Some(cancel(request)),
        BTN_UNBLOCK_APPLY => Some(unblock_apply(service, request).await),
        _ => None,
    }
}

async fn pick(service: &Arc<PhigrosService>, request: &CallbackRequest) -> Outgoing {
    let Some(hit) = candidate(service, &request.data).await else {
        return PhigrosService::notice(TARGET_MISSING);
    };
    if hit.blocked {
        return already(&hit.nickname);
    }
    confirm_out(service, &request.user, &hit)
}

async fn apply(service: &Arc<PhigrosService>, request: &CallbackRequest) -> Outgoing {
    let Some((operator, target)) = unpack(&request.data) else {
        return PhigrosService::notice(TARGET_MISSING);
    };
    if operator != request.user {
        return notice_denied();
    }
    let Some(hit) = candidate(service, &target).await else {
        return PhigrosService::notice(TARGET_MISSING);
    };
    if hit.blocked {
        return already(&hit.nickname);
    }
    let entry = BlockEntry {
        user_id: hit.user_id.clone(),
        nickname: hit.nickname.clone(),
        object_id: hit.object_id.clone(),
        reason: String::new(),
        operator: request.user.clone(),
        created_at: now_seconds(),
    };
    if let Err(error) = service.repo().put_block(&entry).await {
        return PhigrosService::fail(format!("写入屏蔽名单失败：{error}"));
    }
    let mut args = Args::new();
    put(&mut args, "nickname", hit.nickname);
    service.text("admin.block.ok", &args)
}

fn cancel(request: &CallbackRequest) -> Outgoing {
    match unpack(&request.data) {
        Some((operator, _)) if operator == request.user => {
            PhigrosService::notice("已取消，没有改动名单")
        }
        Some(_) => notice_denied(),
        None => PhigrosService::notice("已取消，没有改动名单"),
    }
}

async fn unblock_apply(service: &Arc<PhigrosService>, request: &CallbackRequest) -> Outgoing {
    let target = request.data.trim().to_string();
    if target.is_empty() {
        return PhigrosService::notice(TARGET_MISSING);
    }
    let rows = service.repo().blocks().await.unwrap_or_default();
    let nickname = rows
        .iter()
        .find(|row| row.user_id == target)
        .map(|row| row.nickname.clone());
    let Some(nickname) = nickname else {
        return PhigrosService::notice("这个玩家已经不在名单上了");
    };
    match service.repo().remove_block(&target).await {
        Ok(_) => unblock_ok(service, &nickname),
        Err(error) => PhigrosService::fail(format!("撤销屏蔽失败：{error}")),
    }
}

fn already(nickname: &str) -> Outgoing {
    PhigrosService::notice(format!(
        "「{nickname}」已经在名单上了。\n撤销用 /p unblock {nickname}"
    ))
}

pub fn cache_seed(operator: &str, target: &str) -> u64 {
    fingerprint("admin.block", &(operator, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(user_id: &str, nickname: &str, blocked: bool) -> Candidate {
        Candidate {
            user_id: user_id.to_string(),
            nickname: nickname.to_string(),
            object_id: format!("obj-{user_id}"),
            rks: 15.25,
            blocked,
        }
    }

    #[test]
    fn commands_are_admin_only_and_aliased() {
        assert_eq!(RoleRequired::Admin.as_str(), "admin");
        assert!(RoleRequired::Member.can_use(Some(RoleRequired::Member)));
        assert!(!RoleRequired::Admin.can_use(Some(RoleRequired::Member)));
        assert!(RoleRequired::Admin.can_use(Some(RoleRequired::Admin)));
        assert!(RoleRequired::Admin.can_use(Some(RoleRequired::Owner)));
        assert!(!role_allows(Some(RoleRequired::Admin), None));
        assert!(role_allows(None, None));
    }

    #[test]
    fn packed_callback_data_round_trips() {
        let data = pack("10001", "20002");
        assert!(data.contains(PACK_SEP));
        assert_eq!(unpack(&data), Some((String::from("10001"), String::from("20002"))));
        assert!(unpack("20002").is_none());
        assert!(unpack(&format!("{PACK_SEP}20002")).is_none());
        assert!(unpack(&format!("10001{PACK_SEP}")).is_none());
    }

    #[test]
    fn pick_board_carries_the_target_user_id() {
        let hits = vec![
            hit("20002", "Alpha", false),
            hit("30003", "Beta", true),
        ];
        let board = pick_board("10001", &hits).expect("候选板");
        assert!(board.validate().is_ok());
        let buttons = board.flatten();
        assert_eq!(buttons.len(), 2);
        for button in &buttons {
            assert_eq!(button.id, crate::buttons::callback_id(BTN_BLOCK_PICK));
            let data = button.action.callback_data.clone().unwrap_or_default();
            assert!(!data.is_empty(), "回调数据不能为空");
            assert!(hits.iter().any(|item| item.user_id == data));
        }
        assert!(buttons[1].render_data.label.contains(LABEL_BLOCKED));
    }

    #[test]
    fn confirm_board_limits_clicking_to_the_operator() {
        let board = confirm_board("10001", "20002").expect("确认板");
        assert!(board.validate().is_ok());
        let buttons = board.flatten();
        assert_eq!(buttons.len(), 2);
        assert!(board.has_button(&crate::buttons::callback_id(BTN_BLOCK_APPLY)));
        assert!(board.has_button(&crate::buttons::callback_id(BTN_BLOCK_CANCEL)));
        for button in &buttons {
            let data = button.action.callback_data.clone().unwrap_or_default();
            assert_eq!(unpack(&data).map(|(operator, _)| operator), Some(String::from("10001")));
            assert_eq!(
                button.action.permission.permission_type,
                aurorix_buttons::AllowedPermission::SomeUser
            );
            assert_eq!(
                button.action.permission.specify_user_ids,
                vec![String::from("10001")]
            );
        }
    }

    #[test]
    fn unblock_board_and_labels_stay_in_sync() {
        let board = unblock_board("10001", "20002").expect("撤销板");
        assert!(board.has_button(&crate::buttons::callback_id(BTN_UNBLOCK_APPLY)));
        assert_eq!(state_label(true), LABEL_BLOCKED);
        assert_eq!(state_label(false), "未屏蔽");
        assert_eq!(dash("  "), "-");
        assert_eq!(dash(" 霓虹灯牌 "), "霓虹灯牌");
        assert_eq!(player_name(""), UNKNOWN_PLAYER);
        assert_eq!(player_name(" Alpha "), "Alpha");
        assert!(ACTION_LIST.contains("/p block list"));
        assert!(BLOCK_USAGE.contains("/p unblock"));
        assert!(cache_seed("10001", "20002") > 0);
    }
}
