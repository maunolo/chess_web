use leptos::*;

#[component]
pub fn Username<F>(cx: Scope, submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    view! { cx,
        <form
            class="flex h-fit flex-col justify-center items-center bg-white rounded p-4"
            on:submit=submit
        >
            <label class="w-full flex justify-center text-xl">"Enter your username"</label>
            <div class="w-full flex space-between">
                <input
                    class="border border-gray-400 rounded p-2 m-2"
                    type="text"
                    name="username"
                    placeholder="Ex: John Doe"
                />
                <button class="border border-gray-400 rounded py-2 px-4 m-2" type="submit">
                    ">"
                </button>
            </div>
        </form>
    }
}
