use leptos::*;

#[component]
pub fn Join<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    view! {
        <form
            class="flex h-fit flex-col justify-center items-center bg-white rounded p-4"
            on:submit=submit
        >
            <label class="w-full flex justify-center text-xl">"Enter a room name"</label>
            <div class="w-full flex space-between">
                <input
                    class="border border-gray-400 rounded p-2 m-2"
                    type="text"
                    name="room"
                    placeholder="Ex: Chess fen|trash"
                />
                <button class="border border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-4 m-2" type="submit">
                    ">"
                </button>
            </div>
        </form>
    }
}
