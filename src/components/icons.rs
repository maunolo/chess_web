//! Inline stroke icons, so they take the text color of their button

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Invite,
    People,
    Flip,
    Draw,
    Resign,
    Undo,
    Redo,
    Reset,
    NewGame,
    Join,
    Name,
    Options,
    Refresh,
}

impl Icon {
    pub fn view(self) -> impl IntoView {
        let path = match self {
            Icon::Invite => "M12 3v12M7 8l5-5 5 5M5 14v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5",
            Icon::People => "M9 11.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM3 20c0-3.3 2.7-6 6-6s6 2.7 6 6M16 5a3 3 0 0 1 0 6M21 20c0-2.6-1.6-4.8-4-5.6",
            Icon::Flip => "M4 9a8 8 0 0 1 14-3l2 2M20 15a8 8 0 0 1-14 3l-2-2M18 3v5h-5M6 21v-5h5",
            Icon::Draw => "M3 11l4-4 5 5 3-3 2 2M12 12l-3 3a2 2 0 0 0 3 3l1-1M15 15l-2 2a2 2 0 0 0 3 3l1-1M18 12l3 3M8 7l-5 5",
            Icon::Resign => "M5 21V4M5 4h11l-2 4 2 4H5",
            Icon::Undo => "M9 14 4 9l5-5M4 9h10a6 6 0 0 1 0 12h-3",
            Icon::Redo => "m15 14 5-5-5-5M20 9H10a6 6 0 0 0 0 12h3",
            Icon::Reset => "M3 12a9 9 0 1 0 3-6.7M3 4v5h5",
            Icon::NewGame => "M12 5v14M5 12h14",
            Icon::Join => "M10 17l5-5-5-5M15 12H3M21 4v16",
            Icon::Name => "M4 20h4l10-10-4-4L4 16v4zM13 7l4 4",
            Icon::Options => "M4 6h10M18 6h2M4 12h2M10 12h10M4 18h12M20 18h0M14 4v4M6 10v4M16 16v4",
            Icon::Refresh => "M21 12a9 9 0 0 1-15.5 6.2L3 16M3 12a9 9 0 0 1 15.5-6.2L21 8M21 3v5h-5M3 21v-5h5",
        };

        view! {
            <svg
                aria-hidden="true"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d=path></path>
            </svg>
        }
    }
}
