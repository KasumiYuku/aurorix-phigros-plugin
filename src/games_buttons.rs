use aurorix_buttons::{style, Builder, ButtonStyle, Keyboard};
use aurorix_plugin::Outgoing;

use crate::cmd_games::Game;

pub const SCOPE: &str = crate::PLUGIN_ID;

pub const COMMAND_ROOT: &str = "/p ";

pub const BTN_AGAIN: &str = "again";

pub const BTN_HINT: &str = "hint";

pub const BTN_REVEAL: &str = "reveal";

pub const BTN_QUICK: &str = "quick";

pub const LABEL_AGAIN: &str = "再来一局";

pub const LABEL_HINT: &str = "查看提示";

pub const LABEL_REVEAL: &str = "停止游戏";

pub const LABEL_EXTEND: &str = "扩展时间";

pub const BTN_EXTEND: &str = "extend";

pub const LABEL_QUICK: &str = "点击猜测";

pub const LABEL_SONG_DETAIL: &str = "该曲图鉴";

pub const LABEL_SONG_SCORE: &str = "该曲成绩";

pub const LABEL_SONG_PLAY: &str = "播放该曲";

pub const LABEL_VIDEO_AGAIN: &str = "再看一段";

pub const VIDEO_BUTTON_ID: &str = "video.again";

fn opts() -> Vec<aurorix_buttons::Opt> {
    vec![style(ButtonStyle::GraySolid)]
}

fn finish(builder: Builder) -> Option<Keyboard> {
    let keyboard = builder.build()?;
    match keyboard.validate() {
        Ok(()) => Some(keyboard),
        Err(_) => None,
    }
}

fn with_keyboard(out: Outgoing, keyboard: Option<Keyboard>) -> Outgoing {
    match keyboard {
        Some(keyboard) => out.keyboard(keyboard),
        None => out,
    }
}

fn suffix(game: Game, action: &str) -> String {
    format!("{}.{}", game.command(), action)
}

fn line(game: Game) -> String {
    format!("{COMMAND_ROOT}{}", game.command())
}

fn live_keyboard(game: Game) -> Option<Keyboard> {
    let base = line(game);
    let mut builder = Builder::new().scope(SCOPE).command(
        &suffix(game, BTN_QUICK),
        LABEL_QUICK,
        &format!("{base} "),
        opts(),
    );
    if matches!(game, Game::Clip) {
        builder = builder.command(
            &suffix(game, BTN_EXTEND),
            LABEL_EXTEND,
            &format!("{base} 延长"),
            opts(),
        );
    }
    builder = builder
        .command(
            &suffix(game, BTN_HINT),
            LABEL_HINT,
            &format!("{base} 提示"),
            opts(),
        )
        .command(
            &suffix(game, BTN_REVEAL),
            LABEL_REVEAL,
            &format!("{base} stop"),
            opts(),
        );
    finish(builder)
}

fn over_keyboard(game: Game, title: &str) -> Option<Keyboard> {
    let base = line(game);
    let mut builder = Builder::new()
        .scope(SCOPE)
        .command(&suffix(game, BTN_AGAIN), LABEL_AGAIN, &format!("{base} "), opts());
    if !title.trim().is_empty() {
        builder = builder
            .row()
            .command(
                "game.detail",
                LABEL_SONG_DETAIL,
                &format!("{COMMAND_ROOT}song {title}"),
                opts(),
            )
            .command(
                "game.score",
                LABEL_SONG_SCORE,
                &format!("{COMMAND_ROOT}score {title}"),
                opts(),
            )
            .command(
                "game.play",
                LABEL_SONG_PLAY,
                &format!("{COMMAND_ROOT}play {title}"),
                opts(),
            );
    }
    finish(builder)
}

fn video_keyboard() -> Option<Keyboard> {
    let builder = Builder::new().scope(SCOPE).command(
        VIDEO_BUTTON_ID,
        LABEL_VIDEO_AGAIN,
        &format!("{COMMAND_ROOT}video "),
        opts(),
    );
    finish(builder)
}

pub fn attach(out: Outgoing, game: Game, live: bool) -> Outgoing {
    let keyboard = if live {
        live_keyboard(game)
    } else {
        over_keyboard(game, "")
    };
    with_keyboard(out, keyboard)
}

pub fn attach_over(out: Outgoing, game: Game, title: &str) -> Outgoing {
    with_keyboard(out, over_keyboard(game, title))
}

pub fn attach_video(out: Outgoing) -> Outgoing {
    with_keyboard(out, video_keyboard())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_buttons::ButtonAction;

    fn texts(keyboard: &Keyboard) -> Vec<String> {
        keyboard
            .flatten()
            .into_iter()
            .filter_map(|button| button.action.msg)
            .collect()
    }

    #[test]
    fn live_board_fills_commands_instead_of_callbacks() {
        for game in Game::ALL {
            let board = live_keyboard(game).expect("进行中的键盘");
            for button in board.flatten() {
                assert_eq!(button.action.action_type, ButtonAction::Command);
            }
            let texts = texts(&board);
            let base = line(game);
            let mut want = vec![format!("{base} ")];
            if matches!(game, Game::Clip) {
                want.push(format!("{base} 延长"));
            }
            want.push(format!("{base} 提示"));
            want.push(format!("{base} stop"));
            assert_eq!(texts, want);
        }
    }

    #[test]
    fn over_board_keeps_song_entries() {
        let board = over_keyboard(Game::Guess, "Fracture Ray").expect("收局键盘");
        let texts = texts(&board);
        assert_eq!(
            texts,
            vec![
                "/p guess ".to_string(),
                "/p song Fracture Ray".to_string(),
                "/p score Fracture Ray".to_string(),
                "/p play Fracture Ray".to_string(),
            ]
        );
        assert!(board
            .flatten()
            .iter()
            .all(|button| button.action.action_type == ButtonAction::Command));
    }
}
