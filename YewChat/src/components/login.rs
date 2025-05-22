use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
        <div class="bg-gradient-to-br from-purple-900 to-blue-800 flex w-screen h-screen items-center justify-center">
            <div class="bg-gray-800 bg-opacity-50 backdrop-blur-lg rounded-xl p-8 shadow-2xl max-w-md w-full mx-4">
                <div class="text-center mb-8">
                    <h1 class="text-4xl font-bold text-white mb-2">{"Welcome to "}<span class="text-purple-400">{"RustChat"}</span></h1>
                    <p class="text-gray-300">{"The most secure chat app built with Rust & WebAssembly"}</p>
                </div>
                
                <div class="mb-6">
                    <label class="block text-gray-300 text-sm mb-2">{"Choose your username"}</label>
                    <div class="flex">
                        <input 
                            {oninput} 
                            class="flex-1 rounded-l-lg p-4 border-t border-b border-l border-gray-600 text-white bg-gray-700 focus:outline-none focus:ring-2 focus:ring-purple-500" 
                            placeholder="Your cool name"
                        />
                        <Link<Route> to={Route::Chat}> 
                            <button 
                                {onclick} 
                                disabled={username.len()<1} 
                                class="px-6 rounded-r-lg bg-gradient-to-r from-purple-600 to-blue-600 text-white font-bold p-4 uppercase border-none hover:from-purple-700 hover:to-blue-700 transition-colors disabled:opacity-50"
                            >
                                {"Join Chat"}
                            </button>
                        </Link<Route>>
                    </div>
                </div>
                
                <div class="text-center text-gray-400 text-xs">
                    <div class="mt-4 flex justify-center space-x-4">
                        <span>{"⚡ Blazing fast"}</span>
                        <span>{"🔒 End-to-end encrypted"}</span>
                        <span>{"🦀 Made with Rust"}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}