use dioxus::prelude::*;
use crate::dioxus_component::{Send, Receive, AddressBookPage, Settings};

#[derive(Clone, PartialEq)]
pub enum Page {
    Receive,
	Send,
    AddressBook,
	Settings,
}

#[component]
pub fn App() -> Element {
	let current_page = use_signal(|| Page::Receive);
	
	rsx! {
        div {
            style: "
                height: 100vh;
                width: 100vw;
                display: flex;
                flex-direction: column;
                overflow: hidden;
                position: fixed;
                top: 0;
                left: 0;
            ",
            
            // 主内容区域 - 占90%高度
            div {
                style: "
                    flex: 1;
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                ",
                match current_page() {
                    Page::Send => rsx! { Send {} },
                    Page::Receive => rsx! { Receive {} },
                    Page::AddressBook => rsx! { AddressBookPage {} },
                    Page::Settings => rsx! { Settings {} },
                }
            }
            
            // 底部导航栏 - 占10%高度
            BottomNav { current_page: current_page }
        }
    }
}

#[component]
pub fn BottomNav(current_page: Signal<Page>) -> Element {
	rsx! {
        div {
            style: "
                height: 10%;
                min-height: 60px;
                display: flex;
                border-top: 1px solid #e0e0e0;
                background-color: #f8f9fa;
                flex-shrink: 0;
            ",
            
            NavItem {
                active: *current_page.read() == Page::Receive,
                icon_src: asset!("assets/receive-100.png"),
                label: "接收",
                on_click: move |_| current_page.set(Page::Receive)
            }
            
            NavItem {
                active: *current_page.read() == Page::Send,
                icon_src: asset!("assets/send-100.png"),
                label: "发送",
                on_click: move |_| current_page.set(Page::Send)
            }
            
            NavItem {
                active: *current_page.read() == Page::AddressBook,
                icon_src: asset!("assets/address-book-100.png"),
                label: "通讯录",
                on_click: move |_| current_page.set(Page::AddressBook)
            }
            
            NavItem {
                active: *current_page.read() == Page::Settings,
                icon_src: asset!("assets/setting-100.png"),
                label: "设置",
                on_click: move |_| current_page.set(Page::Settings)
            }
        }
    }
}

#[component]
pub fn NavItem(
	active: bool,
	icon_src: Asset,
	label: &'static str,
	on_click: EventHandler,
) -> Element {
	let text_color = if active { "#1976d2" } else { "#666" };
	
	rsx! {
        button {
            style: "
                flex: 1;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                border: none;
                background-color: transparent;
                cursor: pointer;
                padding: 8px;
            ",
            onclick: move |_| on_click.call(()),
            
            // 图标容器，添加底部指示器
            div {
                style: "
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    margin-bottom: 4px;
                ",
                
                img {
                    style: "width: 30px; height: 30px;",
                    src: "{icon_src}",
                    class: "button-icon",
                }
                
                // 选中状态指示器 - 小圆角矩形（在图标下方）
                {active.then(|| rsx! {
                    div {
                        style: "
                            width: 20px;
                            height: 3px;
                            background-color: #1976d2;
                            border-radius: 2px;
                            margin-top: 2px;
                        ",
                    }
                })}
            }
            
            span {
                style: "font-size: 12px; color: {text_color};",
                "{label}"
            }
        }
    }
}