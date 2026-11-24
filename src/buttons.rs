use std::sync::Arc;
use std::sync::OnceLock;

use aurorix_buttons::{style, Builder, ButtonStyle, Keyboard};
use aurorix_plugin::{
    CallbackBook, CallbackFuture, CallbackHandler, CallbackRequest, Outgoing, ReplyKind,
};
use aurorix_template::Args;

use crate::cmd_query::{STATS_SHORT, STATS_SHORT_MAX};
use crate::service::PhigrosService;

pub const SCOPE: &str = crate::PLUGIN_ID;

pub const COMMAND_ROOT: &str = "/p ";

pub const BTN_PREV: &str = "page.prev";
pub const BTN_NEXT: &str = "page.next";
pub const BTN_REFRESH: &str = "page.refresh";
pub const BTN_MENU: &str = "common.menu";

pub const BTN_UNBIND_CONFIRM: &str = "account.unbind.confirm";
pub const BTN_UNBIND_CANCEL: &str = "account.unbind.cancel";
pub const BTN_CLEAN_CONFIRM: &str = "account.clean.confirm";
pub const BTN_CLEAN_CANCEL: &str = "account.clean.cancel";
pub const BTN_LOGIN_CANCEL: &str = "account.login.cancel";
pub const BTN_LOGIN: &str = "account.unbound.login";
pub const BTN_RECHECK: &str = "score.recheck";
pub const BTN_BOARD_SWITCH: &str = "score.b30.switch";
pub const BTN_CHALLENGE_AGAIN: &str = "challenge.again";

pub const CALLBACK_SUFFIXES: [&str; 13] = [
    BTN_PREV,
    BTN_NEXT,
    BTN_REFRESH,
    BTN_MENU,
    BTN_UNBIND_CONFIRM,
    BTN_UNBIND_CANCEL,
    BTN_CLEAN_CONFIRM,
    BTN_CLEAN_CANCEL,
    BTN_LOGIN,
    BTN_LOGIN_CANCEL,
    BTN_RECHECK,
    BTN_BOARD_SWITCH,
    BTN_CHALLENGE_AGAIN,
];

pub const LABEL_PREV: &str = "上一页";
pub const LABEL_NEXT: &str = "下一页";
pub const LABEL_MORE: &str = "看更多";
pub const LABEL_REFRESH: &str = "刷新";
pub const LABEL_MENU: &str = "PGR菜单";
pub const LABEL_CHAP_DETAIL: &str = "单章详情";
pub const LABEL_CHAP_BACK: &str = "全部章节";
pub const LABEL_SONG_SCORE: &str = "该曲成绩";
pub const LABEL_SONG_DETAIL: &str = "该曲图鉴";
pub const LABEL_SONG_PLAY: &str = "播放该曲";
pub const LABEL_RAND_AGAIN: &str = "再来一张";
pub const LABEL_PUSH_AGAIN: &str = "推分建议";
pub const LABEL_RECHECK: &str = "再查一遍";
pub const LABEL_BEST30: &str = "Best 30";
pub const LABEL_BEST50: &str = "Best 50";
pub const LABEL_SINGLE_ENTRY: &str = "单曲成绩";
pub const LABEL_LOGIN: &str = "Taptap 登录";
pub const LABEL_BIND_ENTRY: &str = "Stk 绑定";
pub const LABEL_CHALLENGE_AGAIN: &str = "再随一次";
pub const LABEL_UNBIND_CONFIRM: &str = "确认解绑";
pub const LABEL_CLEAN_CONFIRM: &str = "确认清除";
pub const LABEL_CANCEL: &str = "取消";
pub const LABEL_LOGIN_CANCEL: &str = "取消登录";
pub const DENIED_NOT_MINE: &str = "这不是你的确认页，请自己发一次指令";

pub const BOARD_SMALL: usize = 30;

pub const BOARD_LARGE: usize = 50;
const KIND_LIST: &str = "list";
const KIND_STATS: &str = "stats";
const KIND_SUGGEST: &str = "suggest";
const KIND_TABLE: &str = "table";
const KIND_CHAPTER: &str = "chapter";
const KIND_CHAPTER_DETAIL: &str = "chapterdetail";
const KIND_PLAIN: &str = "plain";
const KIND_SONG: &str = "song";
const KIND_RAND: &str = "rand";
const KIND_BOARD: &str = "board";
const KIND_BIND_OK: &str = "bindok";
const KIND_ME: &str = "me";
const KIND_UNBOUND: &str = "unbound";
const KIND_CHALLENGE: &str = "challenge";
const KIND_UNBIND_CONFIRM: &str = "unbindconfirm";
const KIND_CLEAN_CONFIRM: &str = "cleanconfirm";
const KIND_LOGIN_TICKET: &str = "loginticket";
const KIND_HELP: &str = "help";
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Plain,
    List {
        raw: String,
        page: usize,
    },
    Stats {
        raw: String,
        page: usize,
    },
    Suggest {
        raw: String,
        page: usize,
    },
    Table {
        raw: String,
        page: usize,
    },
    Chapter {
        chapters: Vec<String>,
    },
    ChapterDetail {
        name: String,
    },
    Song {
        title: String,
    },
    Rand {
        raw: String,
        title: String,
    },
    Board {
        size: usize,
    },
    BindOk,
    Me,
    Help,
    Unbound,
    Challenge {
        raw: String,
    },
    UnbindConfirm {
        user: String,
    },
    CleanConfirm {
        user: String,
    },
    LoginTicket {
        user: String,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonCtx {
    pub page: usize,
    pub pages: usize,
}

pub fn callback_ids() -> Vec<String> {
    CALLBACK_SUFFIXES
        .iter()
        .map(|suffix| format!("{SCOPE}.{suffix}"))
        .collect()
}

pub fn callback_id(suffix: &str) -> String {
    format!("{SCOPE}.{suffix}")
}

fn kind_of(page: &Page) -> &'static str {
    match page {
        Page::Plain => KIND_PLAIN,
        Page::List { .. } => KIND_LIST,
        Page::Stats { .. } => KIND_STATS,
        Page::Suggest { .. } => KIND_SUGGEST,
        Page::Table { .. } => KIND_TABLE,
        Page::Chapter { .. } => KIND_CHAPTER,
        Page::ChapterDetail { .. } => KIND_CHAPTER_DETAIL,
        Page::Song { .. } => KIND_SONG,
        Page::Rand { .. } => KIND_RAND,
        Page::Board { .. } => KIND_BOARD,
        Page::BindOk => KIND_BIND_OK,
        Page::Me => KIND_ME,
        Page::Help => KIND_HELP,
        Page::Unbound => KIND_UNBOUND,
        Page::Challenge { .. } => KIND_CHALLENGE,
        Page::UnbindConfirm { .. } => KIND_UNBIND_CONFIRM,
        Page::CleanConfirm { .. } => KIND_CLEAN_CONFIRM,
        Page::LoginTicket { .. } => KIND_LOGIN_TICKET,
    }
}

fn payload_of(page: &Page) -> String {
    match page {
        Page::List { raw, .. }
        | Page::Stats { raw, .. }
        | Page::Suggest { raw, .. }
        | Page::Table { raw, .. } => raw.clone(),
        Page::ChapterDetail { name } => name.clone(),
        Page::Song { title } => title.clone(),
        Page::Rand { raw, title } => format!("{raw}\u{1}{title}"),
        Page::Challenge { raw } => raw.clone(),
        Page::UnbindConfirm { user } | Page::CleanConfirm { user } | Page::LoginTicket { user } => {
            user.clone()
        }
        Page::Board { .. }
        | Page::BindOk
        | Page::Me
        | Page::Help
        | Page::Unbound
        | Page::Plain
        | Page::Chapter { .. } => String::new(),
    }
}

fn escape(text: &str) -> String {
    text.replace('%', "%25").replace(':', "%3A")
}

fn unescape(text: &str) -> String {
    text.replace("%3A", ":").replace("%25", "%")
}

pub fn encode(page: &Page, number: usize) -> String {
    format!(
        "{}:{}:{}",
        kind_of(page),
        number,
        escape(&payload_of(page))
    )
}

pub fn decode(data: &str) -> Option<(String, usize, String)> {
    let mut parts = data.splitn(3, ':');
    let kind = parts.next()?.to_string();
    let number = parts.next()?.parse::<usize>().ok()?;
    let payload = unescape(parts.next().unwrap_or_default());
    Some((kind, number, payload))
}

pub fn page_of(data: &str) -> Option<Page> {
    let (kind, number, payload) = decode(data)?;
    from_parts(&kind, number, payload)
}

fn from_parts(kind: &str, number: usize, payload: String) -> Option<Page> {
    match kind {
        KIND_LIST => Some(Page::List {
            raw: payload,
            page: number,
        }),
        KIND_STATS => Some(Page::Stats {
            raw: payload,
            page: number,
        }),
        KIND_SUGGEST => Some(Page::Suggest {
            raw: payload,
            page: number,
        }),
        KIND_TABLE => Some(Page::Table {
            raw: payload,
            page: number,
        }),
        KIND_PLAIN => Some(Page::Plain),
        KIND_CHAPTER => Some(Page::Chapter {
            chapters: Vec::new(),
        }),
        KIND_CHAPTER_DETAIL => Some(Page::ChapterDetail { name: payload }),
        KIND_SONG => Some(Page::Song { title: payload }),
        KIND_RAND => {
            let (raw, title) = match payload.split_once('\u{1}') {
                Some((raw, title)) => (raw.to_string(), title.to_string()),
                None => (String::new(), payload),
            };
            Some(Page::Rand { raw, title })
        }
        KIND_CHALLENGE => Some(Page::Challenge { raw: payload }),
        KIND_UNBIND_CONFIRM => Some(Page::UnbindConfirm { user: payload }),
        KIND_CLEAN_CONFIRM => Some(Page::CleanConfirm { user: payload }),
        KIND_LOGIN_TICKET => Some(Page::LoginTicket { user: payload }),
        KIND_HELP => Some(Page::Help),
        _ => None,
    }
}

fn with_number(page: &Page, number: usize) -> Option<Page> {
    match page {
        Page::List { raw, .. } => Some(Page::List {
            raw: raw.clone(),
            page: number,
        }),
        Page::Stats { raw, .. } => Some(Page::Stats {
            raw: raw.clone(),
            page: number,
        }),
        Page::Suggest { raw, .. } => Some(Page::Suggest {
            raw: raw.clone(),
            page: number,
        }),
        Page::Table { raw, .. } => Some(Page::Table {
            raw: raw.clone(),
            page: number,
        }),
        _ => None,
    }
}

fn short_of(number: usize) -> usize {
    if number <= 1 {
        STATS_SHORT
    } else {
        STATS_SHORT_MAX
    }
}

fn clamp(number: usize, pages: usize) -> usize {
    let last = pages.max(1);
    number.clamp(1, last)
}

fn opts() -> Vec<aurorix_buttons::Opt> {
    vec![style(ButtonStyle::GraySolid)]
}

fn opts_only(user: &str) -> Vec<aurorix_buttons::Opt> {
    let mut out = opts();
    out.push(aurorix_buttons::only(vec![user.to_string()]));
    out
}

fn finish(builder: Builder) -> Option<Keyboard> {
    let keyboard = builder.build()?;
    match keyboard.validate() {
        Ok(()) => Some(keyboard),
        Err(_) => None,
    }
}

pub fn keyboard(page: Page, ctx: &ButtonCtx) -> Option<Keyboard> {
    let pages = ctx.pages.max(1);
    let current = clamp(ctx.page, pages);
    let mut builder = Builder::new().scope(SCOPE);
    match &page {
        Page::Plain => {}
        Page::List { .. } => {
            if pages > 1 {
                if current > 1 {
                    if let Some(prev) = with_number(&page, current - 1) {
                        builder = builder.callback(
                            BTN_PREV,
                            LABEL_PREV,
                            &encode(&prev, current - 1),
                            opts(),
                        );
                    }
                }
                if current < pages {
                    if let Some(next) = with_number(&page, current + 1) {
                        builder = builder.callback(
                            BTN_NEXT,
                            LABEL_NEXT,
                            &encode(&next, current + 1),
                            opts(),
                        );
                    }
                }
            }
            builder = builder.callback(
                BTN_REFRESH,
                LABEL_REFRESH,
                &encode(&page, current),
                opts(),
            );
        }
        Page::Stats { .. } => {
            if current < 2 {
                if let Some(next) = with_number(&page, 2) {
                    builder = builder.callback(
                        BTN_NEXT,
                        LABEL_MORE,
                        &encode(&next, 2),
                        opts(),
                    );
                }
            }
            builder = builder.callback(
                BTN_REFRESH,
                LABEL_REFRESH,
                &encode(&page, current),
                opts(),
            );
        }
        Page::Suggest { .. } => {
            builder = builder
                .command(
                    "query.push.again",
                    LABEL_PUSH_AGAIN,
                    &format!("{COMMAND_ROOT}suggest"),
                    opts(),
                )
                .callback(
                    BTN_REFRESH,
                    LABEL_REFRESH,
                    &encode(&page, current),
                    opts(),
                );
        }
        Page::Table { .. } => {
            builder = builder.callback(
                BTN_REFRESH,
                LABEL_REFRESH,
                &encode(&page, current),
                opts(),
            );
        }
        Page::Chapter { chapters } => {
            if !chapters.is_empty() {
                builder = builder.command(
                    "chap.detail",
                    LABEL_CHAP_DETAIL,
                    &format!("{COMMAND_ROOT}chap "),
                    opts(),
                );
            }
        }
        Page::ChapterDetail { .. } => {
            builder = builder
                .callback(
                    BTN_REFRESH,
                    LABEL_REFRESH,
                    &encode(&page, current),
                    opts(),
                )
                .command(
                    "chap.back",
                    LABEL_CHAP_BACK,
                    &format!("{COMMAND_ROOT}chap"),
                    opts(),
                );
        }
        Page::Song { title } => {
            builder = builder
                .command(
                    "song.score",
                    LABEL_SONG_SCORE,
                    &format!("{COMMAND_ROOT}score {title}"),
                    opts(),
                )
                .command(
                    "song.detail",
                    LABEL_SONG_DETAIL,
                    &format!("{COMMAND_ROOT}song {title}"),
                    opts(),
                )
                .command(
                    "song.play",
                    LABEL_SONG_PLAY,
                    &format!("{COMMAND_ROOT}play {title}"),
                    opts(),
                )
                .callback(
                    BTN_REFRESH,
                    LABEL_REFRESH,
                    &encode(&page, current),
                    opts(),
                );
        }
        Page::Rand { raw, title } => {
            builder = builder
                .command(
                    "song.rand.again",
                    LABEL_RAND_AGAIN,
                    &format!("{COMMAND_ROOT}rand {raw}"),
                    opts(),
                )
                .command(
                    "song.detail",
                    LABEL_SONG_DETAIL,
                    &format!("{COMMAND_ROOT}song {title}"),
                    opts(),
                );
        }
        Page::Board { size } => {
            let target = if *size > BOARD_SMALL {
                BOARD_SMALL
            } else {
                BOARD_LARGE
            };
            let label = if *size > BOARD_SMALL {
                LABEL_BEST30
            } else {
                LABEL_BEST50
            };
            builder = builder
                .callback(
                    BTN_RECHECK,
                    LABEL_RECHECK,
                    &encode(&Page::Board { size: *size }, current),
                    opts(),
                )
                .callback(
                    BTN_BOARD_SWITCH,
                    label,
                    &encode(&Page::Board { size: target }, 1),
                    opts(),
                );
        }
        Page::BindOk => {
            builder = builder
                .command(
                    "account.best30",
                    LABEL_BEST30,
                    &format!("{COMMAND_ROOT}b30"),
                    opts(),
                )
                .command(
                    "account.best50",
                    LABEL_BEST50,
                    &format!("{COMMAND_ROOT}b50"),
                    opts(),
                )
                .command(
                    "account.single",
                    LABEL_SINGLE_ENTRY,
                    &format!("{COMMAND_ROOT}score "),
                    opts(),
                )
                .command(
                    "query.push",
                    LABEL_PUSH_AGAIN,
                    &format!("{COMMAND_ROOT}suggest"),
                    opts(),
                );
        }
        Page::Me => {
            builder = builder.command(
                "account.best30",
                LABEL_BEST30,
                &format!("{COMMAND_ROOT}b30"),
                opts(),
            );
        }
        Page::Help => {
            builder = builder
                .callback(
                    BTN_LOGIN,
                    LABEL_LOGIN,
                    &encode(&Page::Help, 1),
                    opts(),
                )
                .callback(
                    BTN_RECHECK,
                    LABEL_BEST30,
                    &encode(&Page::Board { size: BOARD_SMALL }, 1),
                    opts(),
                );
        }
        Page::Unbound => {
            builder = builder
                .callback(
                    BTN_LOGIN,
                    LABEL_LOGIN,
                    &encode(&Page::Unbound, 1),
                    opts(),
                )
                .command(
                    "account.unbound.bind",
                    LABEL_BIND_ENTRY,
                    &format!("{COMMAND_ROOT}bind "),
                    opts(),
                );
        }
        Page::Challenge { raw } => {
            let raw = raw.trim();
            builder = builder.callback(
                BTN_CHALLENGE_AGAIN,
                LABEL_CHALLENGE_AGAIN,
                &encode(&Page::Challenge { raw: raw.to_string() }, current),
                opts(),
            );
        }
        Page::UnbindConfirm { user } => {
            builder = builder
                .callback(
                    BTN_UNBIND_CONFIRM,
                    LABEL_UNBIND_CONFIRM,
                    &encode(&page, current),
                    opts_only(user),
                )
                .callback(
                    BTN_UNBIND_CANCEL,
                    LABEL_CANCEL,
                    &encode(&page, current),
                    opts_only(user),
                );
        }
        Page::CleanConfirm { user } => {
            builder = builder
                .callback(
                    BTN_CLEAN_CONFIRM,
                    LABEL_CLEAN_CONFIRM,
                    &encode(&page, current),
                    opts_only(user),
                )
                .callback(
                    BTN_CLEAN_CANCEL,
                    LABEL_CANCEL,
                    &encode(&page, current),
                    opts_only(user),
                );
        }
        Page::LoginTicket { user } => {
            builder = builder.callback(
                BTN_LOGIN_CANCEL,
                LABEL_LOGIN_CANCEL,
                &encode(&page, current),
                opts_only(user),
            );
        }
    }
    builder = builder.callback(BTN_MENU, LABEL_MENU, "menu", opts());
    finish(builder)
}

pub fn install(service: &Arc<PhigrosService>) -> usize {
    static ONCE: OnceLock<()> = OnceLock::new();
    if ONCE.set(()).is_err() {
        return 0;
    }
    let book = CallbackBook::global();
    let mut registered = 0;
    for suffix in CALLBACK_SUFFIXES {
        let id = callback_id(suffix);
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

async fn dispatch(service: &Arc<PhigrosService>, request: &CallbackRequest) -> Option<Outgoing> {
    let suffix = request
        .button_id
        .strip_prefix(SCOPE)
        .and_then(|rest| rest.strip_prefix('.'))?;
    match suffix {
        BTN_PREV | BTN_NEXT | BTN_REFRESH | BTN_RECHECK | BTN_BOARD_SWITCH | BTN_CHALLENGE_AGAIN => {
            let page = page_of(&request.data)?;
            Some(render(service, &request.user, page).await)
        }
        BTN_MENU => Some(menu(service)),
        BTN_UNBIND_CONFIRM => {
            confirm_dispatch(service, &request.data, &request.user, ConfirmAct::UnbindApply).await
        }
        BTN_UNBIND_CANCEL => {
            confirm_dispatch(service, &request.data, &request.user, ConfirmAct::UnbindCancel).await
        }
        BTN_CLEAN_CONFIRM => {
            confirm_dispatch(service, &request.data, &request.user, ConfirmAct::CleanApply).await
        }
        BTN_CLEAN_CANCEL => {
            confirm_dispatch(service, &request.data, &request.user, ConfirmAct::CleanCancel).await
        }
        BTN_LOGIN => {
            Some(
                crate::cmd_account::login_click(
                    service,
                    &request.user,
                    request.scene,
                    request.sender.clone(),
                )
                .await,
            )
        }
        BTN_LOGIN_CANCEL => login_cancel(service, &request.data, &request.user).await,
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfirmAct {
    UnbindApply,
    UnbindCancel,
    CleanApply,
    CleanCancel,
}

fn denied() -> Outgoing {
    Outgoing::new(ReplyKind::Denied, DENIED_NOT_MINE.to_string())
}

async fn confirm_dispatch(
    service: &Arc<PhigrosService>,
    data: &str,
    user: &str,
    act: ConfirmAct,
) -> Option<Outgoing> {
    let page = page_of(data)?;
    let owner = match &page {
        Page::UnbindConfirm { user } | Page::CleanConfirm { user } => user.clone(),
        _ => return None,
    };
    if owner != user {
        return Some(denied());
    }
    Some(match act {
        ConfirmAct::UnbindApply => crate::cmd_account::unbind_apply(service, user).await,
        ConfirmAct::UnbindCancel => crate::cmd_account::unbind_cancel(),
        ConfirmAct::CleanApply => crate::cmd_account::clean_apply(service, user).await,
        ConfirmAct::CleanCancel => crate::cmd_account::clean_cancel(),
    })
}

async fn login_cancel(
    service: &Arc<PhigrosService>,
    data: &str,
    user: &str,
) -> Option<Outgoing> {
    let page = page_of(data)?;
    let Page::LoginTicket { user: owner } = page else {
        return None;
    };
    if owner != user {
        return Some(denied());
    }
    Some(crate::cmd_account::login_cancel(service, user))
}

pub fn menu(service: &Arc<PhigrosService>) -> Outgoing {
    with_keyboard(service.text("help", &Args::new()), Page::Help)
}

pub async fn render(service: &Arc<PhigrosService>, user: &str, page: Page) -> Outgoing {
    match page {
        Page::Plain => menu(service),
        Page::List { raw, page } => crate::cmd_query::list_out(service, user, &raw, page).await,
        Page::Stats { raw, page } => {
            crate::cmd_query::stats_out(service, user, &raw, short_of(page)).await
        }
        Page::Suggest { raw, .. } => crate::cmd_query::suggest_out(service, user, &raw).await,
        Page::Table { raw, page } => crate::cmd_query::table_out(service, user, &raw, page).await,
        Page::Chapter { .. } => crate::cmd_query::chapter_out(service, user).await,
        Page::ChapterDetail { name } => {
            crate::cmd_query::chapter_detail_out(service, user, &name).await
        }
        Page::Song { title } => crate::cmd_song::song_out(service, user, &title).await,
        Page::Rand { raw, .. } => crate::cmd_song::random_out(service, user, &raw).await,
        Page::Board { size } => crate::cmd_score::board_out(service, user, size).await,
        Page::Challenge { raw } => crate::cmd_score::challenge_out(service, user, &raw).await,
        Page::BindOk => crate::cmd_account::me_out(service, user).await,
        Page::Me => crate::cmd_account::me_out(service, user).await,
        Page::UnbindConfirm { .. } => crate::cmd_account::unbind_confirm(service, user).await,
        Page::CleanConfirm { .. } => crate::cmd_account::clean_confirm(service, user).await,
        Page::Unbound => menu(service),
        Page::Help => menu(service),
        Page::LoginTicket { user } => crate::cmd_account::login_ticket(service, &user).await,
}
}

pub fn with_keyboard(out: Outgoing, page: Page) -> Outgoing {
    match keyboard(page, &ButtonCtx::default()) {
        Some(keyboard) => out.keyboard(keyboard),
        None => out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_buttons::{AllowedPermission, ButtonAction, MAX_BUTTONS_PER_ROW, MAX_ROWS};

    fn every_page() -> Vec<Page> {
        vec![
            Page::Plain,
            Page::List {
                raw: String::from("定数>14 章节:霓虹灯牌"),
                page: 2,
            },
            Page::Stats {
                raw: String::from("AT"),
                page: 1,
            },
            Page::Suggest {
                raw: String::from("15+"),
                page: 1,
            },
            Page::Table {
                raw: String::from("14"),
                page: 1,
            },
            Page::Chapter {
                chapters: vec![String::from("霓虹灯牌"), String::from("方舟蜃景")],
            },
            Page::ChapterDetail {
                name: String::from("霓虹灯牌"),
            },
            Page::Song {
                title: String::from("Spasmodic"),
            },
            Page::Rand {
                raw: String::from("定数>15"),
                title: String::from("Igallta"),
            },
            Page::UnbindConfirm {
                user: String::from("10001"),
            },
            Page::CleanConfirm {
                user: String::from("10001"),
            },
            Page::LoginTicket {
                user: String::from("10001"),
            },
        ]
    }

    fn ids(keyboard: &Keyboard) -> Vec<String> {
        keyboard
            .flatten()
            .into_iter()
            .map(|button| button.id)
            .collect()
    }

    #[test]
    fn encode_decode_roundtrips_every_page() {
        for page in every_page() {
            let data = encode(&page, 1);
            let back = page_of(&data).expect("回调数据要能还原页面");
            assert_eq!(kind_of(&back), kind_of(&page));
            assert_eq!(payload_of(&back), payload_of(&page));
        }
        let raw = Page::List {
            raw: String::from("a:b 100% 定数:15"),
            page: 4,
        };
        let data = encode(&raw, 4);
        assert_eq!(data, String::from("list:4:a%3Ab 100%25 定数%3A15"));
        let back = page_of(&data).expect("冒号与百分号要能还原");
        assert_eq!(back, raw);
        assert!(page_of("nope").is_none());
        assert!(page_of("list:notanumber:x").is_none());
        assert!(page_of("unknownkind:1:x").is_none());
    }

    #[test]
    fn keyboard_fits_platform_limits_and_ids_are_unique() {
        for page in every_page() {
            let ctx = ButtonCtx { page: 2, pages: 3 };
            let board = keyboard(page.clone(), &ctx).expect("每一页都该有按钮");
            assert!(board.validate().is_ok(), "按钮板不合法: {page:?}");
            assert!(board.rows.len() <= MAX_ROWS);
            for row in &board.rows {
                assert!(!row.buttons.is_empty(), "空行没有意义: {page:?}");
                assert!(row.buttons.len() <= MAX_BUTTONS_PER_ROW);
            }
            let list = ids(&board);
            let mut unique = list.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(list.len(), unique.len(), "按钮 id 重复: {page:?}");
            for id in &list {
                assert!(id.starts_with("phigros."), "{id} 少了插件前缀");
            }
            assert!(
                list.contains(&callback_id(BTN_MENU)),
                "结果卡都要挂 PGR菜单: {page:?}"
            );
        }
    }

    #[test]
    fn pagination_follows_page_bounds() {
        let page = Page::List {
            raw: String::from("定数>14"),
            page: 1,
        };
        let first = keyboard(page.clone(), &ButtonCtx { page: 1, pages: 3 }).expect("首页按钮");
        assert!(!first.has_button(&callback_id(BTN_PREV)));
        assert!(first.has_button(&callback_id(BTN_NEXT)));
        let last = keyboard(page.clone(), &ButtonCtx { page: 3, pages: 3 }).expect("末页按钮");
        assert!(last.has_button(&callback_id(BTN_PREV)));
        assert!(!last.has_button(&callback_id(BTN_NEXT)));
        let only = keyboard(page, &ButtonCtx { page: 9, pages: 1 }).expect("单页按钮");
        assert!(!only.has_button(&callback_id(BTN_PREV)));
        assert!(!only.has_button(&callback_id(BTN_NEXT)));
        assert_eq!(clamp(0, 0), 1);
        assert_eq!(clamp(9, 4), 4);
        assert_eq!(short_of(1), STATS_SHORT);
        assert_eq!(short_of(2), STATS_SHORT_MAX);
        let stats = Page::Stats {
            raw: String::new(),
            page: 1,
        };
        let board = keyboard(stats, &ButtonCtx { page: 1, pages: 1 }).expect("统计按钮");
        assert!(board.has_button(&callback_id(BTN_NEXT)), "统计页要能看更多");
    }

    #[test]
    fn command_buttons_carry_the_song_title() {
        let board = keyboard(
            Page::Song {
                title: String::from("Spasmodic"),
            },
            &ButtonCtx { page: 1, pages: 1 },
        )
        .expect("曲目按钮");
        let commands: Vec<String> = board
            .flatten()
            .into_iter()
            .filter(|button| button.action.action_type == ButtonAction::Command)
            .filter_map(|button| button.action.msg)
            .collect();
        assert_eq!(
            commands,
            vec![
                String::from("/p score Spasmodic"),
                String::from("/p song Spasmodic"),
                String::from("/p play Spasmodic"),
            ]
        );
        let rand = keyboard(
            Page::Rand {
                raw: String::from("定数>15"),
                title: String::from("Igallta"),
            },
            &ButtonCtx { page: 1, pages: 1 },
        )
        .expect("随机按钮");
        assert!(rand.has_button("phigros.song.rand.again"));
        let detail = keyboard(
            Page::ChapterDetail {
                name: String::from("霓虹灯牌"),
            },
            &ButtonCtx { page: 1, pages: 1 },
        )
        .expect("章节详情按钮");
        assert!(detail.has_button("phigros.chap.back"));
    }

    #[test]
    fn callback_buttons_are_always_non_empty_and_gray() {
        for page in every_page() {
            let board = keyboard(page, &ButtonCtx { page: 1, pages: 1 }).expect("按钮板");
            for button in board.flatten() {
                assert_eq!(button.render_data.style, ButtonStyle::GraySolid);
                if button.action.action_type == ButtonAction::Callback {
                    let data = button.action.callback_data.unwrap_or_default();
                    assert!(!data.is_empty(), "{} 的回调数据不能为空", button.id);
                }
            }
        }
        assert_eq!(callback_ids().len(), CALLBACK_SUFFIXES.len());
        assert!(callback_ids().contains(&String::from("phigros.account.unbound.login")));
        assert!(callback_ids().contains(&String::from("phigros.account.unbind.confirm")));
        assert!(callback_ids().contains(&String::from("phigros.account.clean.cancel")));
        assert!(callback_ids().contains(&String::from("phigros.account.login.cancel")));
        assert!(callback_ids().contains(&String::from("phigros.page.refresh")));
        assert!(callback_ids().contains(&String::from("phigros.score.recheck")));
        assert!(callback_ids().contains(&String::from("phigros.score.b30.switch")));
        assert!(callback_ids().contains(&String::from("phigros.challenge.again")));
    }
    #[test]
    fn account_and_board_pages_carry_their_entries() {
        let commands = |page: Page| -> Vec<String> {
            keyboard(page, &ButtonCtx { page: 1, pages: 1 })
                .expect("按钮板")
                .flatten()
                .into_iter()
                .filter(|button| button.action.action_type == ButtonAction::Command)
                .filter_map(|button| button.action.msg)
                .collect()
        };
        let callbacks = |page: Page| -> Vec<String> {
            keyboard(page, &ButtonCtx { page: 1, pages: 1 })
                .expect("按钮板")
                .flatten()
                .into_iter()
                .filter(|button| button.action.action_type == ButtonAction::Callback)
                .map(|button| button.id)
                .collect()
        };
        let bound = commands(Page::BindOk);
        for want in [
            "/p b30",
            "/p b50",
            "/p score ",
            "/p suggest",
        ] {
            assert!(
                bound.iter().any(|cmd| cmd == want),
                "绑定成功页缺 {want}: {bound:?}"
            );
        }
        let board = callbacks(Page::Board { size: 50 });
        assert!(board.iter().any(|id| id == &callback_id(BTN_RECHECK)));
        assert!(board.iter().any(|id| id == &callback_id(BTN_BOARD_SWITCH)));
        let unbound = commands(Page::Unbound);
        assert!(
            !unbound.iter().any(|cmd| cmd == "/p login"),
            "登录按钮已改为回调, 不该再发指令"
        );
        assert!(unbound.iter().any(|cmd| cmd == "/p bind "));
        let unbound_clicks = callbacks(Page::Unbound);
        assert!(
            unbound_clicks
                .iter()
                .any(|id| id == &callback_id(BTN_LOGIN)),
            "未绑定页要有登录回调按钮: {unbound_clicks:?}"
        );
        let me = keyboard(Page::Me, &ButtonCtx::default()).expect("总览按钮");
        assert!(me.has_button(&callback_id(BTN_MENU)));
    }
    #[test]
    fn sensitive_buttons_are_limited_to_their_owner() {
        let owner = String::from("10001");
        for page in [
            Page::UnbindConfirm { user: owner.clone() },
            Page::CleanConfirm { user: owner.clone() },
            Page::LoginTicket { user: owner.clone() },
        ] {
            let board = keyboard(page, &ButtonCtx::default()).expect("确认板");
            let guarded: Vec<_> = board
                .flatten()
                .into_iter()
                .filter(|button| button.id != callback_id(BTN_MENU))
                .collect();
            assert!(!guarded.is_empty());
            for button in guarded {
                assert_eq!(
                    button.action.permission.permission_type,
                    AllowedPermission::SomeUser
                );
                assert_eq!(
                    button.action.permission.specify_user_ids,
                    vec![owner.clone()]
                );
            }
        }
    }

}
