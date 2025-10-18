// src/dioxus_component/addressbook/add_modal.rs
use dioxus::prelude::*;
use crate::core::db::AddressBook;

#[component]
pub fn AddModal(on_close: EventHandler, active_tab: &'static str, on_success: EventHandler) -> Element {
	let mut current_step = use_signal(|| if active_tab == "friends" { "friend" } else { "identity" });
	let mut address = use_signal(|| String::new());
	let mut alias = use_signal(|| String::new());
	let mut identity = use_signal(|| String::new());
	let mut error_message = use_signal(|| String::new());
	let success_message = use_signal(|| String::new());
	
	let add_friend = move |_| {
		let address_val = address.read().clone();
		let alias_val = alias.read().clone();
		
		if address_val.trim().is_empty() {
			error_message.set("IPv6地址不能为空".to_string());
			return;
		}
		
		if alias_val.trim().is_empty() {
			error_message.set("好友昵称不能为空".to_string());
			return;
		}
		
		let mut error_message = error_message.to_owned();
		let mut success_message = success_message.to_owned();
		let on_close = on_close.to_owned();
		let on_success = on_success.to_owned();
		
		spawn(async move {
			match tokio::task::spawn_blocking(move || {
				AddressBook::add_friend(&address_val, &alias_val)
			}).await {
				Ok(Ok(())) => {
					success_message.set("好友添加成功".to_string());
					error_message.set(String::new());
					on_success.call(());
					on_close.call(());
				}
				Ok(Err(e)) => {
					error_message.set(format!("添加失败: {}", e));
					success_message.set(String::new());
				}
				Err(e) => {
					error_message.set(format!("任务执行失败: {}", e));
					success_message.set(String::new());
				}
			}
		});
	};
	
	let add_identity = move |_| {
		let identity_val = identity.read().clone();
		let alias_val = alias.read().clone();
		
		if identity_val.trim().is_empty() {
			error_message.set("身份标识不能为空".to_string());
			return;
		}
		
		if identity_val.trim().len() != 64 {
			error_message.set("身份标识必须为64个字符".to_string());
			return;
		}
		
		if alias_val.trim().is_empty() {
			error_message.set("别名不能为空".to_string());
			return;
		}
		
		let mut error_message = error_message.to_owned();
		let mut success_message = success_message.to_owned();
		let on_close = on_close.to_owned();
		let on_success = on_success.to_owned();
		
		spawn(async move {
			match tokio::task::spawn_blocking(move || {
				AddressBook::add_identity(&identity_val, &alias_val)
			}).await {
				Ok(Ok(())) => {
					success_message.set("身份标识添加成功".to_string());
					error_message.set(String::new());
					on_success.call(());
					on_close.call(());
				}
				Ok(Err(e)) => {
					error_message.set(format!("添加失败: {}", e));
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
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0,0,0,0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1000;
            ",
            
            div {
                style: "
                    background: white;
                    padding: 0;
                    border-radius: 12px;
                    max-width: 500px;
                    width: 90%;
                    max-height: 90vh;
                    overflow: hidden;
                ",
                
                // 模态窗口头部
                div {
                    style: "
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        padding: 20px 24px;
                        border-bottom: 1px solid #e5e7eb;
                    ",
                    
                    h2 {
                        style: "margin: 0; font-size: 18px; font-weight: 600;",
                        "添加新条目"
                    }
                    
                    button {
						class: "modal-close-button",
                        style: "
                            background: none;
                            border: none;
                            font-size: 24px;
                            cursor: pointer;
                            color: #6b7280;
                        ",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                // 内容区域
                div {
                    style: "padding: 24px;",
                    
                    // 步骤选择
                    div {
                        style: "
                            display: flex;
                            gap: 8px;
                            margin-bottom: 24px;
                        ",
                        
                        button {
							class: "tab-button",
                            style: if *current_step.read() == "friend" {
                                "
                                    flex: 1;
                                    background: #3b82f6;
                                    color: white;
                                    border: none;
                                    padding: 12px;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-weight: 500;
                                "
                            } else {
                                "
                                    flex: 1;
                                    background: #f3f4f6;
                                    color: #374151;
                                    border: none;
                                    padding: 12px;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-weight: 500;
                                "
                            },
                            onclick: move |_| current_step.set("friend"),
                            "添加好友"
                        }
                        
                        button {
							class: "tab-button",
                            style: if *current_step.read() == "identity" {
                                "
                                    flex: 1;
                                    background: #3b82f6;
                                    color: white;
                                    border: none;
                                    padding: 12px;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-weight: 500;
                                "
                            } else {
                                "
                                    flex: 1;
                                    background: #f3f4f6;
                                    color: #374151;
                                    border: none;
                                    padding: 12px;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-weight: 500;
                                "
                            },
                            onclick: move |_| current_step.set("identity"),
                            "添加身份"
                        }
                    }
                    
                    // 错误和成功消息
                    if !error_message.read().is_empty() {
                        div {
                            style: "
                                background: #fee2e2;
                                border: 1px solid #fecaca;
                                color: #dc2626;
                                padding: 12px 16px;
                                border-radius: 8px;
                                margin-bottom: 20px;
                            ",
                            {error_message.read().as_str()}
                        }
                    }

                    if !success_message.read().is_empty() {
                        div {
                            style: "
                                background: #dcfce7;
                                border: 1px solid #bbf7d0;
                                color: #16a34a;
                                padding: 12px 16px;
                                border-radius: 8px;
                                margin-bottom: 20px;
                            ",
                            {success_message.read().as_str()}
                        }
                    }
                    
                    // 表单内容
                    match *current_step.read() {
                        "friend" => rsx! {
                            div {
                                style: "space-y-4",
                                
                                div {
                                    label {
                                        style: "
                                            display: block;
                                            margin-bottom: 6px;
                                            font-weight: 500;
                                            color: #374151;
                                        ",
                                        "IPv6 地址"
                                    }
                                    input {
                                        style: "
                                            width: 90%;
                                            padding: 10px 12px;
                                            border: 1px solid #d1d5db;
                                            border-radius: 8px;
                                            font-size: 14px;
                                        ",
                                        r#type: "text",
                                        placeholder: "例如: ::1 或 fe80::...",
                                        value: "{address}",
                                        oninput: move |e| address.set(e.value().clone())
                                    }
                                }
                                
                                div {
                                    label {
                                        style: "
                                            display: block;
                                            margin-bottom: 6px;
                                            font-weight: 500;
                                            color: #374151;
                                        ",
                                        "好友昵称"
                                    }
                                    input {
                                        style: "
                                            width: 90%;
                                            padding: 10px 12px;
                                            border: 1px solid #d1d5db;
                                            border-radius: 8px;
                                            font-size: 14px;
                                        ",
                                        r#type: "text",
                                        placeholder: "例如: 张三的电脑",
                                        value: "{alias}",
                                        oninput: move |e| alias.set(e.value().clone())
                                    }
                                }
                                
                                button {
									class: "modal-button",
                                    style: "
                                        width: 100%;
                                        background: #3b82f6;
                                        color: white;
                                        border: none;
                                        border-radius: 8px;
                                        padding: 12px;
                                        font-weight: 500;
                                        cursor: pointer;
                                        margin-top: 16px;
                                    ",
                                    onclick: add_friend,
                                    "添加好友"
                                }
                            }
                        },
                        "identity" => rsx! {
                            div {
                                style: "space-y-4",
                                
                                div {
                                    label {
                                        style: "
                                            display: block;
                                            margin-bottom: 6px;
                                            font-weight: 500;
                                            color: #374151;
                                        ",
                                        "身份标识 (64字符)"
                                    }
                                    input {
                                        style: "
                                            width: 90%;
                                            padding: 10px 12px;
                                            border: 1px solid #d1d5db;
                                            border-radius: 8px;
                                            font-size: 14px;
                                            font-family: monospace;
                                        ",
                                        r#type: "text",
                                        placeholder: "输入64个字符的身份标识...",
                                        value: "{identity}",
                                        oninput: move |e| identity.set(e.value().clone())
                                    }
                                }
                                
                                div {
                                    label {
                                        style: "
                                            display: block;
                                            margin-bottom: 6px;
                                            font-weight: 500;
                                            color: #374151;
                                        ",
                                        "别名"
                                    }
                                    input {
                                        style: "
                                            width: 90%;
                                            padding: 10px 12px;
                                            border: 1px solid #d1d5db;
                                            border-radius: 8px;
                                            font-size: 14px;
                                        ",
                                        r#type: "text",
                                        placeholder: "例如: 我的身份",
                                        value: "{alias}",
                                        oninput: move |e| alias.set(e.value().clone())
                                    }
                                }
                                
                                button {
                                    style: "
                                        width: 100%;
                                        background: #3b82f6;
                                        color: white;
                                        border: none;
                                        border-radius: 8px;
                                        padding: 12px;
                                        font-weight: 500;
                                        cursor: pointer;
                                        margin-top: 16px;
                                    ",
                                    onclick: add_identity,
                                    "添加身份"
                                }
                            }
                        },
                        _ => rsx! { div { "未知类型" } }
                    }
                }
            }
        }
    }
}