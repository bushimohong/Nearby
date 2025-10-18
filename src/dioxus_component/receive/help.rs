// src/dioxus_component/receive/help.rs
use log::{info, error};
use dioxus::prelude::*;
use crate::core::filereceiver::FileReceiver;
use std::net::Ipv6Addr;
use arboard::Clipboard;

#[component]
pub fn HelpButton(show_help_window: Signal<bool>) -> Element {
	rsx! {
        div {
            style: "position: relative;",
            
            button {
                class: "icon-button",
                onclick: move |_| {
                    show_help_window.toggle();
                },
                img {
                    src: asset!("assets/help-100.png"),
                    class: "button-icon",
                }
            }
            
            // 帮助小窗口
            if *show_help_window.read() {
                HelpWindow { show_help_window }
            }
        }
    }
}

#[component]
fn HelpWindow(show_help_window: Signal<bool>) -> Element {
	rsx! {
        div {
            style: "
                position: absolute;
                top: 100%;
                right: 0;
                margin-top: 8px;
                background: white;
                border: 1px solid #e0e0e0;
                border-radius: 8px;
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
                padding: 16px;
                min-width: 280px;
                z-index: 1000;
            ",
            
            // 窗口标题和关闭按钮
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 12px;
                ",
                
                h3 {
                    style: "
                        margin: 0;
                        font-size: 16px;
                        font-weight: 600;
                        color: #333;
                    ",
                    "网络地址信息"
                }
                
                button {
                    style: "
                        background: none;
                        border: none;
                        cursor: pointer;
                        padding: 4px;
                        border-radius: 4px;
                        color: #666;
                        font-size: 18px;
                        line-height: 1;
                    ",
                    onclick: move |_| {
                        show_help_window.set(false);
                    },
                    "×"
                }
            }
            
            // IPv6 地址列表
            div {
                style: "
                    max-height: 200px;
                    overflow-y: auto;
                ",
                
                {
                    let ipv6_addrs: Vec<Ipv6Addr> = FileReceiver::get_ipv6_addr();
                    
                    if ipv6_addrs.is_empty() {
                        rsx! {
                            p {
                                style: "
                                    margin: 0;
                                    color: #666;
                                    font-style: italic;
                                ",
                                "未找到可用的 IPv6 地址"
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 6px;",

                                {ipv6_addrs.iter().filter_map(|addr| Some(addr)).enumerate().map(|(idx, addr)| {
                                    let addr_str = addr.to_string();
                                    rsx! {
                                        div {
                                            key: "{idx}",
                                            style: "
                                                display: flex;
                                                align-items: center;
                                                justify-content: space-between;
                                                padding: 8px 12px;
                                                background: #f8f9fa;
                                                border-radius: 4px;
                                                border: 1px solid #e9ecef;
                                            ",
                                            // 地址文本
                                            span {
                                                style: "
                                                    font-family: monospace;
                                                    font-size: 12px;
                                                    color: #495057;
                                                    word-break: break-all;
                                                    flex: 1;
                                                    margin-right: 8px;
                                                ",
                                                "{addr_str}"
                                            }
                                            button {
                                                style: "
                                                    background: none;
                                                    border: none;
                                                    cursor: pointer;
                                                    padding: 4px;
                                                    border-radius: 4px;
                                                    display: flex;
                                                    align-items: center;
                                                    justify-content: center;
                                                    transition: background-color 0.2s;
                                                ",
                                                onclick: move |_| {
                                                    // 复制功能实现
                                                    let mut clipboard = Clipboard::new().unwrap();
                                                    if let Err(e) = clipboard.set_text(addr_str.to_string().clone()) {
                                                        error!("复制失败: {}", e);
                                                    } else {
                                                        info!("已复制地址: {}", addr_str);
                                                    }
                                                },
                                                img {
                                                    src: asset!("assets/copy-100.png"),
                                                    class: "small_button-icon",
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}