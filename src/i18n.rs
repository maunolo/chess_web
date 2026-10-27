//! Interface text in English and Brazilian Portuguese.
//!
//! The language comes from the browser: the server reads `Accept-Language`
//! when it renders the page and writes the choice on `<html lang>`, and the
//! client reads that attribute when it hydrates, so both render the same
//! text. Components read it once from context with [`current`].

use leptos::prelude::*;

use crate::entities::protocol::{GameResult, Side, Text, TimeControl, MAIN_ROOM};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    PtBr,
}

/// Every piece of static interface text
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Menu,
    NewGame,
    JoinRoom,
    Invite,
    CasualGame,
    FreeBoard,
    Game,
    InTheRoom,
    ChangeName,
    Reconnect,
    Watching,
    You,
    BoardOptions,
    FlipBoard,
    Flip,
    OfferDraw,
    DrawOffered,
    AcceptDraw,
    Resign,
    ConfirmResign,
    Undo,
    Redo,
    Reset,
    Options,
    Moves,
    NoMovesYet,
    Waiting,
    Player,
    WaitingForOpponent,
    OpponentOffersDraw,
    Accept,
    Decline,
    YouWon,
    YouLost,
    GameOver,
    Draw,
    RematchOffered,
    AcceptRematch,
    Rematch,
    Close,
    Checkmate,
    TheWinnerIs,
    ResetBoard,
    UndoLastMove,
    PromoteTo,
    Queen,
    Rook,
    Bishop,
    Knight,
    JoinARoom,
    JoinHint,
    JoinPlaceholder,
    Join,
    WhatsYourName,
    NameHint,
    NamePlaceholder,
    Continue,
    Validation,
    ValidationHint,
    Turns,
    TurnsHint,
    Apply,
    PlayAs,
    White,
    Random,
    Black,
    Clock,
    NoClock,
    CreateGame,
    InviteGameHint,
    InviteBoardHint,
    CopyLink,
    Copied,
    Share,
    ShareTitle,
    Bullet,
    Blitz,
    Rapid,
    Classical,
}

impl Key {
    fn texts(self) -> (&'static str, &'static str) {
        match self {
            Key::Menu => ("Menu", "Menu"),
            Key::NewGame => ("New game", "Nova partida"),
            Key::JoinRoom => ("Join room", "Entrar em sala"),
            Key::Invite => ("Invite", "Convidar"),
            Key::CasualGame => ("Casual game", "Partida casual"),
            Key::FreeBoard => ("Free board", "Tabuleiro livre"),
            Key::Game => ("Game", "Partida"),
            Key::InTheRoom => ("In the room", "Na sala"),
            Key::ChangeName => ("Change name", "Mudar nome"),
            Key::Reconnect => ("Reconnect", "Reconectar"),
            Key::Watching => ("Watching", "Assistindo"),
            Key::You => ("you", "você"),
            Key::BoardOptions => ("Board options", "Opções do tabuleiro"),
            Key::FlipBoard => ("Flip board", "Girar tabuleiro"),
            Key::Flip => ("Flip", "Girar"),
            Key::OfferDraw => ("Offer draw", "Propor empate"),
            Key::DrawOffered => ("Draw offered", "Empate proposto"),
            Key::AcceptDraw => ("Accept draw", "Aceitar empate"),
            Key::Resign => ("Resign", "Desistir"),
            Key::ConfirmResign => ("Confirm resign", "Confirmar desistência"),
            Key::Undo => ("Undo", "Desfazer"),
            Key::Redo => ("Redo", "Refazer"),
            Key::Reset => ("Reset", "Reiniciar"),
            Key::Options => ("Options", "Opções"),
            Key::Moves => ("Moves", "Lances"),
            Key::NoMovesYet => ("No moves yet", "Nenhum lance ainda"),
            Key::Waiting => ("Waiting…", "Aguardando…"),
            Key::Player => ("Player", "Jogador"),
            Key::WaitingForOpponent => ("Waiting for an opponent", "Aguardando um adversário"),
            Key::OpponentOffersDraw => ("Your opponent offers a draw", "Seu adversário propõe empate"),
            Key::Accept => ("Accept", "Aceitar"),
            Key::Decline => ("Decline", "Recusar"),
            Key::YouWon => ("You won!", "Você venceu!"),
            Key::YouLost => ("You lost", "Você perdeu"),
            Key::GameOver => ("Game over", "Fim de jogo"),
            Key::Draw => ("Draw", "Empate"),
            Key::RematchOffered => ("Rematch offered", "Revanche proposta"),
            Key::AcceptRematch => ("Accept rematch", "Aceitar revanche"),
            Key::Rematch => ("Rematch", "Revanche"),
            Key::Close => ("Close", "Fechar"),
            Key::Checkmate => ("Checkmate!", "Xeque-mate!"),
            Key::TheWinnerIs => ("The winner is", "O vencedor é"),
            Key::ResetBoard => ("Reset board", "Reiniciar tabuleiro"),
            Key::UndoLastMove => ("Undo last move", "Desfazer último lance"),
            Key::PromoteTo => ("Promote to", "Promover a"),
            Key::Queen => ("Queen", "Dama"),
            Key::Rook => ("Rook", "Torre"),
            Key::Bishop => ("Bishop", "Bispo"),
            Key::Knight => ("Knight", "Cavalo"),
            Key::JoinARoom => ("Join a room", "Entrar em uma sala"),
            Key::JoinHint => (
                "A game's code, or any name for a free board. A FEN after the name starts a new free board from that position.",
                "O código de uma partida, ou qualquer nome para um tabuleiro livre. Um FEN depois do nome inicia um tabuleiro livre nessa posição.",
            ),
            Key::JoinPlaceholder => ("Room name or game code", "Nome da sala ou código da partida"),
            Key::Join => ("Join", "Entrar"),
            Key::WhatsYourName => ("What's your name?", "Qual é o seu nome?"),
            Key::NameHint => (
                "Shown to the other players in the room.",
                "Visível para os outros jogadores na sala.",
            ),
            Key::NamePlaceholder => ("Ex: John Doe", "Ex: João Silva"),
            Key::Continue => ("Continue", "Continuar"),
            Key::Validation => ("Validation", "Validação"),
            Key::ValidationHint => (
                "Only legal moves, with check and mate",
                "Só lances legais, com xeque e mate",
            ),
            Key::Turns => ("Turns", "Turnos"),
            Key::TurnsHint => ("White and black alternate", "Brancas e pretas se alternam"),
            Key::Apply => ("Apply", "Aplicar"),
            Key::PlayAs => ("Play as", "Jogar com"),
            Key::White => ("White", "Brancas"),
            Key::Random => ("Random", "Aleatório"),
            Key::Black => ("Black", "Pretas"),
            Key::Clock => ("Clock", "Relógio"),
            Key::NoClock => ("No clock", "Sem relógio"),
            Key::CreateGame => ("Create game", "Criar partida"),
            Key::InviteGameHint => (
                "Whoever opens this link first takes the free seat; everyone after that watches.",
                "Quem abrir este link primeiro ocupa a vaga livre; os demais assistem.",
            ),
            Key::InviteBoardHint => (
                "Anyone with this link joins the board.",
                "Qualquer pessoa com este link entra no tabuleiro.",
            ),
            Key::CopyLink => ("Copy link", "Copiar link"),
            Key::Copied => ("Copied!", "Copiado!"),
            Key::Share => ("Share", "Compartilhar"),
            Key::ShareTitle => ("Play chess with me", "Jogue xadrez comigo"),
            Key::Bullet => ("Bullet", "Bullet"),
            Key::Blitz => ("Blitz", "Blitz"),
            Key::Rapid => ("Rapid", "Rápida"),
            Key::Classical => ("Classical", "Clássica"),
        }
    }
}

impl Lang {
    /// The first language we have in a BCP 47 tag or an `Accept-Language`
    /// header (`pt-BR,pt;q=0.9,en;q=0.8`), English when there is none
    pub fn from_tag(tag: &str) -> Lang {
        for part in tag.split(',') {
            let tag = part
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if tag == "pt" || tag.starts_with("pt-") {
                return Lang::PtBr;
            }
            if tag == "en" || tag.starts_with("en-") || tag == "*" {
                return Lang::En;
            }
        }
        Lang::En
    }

    /// What goes on `<html lang>`
    pub fn tag(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
        }
    }

    pub fn t(self, key: Key) -> &'static str {
        let (en, pt) = key.texts();
        match self {
            Lang::En => en,
            Lang::PtBr => pt,
        }
    }

    pub fn side(self, side: Side) -> &'static str {
        match side {
            Side::White => self.t(Key::White),
            Side::Black => self.t(Key::Black),
        }
    }

    /// A room's name as a title: the main room by its name, the others capitalized
    pub fn room_name(self, room: &str) -> String {
        if room == MAIN_ROOM {
            return match self {
                Lang::En => "Main",
                Lang::PtBr => "Principal",
            }
            .to_string();
        }
        let mut name = room.to_string();
        if let Some(first) = name.get_mut(..1) {
            first.make_ascii_uppercase();
        }
        name
    }

    pub fn game_title(self, room: &str) -> String {
        format!("{} {}", self.t(Key::Game), room)
    }

    pub fn in_the_room(self, count: usize) -> String {
        format!("{} · {}", self.t(Key::InTheRoom), count)
    }

    /// `3+2 · Blitz`
    pub fn time_control(self, time_control: &TimeControl) -> String {
        let category = match time_control.initial_secs {
            secs if secs < 180 => Key::Bullet,
            secs if secs < 600 => Key::Blitz,
            secs if secs < 1800 => Key::Rapid,
            _ => Key::Classical,
        };
        format!("{} · {}", time_control.label(), self.t(category))
    }

    /// The accessible label of a player card
    pub fn card_label(self, side: Side, to_move: bool, me: bool) -> String {
        let side = self.side(side);
        match (self, to_move, me) {
            (_, false, _) => side.to_string(),
            (Lang::En, true, true) => format!("{side}, your move"),
            (Lang::En, true, false) => format!("{side}, to move"),
            (Lang::PtBr, true, true) => format!("{side}, sua vez"),
            (Lang::PtBr, true, false) => format!("{side}, a jogar"),
        }
    }

    pub fn result(self, result: GameResult) -> String {
        let wins = |winner: &Side, how: (&str, &str)| match self {
            Lang::En => format!("{} wins {}", self.side(*winner), how.0),
            Lang::PtBr => format!("{} vencem {}", self.side(*winner), how.1),
        };
        let pick = |en: &str, pt: &str| match self {
            Lang::En => en.to_string(),
            Lang::PtBr => pt.to_string(),
        };
        match &result {
            GameResult::Checkmate { winner } => wins(winner, ("by checkmate", "por xeque-mate")),
            GameResult::Resignation { winner } => {
                wins(winner, ("by resignation", "por desistência"))
            }
            GameResult::Timeout { winner } => wins(winner, ("on time", "no tempo")),
            GameResult::Stalemate => pick("Draw by stalemate", "Empate por afogamento"),
            GameResult::Agreement => pick("Draw by agreement", "Empate por acordo"),
            GameResult::InsufficientMaterial => pick(
                "Draw by insufficient material",
                "Empate por material insuficiente",
            ),
            GameResult::TimeoutVsInsufficientMaterial => pick(
                "Draw: time out with no mating material",
                "Empate: tempo esgotado sem material para dar mate",
            ),
            GameResult::Repetition => pick(
                "Draw by threefold repetition",
                "Empate por tripla repetição",
            ),
            GameResult::FiftyMoves => pick(
                "Draw by the fifty-move rule",
                "Empate pela regra dos cinquenta lances",
            ),
        }
    }

    /// A notification from the server, or one the client raises itself
    pub fn message(self, text: &Text) -> String {
        let pick = |en: &str, pt: &str| match self {
            Lang::En => en.to_string(),
            Lang::PtBr => pt.to_string(),
        };
        match text {
            Text::JoinedGame { side } => match self {
                Lang::En => format!("Joined the game as {}", self.side(*side)),
                Lang::PtBr => format!("Você entrou na partida com as {}", self.side(*side)),
            },
            Text::WatchingGame => pick("Watching the game", "Assistindo à partida"),
            Text::JoinedRoom { room } => match self {
                Lang::En => format!("Joined room {room}"),
                Lang::PtBr => format!("Você entrou na sala {room}"),
            },
            Text::InvalidRoomName => pick("Invalid room name", "Nome de sala inválido"),
            Text::FailedToCreateRoom => {
                pick("Failed to create room", "Não foi possível criar a sala")
            }
            Text::NameRequired => pick("A name is required", "É preciso um nome"),
            Text::Spectator => pick(
                "You are watching this game",
                "Você está assistindo a esta partida",
            ),
            Text::WaitingForOpponent => pick(
                "Waiting for an opponent to join",
                "Aguardando um adversário entrar",
            ),
            Text::NotYourTurn => pick("It's not your turn", "Não é a sua vez"),
            Text::GameOver => pick("The game is over", "A partida acabou"),
            Text::NotDuringGame => pick(
                "Not available during a game",
                "Indisponível durante uma partida",
            ),
            Text::FailedToReset => pick(
                "Failed to reset board",
                "Não foi possível reiniciar o tabuleiro",
            ),
            Text::NoMoreToUndo => pick(
                "No more moves to undo",
                "Não há mais lances para desfazer",
            ),
            Text::NoMoreToRedo => pick(
                "No more moves to redo",
                "Não há mais lances para refazer",
            ),
            Text::FailedToUndo => pick("Failed to undo move", "Não foi possível desfazer o lance"),
            Text::FailedToRedo => pick("Failed to redo move", "Não foi possível refazer o lance"),
            Text::OptionsApplied => pick("Options applied", "Opções aplicadas"),
            Text::FailedToApplyOptions => pick(
                "Failed to apply options",
                "Não foi possível aplicar as opções",
            ),
            Text::InvalidTimeControl => pick("Invalid time control", "Controle de tempo inválido"),
            Text::NoGameOnFreeBoard => pick(
                "There is no game on a free board",
                "Não há partida em um tabuleiro livre",
            ),
            Text::PlayersOnly => pick(
                "Only the players can do that",
                "Só os jogadores podem fazer isso",
            ),
            Text::NoDrawOffer => pick("There is no draw offer", "Não há proposta de empate"),
            Text::GameStillGoing => pick(
                "The game is still going",
                "A partida ainda está em andamento",
            ),
            Text::GameGone => pick(
                "That game has ended and is no longer available",
                "Essa partida acabou e não está mais disponível",
            ),
            Text::OpenedElsewhere => pick(
                "The board was opened in another tab",
                "O tabuleiro foi aberto em outra aba",
            ),
            Text::Error { message } => match self {
                Lang::En => format!("Error: {message}"),
                Lang::PtBr => format!("Erro: {message}"),
            },
        }
    }
}

/// The language of the page being rendered: the request's `Accept-Language`
/// on the server, `<html lang>` in the browser
pub fn detect() -> Lang {
    #[cfg(feature = "ssr")]
    {
        use_context::<leptos_actix::Request>()
            .and_then(|request| {
                request
                    .headers()
                    .get(actix_web::http::header::ACCEPT_LANGUAGE)?
                    .to_str()
                    .ok()
                    .map(Lang::from_tag)
            })
            .unwrap_or_default()
    }
    #[cfg(not(feature = "ssr"))]
    {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
            .and_then(|html| html.get_attribute("lang"))
            .map(|lang| Lang::from_tag(&lang))
            .unwrap_or_default()
    }
}

/// The language the app was given, see [`crate::app::App`]
pub fn current() -> Lang {
    use_context::<Lang>().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_tag() {
        assert_eq!(Lang::from_tag("pt-BR,pt;q=0.9,en-US;q=0.8"), Lang::PtBr);
        assert_eq!(Lang::from_tag("pt"), Lang::PtBr);
        assert_eq!(Lang::from_tag("PT-PT"), Lang::PtBr);
        assert_eq!(Lang::from_tag("en-US,en;q=0.9,pt;q=0.8"), Lang::En);
        // the first language we have wins, not the first listed
        assert_eq!(Lang::from_tag("es-ES,pt;q=0.8"), Lang::PtBr);
        assert_eq!(Lang::from_tag("es-ES,*;q=0.5"), Lang::En);
        assert_eq!(Lang::from_tag(""), Lang::En);
    }

    #[test]
    fn test_texts() {
        assert_eq!(Lang::PtBr.t(Key::NewGame), "Nova partida");
        assert_eq!(Lang::En.room_name("main"), "Main");
        assert_eq!(Lang::PtBr.room_name("main"), "Principal");
        assert_eq!(Lang::PtBr.room_name("abc"), "Abc");
        assert_eq!(
            Lang::PtBr.result(GameResult::Timeout { winner: Side::Black }),
            "Pretas vencem no tempo"
        );
        assert_eq!(
            Lang::En.message(&Text::JoinedGame { side: Side::White }),
            "Joined the game as White"
        );
        assert_eq!(Lang::PtBr.card_label(Side::White, true, true), "Brancas, sua vez");
        assert_eq!(
            Lang::PtBr.time_control(&TimeControl {
                initial_secs: 600,
                increment_secs: 0
            }),
            "10+0 · Rápida"
        );
    }
}
