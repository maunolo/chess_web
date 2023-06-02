use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{
    components::chess_board::ChessBoard,
    entities::chess_board::ChessBoard as ChessBoardEntity,
    handlers::mouse::{mousemove, mouseup},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        // <Stylesheet id="leptos" href="/pkg/tailwind.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="" view=  move |cx| view! { cx, <Home/> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let chessboard_entity = ChessBoardEntity::new(fen);
    let (chessboard, _set_chessboard) = create_signal(cx, chessboard_entity);

    view! { cx,

      <div
        class="flex justify-center items-center h-screen w-screen overflow-hidden"
        // ontouchmove={Callback::from(mousemove)}
        // ontouchup={Callback::from(mouseup)}
        on:mousemove=move |e| mousemove(e)
        on:mouseup=move |e| mouseup(e)
      >
        <ChessBoard chess_board=chessboard />
      </div>
    }
}

// <main class="my-0 mx-auto max-w-3xl text-center">
//     <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
//     <p class="px-10 pb-10 text-left">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
//     <button
//         class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
//         on:click=move |_| set_count.update(|count| *count += 1)
//     >
//         "Something's here | "
//         {move || if count() == 0 {
//             "Click me!".to_string()
//         } else {
//             count().to_string()
//         }}
//         " | Some more text"
//     </button>
// </main>
