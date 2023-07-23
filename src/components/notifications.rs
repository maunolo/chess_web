use leptos::*;

use crate::entities::{chess_board::signals::ChessBoardSignals, notification::NotifyType};

#[component]
pub fn Notifications(cx: Scope, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let success_icon = move || {
        view! { cx,
            <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64" fill="none">
                <path d="M28.88 43.77C28.6519 43.7708 28.4267 43.7188 28.222 43.6182C28.0173 43.5176 27.8387 43.3711 27.7 43.19L17.87 30.69C17.6385 30.376 17.5385 29.9841 17.5911 29.5975C17.6438 29.211 17.845 28.8601 18.152 28.6195C18.459 28.3788 18.8478 28.2672 19.2357 28.3084C19.6237 28.3495 19.9803 28.5402 20.23 28.84L28.87 39.84L43.76 20.84C44.0053 20.5257 44.3654 20.3218 44.7611 20.273C45.1568 20.2242 45.5557 20.3347 45.87 20.58C46.1843 20.8253 46.3882 21.1855 46.437 21.5812C46.4857 21.9769 46.3753 22.3757 46.13 22.69L30.06 43.19C29.9208 43.3705 29.742 43.5167 29.5374 43.6172C29.3328 43.7178 29.1079 43.77 28.88 43.77V43.77Z"/>
                <path d="M32 58.5C26.7588 58.5 21.6353 56.9458 17.2774 54.034C12.9195 51.1221 9.52293 46.9834 7.5172 42.1411C5.51148 37.2989 4.98669 31.9706 6.0092 26.8301C7.03171 21.6896 9.55559 16.9678 13.2617 13.2617C16.9678 9.55559 21.6896 7.03171 26.8301 6.0092C31.9706 4.98669 37.2989 5.51148 42.1411 7.5172C46.9834 9.52293 51.1221 12.9195 54.034 17.2774C56.9458 21.6353 58.5 26.7588 58.5 32C58.4921 39.0258 55.6976 45.7616 50.7296 50.7296C45.7616 55.6976 39.0258 58.4921 32 58.5V58.5ZM32 8.50001C27.3522 8.50001 22.8087 9.87826 18.9441 12.4605C15.0796 15.0427 12.0675 18.7129 10.2888 23.007C8.51018 27.301 8.0448 32.0261 8.95156 36.5846C9.85831 41.1432 12.0965 45.3305 15.383 48.617C18.6695 51.9036 22.8568 54.1417 27.4154 55.0485C31.9739 55.9552 36.699 55.4898 40.9931 53.7112C45.2871 51.9325 48.9573 48.9205 51.5395 45.0559C54.1218 41.1914 55.5 36.6479 55.5 32C55.4947 25.769 53.0171 19.7948 48.6112 15.3889C44.2052 10.9829 38.231 8.50531 32 8.50001V8.50001Z"/>
            </svg>
        }
    };

    let warning_icon = move || {
        view! { cx,
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
                <g id="Layer_43" data-name="Layer 43">
                    <path d="M59.89,51.31,35.57,9.19a4.12,4.12,0,0,0-7.14,0L4.11,51.31A4.12,4.12,0,0,0,7.68,57.5H56.32a4.12,4.12,0,0,0,3.57-6.19Zm-3.46,2.12a.11.11,0,0,1-.11.06H7.68a.12.12,0,0,1-.11-.19L31.89,11.19a.12.12,0,0,1,.21,0L56.43,53.31A.11.11,0,0,1,56.43,53.44Z"/>
                    <path d="M32,20.38a2,2,0,0,0-2,2V40.5a2,2,0,0,0,4,0V22.38A2,2,0,0,0,32,20.38Z"/>
                    <path d="M33.85,46.61a1.68,1.68,0,0,0-.19-.35,1.75,1.75,0,0,0-.24-.3,2.4,2.4,0,0,0-.31-.25,2.18,2.18,0,0,0-.34-.18,1.67,1.67,0,0,0-.38-.11,2,2,0,0,0-1.81.54,1.75,1.75,0,0,0-.24.3,2.36,2.36,0,0,0-.19.35A2.53,2.53,0,0,0,30,47a1.73,1.73,0,0,0,0,.4,2,2,0,0,0,.58,1.41,2,2,0,0,0,2.84,0A2,2,0,0,0,34,47.38a1.76,1.76,0,0,0,0-.4A1.72,1.72,0,0,0,33.85,46.61Z"/>
                </g>
            </svg>
        }
    };

    let error_icon = move || {
        view! { cx,
            <svg xmlns="http://www.w3.org/2000/svg" data-name="Layer 1" viewBox="0 0 64 64">
                <circle cx="32" cy="32" r="28" fill="none" stroke-miterlimit="10" stroke-width="4"/>
                <line x1="32" x2="32" y1="18" y2="38" fill="none" stroke-miterlimit="10" stroke-width="4"/>
                <line x1="32" x2="32" y1="42" y2="46" fill="none" stroke-miterlimit="10" stroke-width="4"/>
            </svg>
        }
    };

    let icon = move || match chess_board_signals
        .notification()
        .with(|n| n.notify_type.clone())
    {
        NotifyType::Success => success_icon(),
        NotifyType::Warning => warning_icon(),
        NotifyType::Error => error_icon(),
    };

    let notify_class = move || match chess_board_signals.notification().with(|n| n.is_active) {
        true => "notify notify--is-active",
        false => "notify",
    };

    let notify_icon_class = move || match chess_board_signals
        .notification()
        .with(|n| n.notify_type.clone())
    {
        NotifyType::Success => "notify-icon success",
        NotifyType::Warning => "notify-icon warning",
        NotifyType::Error => "notify-icon error",
    };

    let clone_notification = move |_: web_sys::MouseEvent| {
        chess_board_signals.notification().update(|n| n.disable());
    };

    view! { cx,
        <Show
            when=move || { chess_board_signals.notification().get().is_active }
            fallback=|cx| view! { cx, <div class="hidden"></div> }
        >
            <div class=notify_class>
                <div class=notify_icon_class>
                    {icon}
                </div>
                <p class="notify-text">
                    {move || chess_board_signals
                        .notification()
                        .get().message}
                </p>
                <button class="notify-close" on:click=clone_notification>
                    <span class="notify-close-icon"></span>
                </button>
            </div>
        </Show>
    }
}
