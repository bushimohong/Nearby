use dioxus::prelude::*;
use crate::core::filereceiver::{FileReceiver, ReceiveStatus};
use super::help::HelpButton;

#[component]
pub fn Receive() -> Element {
    let mut status = use_signal(|| ReceiveStatus::Closed);
    let show_help_window = use_signal(|| false);
    
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
                
                // 帮助按钮 - 使用抽象后的组件
                HelpButton { show_help_window }
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