use leptos::*;

use crate::entities::chess_board::ChessBoard;

use cfg_if::cfg_if;

#[component]
#[allow(unused_variables)]
pub fn Overlay<R>(
    cx: Scope,
    chessboard: WriteSignal<ChessBoard>,
    reset: R,
    chess_board_socket: Option<web_sys::WebSocket>,
) -> impl IntoView
where
    R: Fn(web_sys::MouseEvent) -> () + 'static,
{
    let (show_form, set_show_form) = create_signal(cx, false);
    let join = move |_| {
        set_show_form.set(true);
    };

    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let join_submit = |_: web_sys::SubmitEvent| {};
        } else {
            let (clone_ws, _) = create_signal(cx, chess_board_socket);
            let join_submit = move |e: web_sys::SubmitEvent| {
                e.prevent_default();
                let form = wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlFormElement>(&e.target().unwrap())
                    .unwrap()
                    .to_owned();
                let data = web_sys::FormData::new_with_form(&form).unwrap();
                let room = data.get("room").as_string().unwrap();
                let msg = format!("/join {}", room);
                if let Some(socket) = clone_ws().as_ref() {
                    match socket.send_with_str(&msg) {
                        Ok(_) => log::debug!("message successfully sent: {:?}", msg),
                        Err(err) => log::debug!("error sending message: {:?}", err),
                    }
                }

                set_show_form.set(false);
            };
        }
    }

    view! {
        cx,
        <>
            <div class="pointer-events-none flex flex-none gap-2 absolute top-0 z-30 py-2 px-2 w-full justify-center sm:w-auto sm:h-full sm:left-0 sm:flex-col sm:top-auto">
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=move |_| { chessboard.update(|cb| cb.flip()) }
                >
                    "Flip"
                </button>
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=reset
                >
                    "Reset"
                </button>
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=join
                >
                    "Join"
                </button>
            </div>
            <Show
                when=move || show_form.get()
                fallback=|_| view! {cx, ""}
            >
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <form
                        class="flex h-fit flex-col justify-center items-center bg-white rounded p-4"
                        on:submit=join_submit
                    >
                        <label class="w-full flex justify-center text-xl">"Enter a room name"</label>
                        <div class="w-full flex space-between">
                            <input
                                class="border border-gray-400 rounded p-2 m-2"
                                type="text"
                                name="room"
                                placeholder="Room name"
                            />
                            <button
                                class="border border-gray-400 rounded p-2 m-2"
                                type="submit"
                            >
                                ">"
                            </button>
                        </div>
                    </form>
                </div>
            </Show>
        </>
    }
}
