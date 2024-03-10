use leptos::*;

use crate::entities::chess_board::signals::ChessBoardSignals;

#[component]
pub fn Options<F>(chess_board_signals: ChessBoardSignals, submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let validation_switch = move || {
        let validation = chess_board_signals
            .room_status()
            .get()
            .map(|rs| rs.options().validation())
            .unwrap_or(false);

        if validation {
            view! {
                <input type="checkbox" name="validation" checked/>
            }
        } else {
            view! {
                <input type="checkbox" name="validation"/>
            }
        }
    };

    let sync_switch = move || {
        let sync = chess_board_signals
            .room_status()
            .get()
            .map(|rs| rs.options().sync())
            .unwrap_or(false);
        if sync {
            view! {
                <input type="checkbox" name="sync" checked/>
            }
        } else {
            view! {
                <input type="checkbox" name="sync"/>
            }
        }
    };
    view! {
        <form
            class="flex h-fit flex-col justify-center items-center bg-white rounded p-4"
            on:submit=submit
        >
            <label class="w-full flex justify-center text-xl mb-4">"Options"</label>
            <div class="w-full flex space-between gap-2 items-center justify-center mb-2">
                <label class="switch">
                    {validation_switch}
                    <span class="slider round"></span>
                </label>
                <label>"Validation"</label>
            </div>
            <div class="w-full flex space-between gap-2 items-center">
                <label class="switch">
                    {sync_switch}
                    <span class="slider round"></span>
                </label>
                <label>"Sync"</label>
            </div>
            <button class="border border-gray-400 rounded py-2 px-4 m-2 mt-6" type="submit">
                "Apply"
            </button>
        </form>
    }
}
