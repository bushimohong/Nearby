// src/dioxus_component/addressbook/addressbook.rs
use dioxus::prelude::*;
use super::add_member::AddModal;
use super::friends::FriendsList;
use super::whitelist::Whitelist;

#[component]
pub fn AddressBookPage() -> Element {
    let mut active_tab = use_signal(|| "friends");
    let mut show_add_modal = use_signal(|| false);
    let mut refresh_trigger = use_signal(|| 0); // 添加刷新触发器
    
    // 刷新列表的函数
    let mut refresh_list = move || {
        refresh_trigger += 1;
    };
    
    rsx! {
        div {
            style: "
                padding: 20px;
                height: 100%;
                display: flex;
                flex-direction: row;
                align-items: center;
                justify-content: flex-start;
                overflow: hidden;
            ",
            
            // 左侧边栏
            div {
                style: "
                    width: 10%;
                    min-width: 200px;
                    border-right: 1px solid #e5e7eb;
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                ",

                // 选项卡选择区域
                div {
                    style: "
                        padding: 20px;
                        border-bottom: 1px solid #e5e7eb;
                        flex-shrink: 0;
                    ",
                    
                    TabButton {
                        active: *active_tab.read() == "friends",
                        onclick: move |_| active_tab.set("friends"),
                        label: "好友列表",
                        icon: rsx! {
                            img {
                                style: "width: 20px; height: 20px;",
                                src: asset!("assets/friends-100.png"),
                            }
                        }
                    }
                    
                    TabButton {
                        active: *active_tab.read() == "whitelist",
                        onclick: move |_| active_tab.set("whitelist"),
                        label: "白名单",
                        icon: rsx! {
                            img {
                                style: "width: 20px; height: 20px;",
                                src: asset!("assets/list-100.png"),
                            }
                        }
                    }
                }

                // 添加按钮区域
                div {
                    style: "
                        padding: 20px;
                        margin-top: auto;
                        flex-shrink: 0;
                    ",
                    
                    button {
                        class: "tab-button",
                        style: "
                            width: 100%;
                            background: #3b82f6;
                            color: white;
                            border: none;
                            border-radius: 8px;
                            padding: 12px 16px;
                            font-weight: 500;
                            cursor: pointer;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            gap: 8px;
                            transition: background-color 0.2s;
                        ",
                        onmouseenter: move |_| {
                            // 悬停效果可以在 CSS 中处理
                        },
                        onclick: move |_| show_add_modal.set(true),
                        
                        img {
                            style: "width: 18px; height: 18px;",
                            src: asset!("assets/add-user-100.png"),
                        }
                        "添加新条目"
                    }
                }
            },
            
            // 右侧区域
            div {
                style: "
                    flex: 1;
                    height: 100%;
                    display: flex;
                    flex-direction: column;
                    padding-left: 10px;
                ",
                
                // 内容容器
                div {
                    style: "
                        flex: 1;
                        overflow-y: auto;
                        border: 1px solid #e0e0e0;
                        border-radius: 8px;
                        background: white;
                        min-height: 400px;
                    ",
                    match *active_tab.read() {
                        "friends" => rsx! {
                            FriendsList {
                                refresh_trigger: *refresh_trigger.read()
                            }
                        },
                        "whitelist" => rsx! {
                            Whitelist {
                                refresh_trigger: *refresh_trigger.read()
                            }
                        },
                        _ => rsx! { div { "未知标签" } }
                    }
                }
            }
            
            // 添加模态窗口
            if *show_add_modal.read() {
                AddModal {
                    on_close: move |_| show_add_modal.set(false),
                    active_tab: *active_tab.read(),
                    on_success: move |_| refresh_list(),
                }
            }
        }
    }
}

#[component]
fn TabButton(active: bool, onclick: EventHandler, label: &'static str, icon: Element) -> Element {
    let background = if active { "#3b82f6" } else { "transparent" };
    let text_color = if active { "white" } else { "#374151" };
    
    rsx! {
        button {
            class: "tab-button",
            style: "
                width: 100%;
                background: {background};
                color: {text_color};
                border: none;
                padding: 12px 16px;
                border-radius: 8px;
                font-size: 14px;
                font-weight: 500;
                cursor: pointer;
                transition: all 0.2s;
                display: flex;
                align-items: center;
                gap: 12px;
                margin-bottom: 8px;
                text-align: left;
            ",
            onmouseenter: move |_| {
                // 悬停效果
            },
            onclick: move |_| onclick.call(()),
            {icon}
            span { "{label}" }
        }
    }
}