use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use chrono::Local;
use gloo_timers::callback::Timeout;
use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
    AddReaction(usize, String),
    SetTyping(bool),
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
    timestamp: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
    Typing,
    Reaction,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
    online: bool,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    dark_mode: bool,
    typing: bool,
    typing_users: Vec<String>,
    reactions: Vec<(usize, String)>,
    _emoji_picker_open: bool,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            dark_mode: false,
            typing: false,
            typing_users: vec![],
            reactions: vec![],
            _emoji_picker_open: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                ).into(),
                                online: true,
                            })
                            .collect();
                        true
                    }
                    MsgTypes::Message => {
                        let mut message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        if message_data.timestamp.is_none() {
                            message_data.timestamp = Some(Local::now().format("%H:%M").to_string());
                        }
                        self.messages.push(message_data);
                        true
                    }
                    MsgTypes::Typing => {
                        let username = msg.data.unwrap();
                        if !self.typing_users.contains(&username) {
                            self.typing_users.push(username);
                        }
                        let link = ctx.link().clone();
                        Timeout::new(3000, move || {
                            link.send_message(Msg::SetTyping(false));
                        }).forget();
                        true
                    }
                    MsgTypes::Reaction => {
                        let data: (usize, String) = serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.reactions.push(data);
                        true
                    }
                    _ => false,
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message_text = input.value();
                    if message_text.trim().is_empty() {
                        return false;
                    }

                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(message_text.clone()),
                        data_array: None,
                    };

                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                true
            }
            Msg::AddReaction(index, reaction) => {
                let message = WebSocketMessage {
                    message_type: MsgTypes::Reaction,
                    data: Some(serde_json::to_string(&(index, reaction)).unwrap()),
                    data_array: None,
                };
                self.wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()).ok();
                false
            }
            Msg::SetTyping(typing) => {
                self.typing = typing;
                if typing {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Typing,
                        data: None,
                        data_array: None,
                    };
                    self.wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()).ok();
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_dark = ctx.link().callback(|_| Msg::ToggleDarkMode);
        let oninput = ctx.link().batch_callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if !input.value().is_empty() {
                Some(Msg::SetTyping(true))
            } else {
                Some(Msg::SetTyping(false))
            }
        });

        let emojis = ["😀", "😍", "👍", "❤️", "😂", "😎", "🤔", "🎉"];
        
        let theme_class = if self.dark_mode {
            "bg-gray-900 text-white"
        } else {
            "bg-gray-100 text-gray-900"
        };

        let message_theme = if self.dark_mode {
            "bg-gray-700 text-white"
        } else {
            "bg-gray-200 text-gray-900"
        };

        html! {
            <div class={classes!("flex", "w-screen", "h-screen", theme_class)}>
                <div class="flex-none w-56 h-screen bg-gray-800 text-white">
                    <div class="flex justify-between items-center p-3">
                        <div class="text-xl">{"Users"}</div>
                        <button onclick={toggle_dark} class="p-1 rounded-full bg-gray-700">
                            {if self.dark_mode { "☀️" } else { "🌙" }}
                        </button>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-gray-700 rounded-lg p-2 items-center">
                                    <div class="relative">
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                        <div class={classes!(
                                            "absolute", "bottom-0", "right-0", 
                                            "w-3", "h-3", "rounded-full", 
                                            "border-2", "border-gray-800",
                                            if u.online { "bg-green-500" } else { "bg-gray-500" }
                                        )}></div>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {if u.online { "Online" } else { "Offline" }}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-600 flex items-center px-4">
                        <div class="text-xl">{"💬 Chat!"}</div>
                        <div class="ml-2 text-sm text-gray-400">
                            {if !self.typing_users.is_empty() {
                                format!("{} is typing...", self.typing_users.join(", "))
                            } else {
                                "".to_string()
                            }}
                        </div>
                    </div>
                    <div class="w-full grow overflow-auto">
                        {
                            self.messages.iter().enumerate().map(|(idx, m)| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                let reactions = self.reactions.iter()
                                    .filter(|(i, _)| *i == idx)
                                    .map(|(_, r)| r)
                                    .collect::<Vec<_>>();
                                
                                html!{
                                    <div class="flex items-start w-full p-4 hover:bg-opacity-50 hover:bg-gray-800 transition-all duration-200">
                                        <img class="w-10 h-10 rounded-full m-2" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="flex-1">
                                            <div class="flex items-center">
                                                <div class="font-semibold">{m.from.clone()}</div>
                                                <div class="ml-2 text-xs text-gray-400">{m.timestamp.clone().unwrap_or_default()}</div>
                                            </div>
                                            <div class={classes!("p-3", "rounded-lg", "inline-block", message_theme)}>
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-1 max-w-xs rounded" src={m.message.clone()}/>
                                                } else {
                                                    <div class="text-sm whitespace-pre-wrap">{m.message.clone()}</div>
                                                }
                                            </div>
                                            if !reactions.is_empty() {
                                                <div class="flex mt-1">
                                                    {for reactions.iter().map(|r| {
                                                        html!{ <div class="mr-1 text-lg">{r}</div> }
                                                    })}
                                                </div>
                                            }
                                            <div class="flex mt-1 space-x-1">
                                                {for emojis.iter().map(|&emoji| {
                                                    let emoji_str = emoji.to_string();
                                                    let idx = idx;
                                                    let onclick = ctx.link().callback(move |_| Msg::AddReaction(idx, emoji_str.clone()));
                                                    html!{
                                                        <button onclick={onclick} class="text-sm opacity-70 hover:opacity-100 transition-opacity">
                                                            {emoji}  // Use the original &str here
                                                        </button>
                                                    }
                                                })}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-20 flex px-3 items-center bg-gray-800">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Message" 
                            {oninput}
                            class="block w-full py-3 pl-4 mx-3 bg-gray-700 text-white rounded-full outline-none focus:text-gray-200" 
                            name="message" 
                            required=true 
                            onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    Some(Msg::SubmitMessage)
                                } else {
                                    None
                                }
                            })}
                        />
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-12 h-12 rounded-full flex justify-center items-center text-white hover:bg-blue-700 transition-colors">
                            <svg viewBox="0 0 24 24" class="w-6 h-6 fill-current">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}