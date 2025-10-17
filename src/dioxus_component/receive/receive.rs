use dioxus::prelude::*;
use crate::core::filereceiver::{FileReceiver, ReceiveStatus};
use std::net::Ipv6Addr;
use arboard::Clipboard;

#[component]
pub fn Receive() -> Element {
    let mut status = use_signal(|| ReceiveStatus::Closed);
    let mut show_help_window = use_signal(|| false);
    
    // 初始化状态
    use_effect(move || {
        let current_status = FileReceiver::get_receive_status();
        status.set(current_status);
    });
    
    rsx! {
        div {
            style: "
                height: 100%;
                display: flex;
                flex-direction: column;
                overflow: hidden;
                position: relative;
            ",
            
            // 右上角按钮容器
            div {
                style: "
                    position: absolute;
                    top: 24px;
                    right: 24px;
                    display: flex;
                    gap: 10px;
                    z-index: 10;
                ",
                
                // 历史按钮
                button {
                    class: "icon-button",
                    onclick: move |_| {
                        println!("历史按钮被点击");
                    },
                    img {
                        src: asset!("assets/history-100.png"),
                        class: "button-icon",
                    }
                }
                
                // 帮助按钮
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
                                                                        eprintln!("复制失败: {}", e);
                                                                    } else {
                                                                        println!("已复制地址: {}", addr_str);
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
            }
            
            // 状态选择区域
            div {
                style: "
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    flex: 1;
                    padding: 20px;
                ",
                
                // 胶囊状状态栏
                div {
                    style: "
                        display: flex;
                        background-color: #f0f0f0;
                        border-radius: 25px;
                        padding: 4px;
                        margin-bottom: 20px;
                    ",
                    
                    StatusButton {
                        current_status: status,
                        target_status: ReceiveStatus::Open,
                        label: "开启",
                        on_click: move |_| {
                            if *status.read() != ReceiveStatus::Open {
                                if let Err(e) = FileReceiver::set_receive_status(ReceiveStatus::Open) {
                                    eprintln!("设置状态失败: {}", e);
                                } else {
                                    status.set(ReceiveStatus::Open);
                                    println!("状态改为: 开启");
                                }
                            }
                        }
                    }
                    
                    StatusButton {
                        current_status: status,
                        target_status: ReceiveStatus::Collect,
                        label: "收藏",
                        on_click: move |_| {
                            if *status.read() != ReceiveStatus::Collect {
                                if let Err(e) = FileReceiver::set_receive_status(ReceiveStatus::Collect) {
                                    eprintln!("设置状态失败: {}", e);
                                } else {
                                    status.set(ReceiveStatus::Collect);
                                    println!("状态改为: 收藏");
                                }
                            }
                        }
                    }
                    
                    StatusButton {
                        current_status: status,
                        target_status: ReceiveStatus::Closed,
                        label: "关闭",
                        on_click: move |_| {
                            if *status.read() != ReceiveStatus::Closed {
                                if let Err(e) = FileReceiver::set_receive_status(ReceiveStatus::Closed) {
                                    eprintln!("设置状态失败: {}", e);
                                } else {
                                    status.set(ReceiveStatus::Closed);
                                    println!("状态改为: 关闭");
                                }
                            }
                        }
                    }
                }
            }
            
            // // 点击小窗口外部关闭的遮罩层
            // if *show_help_window.read() {
            //     div {
            //         style: "
            //             position: fixed;
            //             top: 0;
            //             left: 0;
            //             right: 0;
            //             bottom: 0;
            //             z-index: 999;
            //             background: transparent;
            //         ",
            //         onclick: move |_| {
            //             show_help_window.set(false);
            //         }
            //     }
            // }
        }
    }
}

#[component]
fn StatusButton(
    current_status: Signal<ReceiveStatus>,
    target_status: ReceiveStatus,
    label: &'static str,
    on_click: EventHandler,
) -> Element {
    let is_active = *current_status.read() == target_status;
    let bg_color = if is_active {
        match target_status {
            ReceiveStatus::Open => "#e8f5e8",
            ReceiveStatus::Collect => "#fff3e0",
            ReceiveStatus::Closed => "#ffebee",
        }
    } else {
        "transparent"
    };
    
    let text_color = if is_active {
        match target_status {
            ReceiveStatus::Open => "#2e7d32",
            ReceiveStatus::Collect => "#ef6c00",
            ReceiveStatus::Closed => "#c62828",
        }
    } else {
        "#666"
    };
    
    rsx! {
        button {
            style: "
                padding: 10px 20px;
                border: none;
                background-color: {bg_color};
                color: {text_color};
                border-radius: 20px;
                cursor: pointer;
                font-size: 14px;
                font-weight: 500;
                transition: all 0.2s ease;
                min-width: 80px;
            ",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}