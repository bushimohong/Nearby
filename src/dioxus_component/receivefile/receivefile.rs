use dioxus::prelude::*;
use crate::core::filereceiver::{FileReceiver, ReceiveStatus};

#[component]
pub fn Receive() -> Element {
    let mut status = use_signal(|| ReceiveStatus::Closed);
    
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
            ",
            
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
                            // 只有当状态不同时才更新
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
                            // 只有当状态不同时才更新
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
                            // 只有当状态不同时才更新
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
                
                // 显示当前状态的文本
                div {
                    style: "
                        margin-top: 20px;
                        padding: 10px 20px;
                        background-color: white;
                        border-radius: 8px;
                        border: 1px solid #e0e0e0;
                    ",
                    "当前状态: {status:?}"
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
            ReceiveStatus::Open => "#e8f5e8",      // 绿色系
            ReceiveStatus::Collect => "#fff3e0",   // 橙色系
            ReceiveStatus::Closed => "#ffebee",    // 红色系
        }
    } else {
        "transparent"
    };
    
    let text_color = if is_active {
        match target_status {
            ReceiveStatus::Open => "#2e7d32",      // 深绿色
            ReceiveStatus::Collect => "#ef6c00",   // 深橙色
            ReceiveStatus::Closed => "#c62828",    // 深红色
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