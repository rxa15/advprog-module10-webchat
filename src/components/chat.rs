use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
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
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
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
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                                avatar: generate_avatar_for_user(u),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
        <div class="flex min-h-screen w-screen">
            <aside class="w-64 bg-gray-100 p-4">
                <h2 class="text-xl font-bold mb-4">{"Users"}</h2>
                {for self.users.iter().map(|user| {
                    html!{
                        <div class="flex items-center bg-white rounded-lg p-2 mb-2 shadow hover:bg-gray-50">
                            <img class="w-12 h-12 rounded-full" src={user.avatar.clone()} alt={format!("{}'s avatar", user.name)} />
                            <div class="ml-4">
                                <p class="text-sm font-medium">{&user.name}</p>
                                <p class="text-xs text-gray-400">{"Active now"}</p>
                            </div>
                        </div>
                    }
                })}
            </aside>
            <main class="flex-grow flex flex-col bg-gray-50">
                <header class="bg-white shadow p-4">
                    <h1 class="text-xl font-bold">{"💬 Chat"}</h1>
                </header>
                <div class="flex-grow overflow-auto p-4">
                    {for self.messages.iter().map(|message| {
                        let user = self.users.iter().find(|u| u.name == message.from).unwrap();
                        html!{
                            <div class="flex items-end mb-4">
                                <img class="w-8 h-8 rounded-full mr-3" src={user.avatar.clone()} alt={format!("{}'s avatar", user.name)} />
                                <div class="flex flex-col bg-white rounded-lg p-3 shadow">
                                    <span class="text-sm font-medium">{&message.from}</span>
                                    <span class="text-gray-600 text-xs">
                                        {if message.message.ends_with(".gif") {
                                            html! { <img src={message.message.clone()} alt="gif image" /> }
                                        } else {
                                            html! { <p>{&message.message}</p> }
                                        }}
                                    </span>
                                </div>
                            </div>
                        }
                    })}
                </div>
                <footer class="flex items-center p-4 bg-white shadow">
                    <input ref={self.chat_input.clone()} type="text" placeholder="Type a message..." class="flex-grow rounded-full border-2 border-gray-300 p-2 mr-2 focus:border-blue-500 outline-none" />
                    <button onclick={submit} class="flex justify-center items-center w-12 h-12 text-white bg-blue-600 rounded-full hover:bg-blue-700 focus:outline-none">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
                    </button>
                </footer>
            </main>
        </div>
    }
    }
}
fn generate_avatar_for_user(user_name: &str) -> String {
    format!("https://robohash.org/{}.png?set=set4", user_name)
}