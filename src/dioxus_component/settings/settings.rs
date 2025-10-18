// src/dioxus_component/settings/setting.rs
use dioxus::prelude::*;
use crate::core::db::AddressBook;

#[component]
pub fn Settings() -> Element {
    let my_identity = use_signal(|| String::new());
    let error_message = use_signal(|| String::new());
    let success_message = use_signal(|| String::new());
    let copy_success = use_signal(|| false);
    let reset_success = use_signal(|| false);
    
    // 加载我的身份码
    use_effect(move || {
        let my_identity = my_identity.to_owned();
        let mut error_message = error_message.to_owned();
        
        spawn(async move {
            if let Err(e) = AddressBook::init_db() {
                error_message.set(format!("数据库初始化失败: {}", e));
                return;
            }
            load_my_identity(my_identity, error_message).await;
        });
    });
    
    // 复制身份码到剪贴板
    let copy_identity = move |_| {
        let identity = my_identity.read().clone();
        let mut copy_success = copy_success.to_owned();
        
        spawn(async move {
            // 使用 web_sys 来复制到剪贴板
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if let Ok(clipboard) = window.navigator().clipboard() {
                        if clipboard.write_text(&identity).await.is_ok() {
                            copy_success.set(true);
                            
                            // 2秒后重置复制成功状态
                            spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                copy_success.set(false);
                            });
                        }
                    }
                }
            }
            
            // 对于桌面应用，使用 arboard（需要在 Cargo.toml 中添加依赖）
            #[cfg(not(target_arch = "wasm32"))]
            {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&identity).is_ok() {
                        copy_success.set(true);
                        
                        // 3秒后重置复制成功状态
                        spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            copy_success.set(false);
                        });
                    }
                }
            }
        });
    };
    
    // 重置身份码
    let reset_identity = move |_| {
        let mut my_identity = my_identity.to_owned();
        let mut error_message = error_message.to_owned();
        let mut success_message = success_message.to_owned();
        let mut reset_success = reset_success.to_owned();
        
        spawn(async move {
            match tokio::task::spawn_blocking(|| AddressBook::reset_my_identity()).await {
                Ok(Ok(new_identity)) => {
                    success_message.set("身份码重置成功".to_string());
                    error_message.set(String::new());
                    my_identity.set(new_identity);
                    
                    reset_success.set(true);
                    
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        reset_success.set(false);
                    });
                }
                Ok(Err(e)) => {
                    error_message.set(format!("重置失败: {}", e));
                    success_message.set(String::new());
                }
                Err(e) => {
                    error_message.set(format!("任务执行失败: {}", e));
                    success_message.set(String::new());
                }
            }
        });
    };
    
    rsx! {
        div {
            style: "
                height: 100%;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            ",
            
            h1 {
                style: "
                    color: #333;
                    padding: 20px;
                    margin: 0;
                    border-bottom: 1px solid #e0e0e0;
                    background: white;
                    text-align: center;
                ",
                "设置"
            }
            
            // 设置列表 - 可滚动区域
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: 10px;
                ",
                
                IdentitySection {
                    my_identity,
                    error_message,
                    success_message,
                    copy_success,
                    reset_success,
                    on_copy: copy_identity,
                    on_reset: reset_identity,
                }
            }
        }
    }
}

// 新的身份码设置组件
#[component]
fn IdentitySection(
    my_identity: Signal<String>,
    error_message: Signal<String>,
    success_message: Signal<String>,
    copy_success: Signal<bool>,
    reset_success: Signal<bool>,
    on_copy: EventHandler,
    on_reset: EventHandler,
) -> Element {
    rsx! {
        div {
            style: "
                padding: 20px;
                margin-bottom: 15px;
                background-color: white;
                border-radius: 8px;
                border: 1px solid #e0e0e0;
            ",
            
            // 标题
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 15px;
                ",
                
                span {
                    style: "
                        color: #333;
                        font-weight: bold;
                        font-size: 16px;
                    ",
                    "我的身份码"
                }
                
                span {
                    style: "color: #999;",
                    "身份标识"
                }
            }
            
            // 身份码显示区域
            div {
                style: "
                    background-color: #f5f5f5;
                    padding: 12px;
                    border-radius: 6px;
                    margin-bottom: 15px;
                    word-break: break-all;
                    font-family: monospace;
                    border: 1px solid #e0e0e0;
                ",
                
                if my_identity.read().is_empty() {
                    span {
                        style: "color: #999;",
                        "加载中..."
                    }
                } else {
                    span {
                        style: "color: #333;",
                        "{my_identity}"
                    }
                }
            }
            
            // 按钮区域
            div {
                style: "
                    display: flex;
                    gap: 10px;
                ",
                
                // 复制按钮
                button {
                    style: "
                        flex: 1;
                        padding: 10px;
                        background-color: #007bff;
                        color: white;
                        border: none;
                        border-radius: 6px;
                        cursor: pointer;
                        font-size: 14px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        gap: 8px;
                    ",
                    onclick: move |_| on_copy.call(()),
                    
                    if copy_success() {
                        div {
                            img {
                                style: "width: 16px; height: 16px; position: relative; left: -2px; top: 2px;",
                                src: asset!("assets/success-100.png"),
                                alt: "  成功"
                            }
                            span { "  已复制" }
                        }
                    } else {
                        div {
                            img {
                                style: "width: 16px; height: 16px;",
                                src: asset!("assets/copy-100.png"),
                                alt: "  复制"
                            }
                            span { "  复制身份码" }
                        }
                    }
                }
                
                // 重置按钮
                button {
                    style: "
                        flex: 1;
                        padding: 10px;
                        background-color: #ff6b6b;
                        color: white;
                        border: none;
                        border-radius: 6px;
                        cursor: pointer;
                        font-size: 14px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        gap: 8px;
                    ",
                    onclick: move |_| on_reset.call(()),
                    
                    if reset_success() {
                        div {
                            img {
                                style: "width: 16px; height: 16px; position: relative; left: -2px; top: 2px;",
                                src: asset!("assets/success-100.png"),
                                alt: "  成功"
                            }
                            span { "  已重置" }
                        }
                    } else {
                        div {
                            img {
                                style: "width: 16px; height: 16px;",
                                src: asset!("assets/reset-100.png"),
                                alt: "  重置"
                            }
                            span { "  重置身份码" }
                        }
                    }
                }
            }
        }
    }
}

// 加载我的身份码函数
async fn load_my_identity(
    mut my_identity: Signal<String>,
    mut error_message: Signal<String>,
) {
    match tokio::task::spawn_blocking(|| AddressBook::get_my_identity()).await {
        Ok(Ok(identity)) => {
            my_identity.set(identity);
        }
        Ok(Err(e)) => {
            error_message.set(format!("加载身份码失败: {}", e));
        }
        Err(e) => {
            error_message.set(format!("任务执行失败: {}", e));
        }
    }
}