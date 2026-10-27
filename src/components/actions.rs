use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon},
    entities::{chess_board::signals::ChessBoardSignals, protocol::ClientMsg},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActionsLayout {
    /// The phone's bottom bar
    Bar,
    /// The desktop's side panel
    Panel,
}

#[derive(Clone, PartialEq)]
enum Run {
    Send(ClientMsg),
    Flip,
    Resign,
    Open(Form),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Normal,
    Danger,
    Ghost,
}

#[derive(Clone, PartialEq)]
struct Action {
    label: &'static str,
    icon: Icon,
    kind: Kind,
    run: Run,
}

fn action(label: &'static str, icon: Icon, kind: Kind, run: Run) -> Action {
    Action {
        label,
        icon,
        kind,
        run,
    }
}

/// What the user can do with the board right now: game actions for a seated
/// player, a free board's undo and redo, and flipping for everyone
#[component]
pub fn GameActions(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>, layout: ActionsLayout) -> impl IntoView {
    // a first tap on "Resign" asks for a second one
    let confirm_resign = RwSignal::new(false);

    let actions = move || {
        let game = chess_board_signals.game().get();
        let seat = chess_board_signals.my_seat();
        let flip = action("Flip", Icon::Flip, Kind::Ghost, Run::Flip);
        let invite = action("Invite", Icon::Invite, Kind::Normal, Run::Open(Form::Invite));

        match game {
            Some(game) if seat.is_some() && game.result.is_none() => {
                let draw_label = match (game.draw_offer, seat) {
                    (Some(offer), Some(seat)) if offer == seat => "Draw offered",
                    (Some(_), _) => "Accept draw",
                    _ => "Offer draw",
                };
                let draw_msg = if draw_label == "Accept draw" {
                    ClientMsg::AcceptDraw
                } else {
                    ClientMsg::OfferDraw
                };
                let resign_label = if confirm_resign.get() {
                    "Confirm resign"
                } else {
                    "Resign"
                };
                vec![
                    flip,
                    action(draw_label, Icon::Draw, Kind::Normal, Run::Send(draw_msg)),
                    action(resign_label, Icon::Resign, Kind::Danger, Run::Resign),
                    invite,
                ]
            }
            Some(_) => vec![
                flip,
                action("New game", Icon::NewGame, Kind::Normal, Run::Open(Form::NewGame)),
                invite,
            ],
            None => vec![
                flip,
                action("Undo", Icon::Undo, Kind::Normal, Run::Send(ClientMsg::Undo)),
                action("Redo", Icon::Redo, Kind::Normal, Run::Send(ClientMsg::Redo)),
                action("Reset", Icon::Reset, Kind::Normal, Run::Send(ClientMsg::Reset)),
            ],
        }
    };

    let run = move |run: Run| match run {
        Run::Send(msg) => chess_board_signals.send(&msg),
        Run::Flip => chess_board_signals.chess_board().update(|board| board.flip()),
        Run::Resign => {
            if confirm_resign.get_untracked() {
                confirm_resign.set(false);
                chess_board_signals.send(&ClientMsg::Resign);
            } else {
                confirm_resign.set(true);
            }
        }
        Run::Open(form) => show_form.set(form),
    };

    let container_class = match layout {
        ActionsLayout::Bar => "action-bar lg:hidden",
        ActionsLayout::Panel => "action-panel",
    };

    view! {
        <div class=container_class>
            <For
                each=actions
                key=|action| action.label
                children=move |action| {
                    let class = match action.kind {
                        Kind::Normal => "action",
                        Kind::Danger => "action action--danger",
                        Kind::Ghost => "action action--ghost",
                    };
                    let run_action = action.run.clone();
                    view! {
                        <button class=class on:click=move |_| run(run_action.clone())>
                            {action.icon.view()}
                            <span>{action.label}</span>
                        </button>
                    }
                }
            />
        </div>
    }
}
