// src/dioxus_component/addressbook/whitelist.rs
use dioxus::prelude::*;
use crate::core::db::AddressBook;

#[component]
pub fn Whitelist(refresh_trigger: u32) -> Element {
	let identities = use_signal(|| Vec::new());
	let mut search_query = use_signal(|| String::new());
	let mut error_message = use_signal(|| None::<String>);
	let mut selected_identity = use_signal(|| None::<crate::core::db::IdentityEntry>);
	let mut show_edit_modal = use_signal(|| false);
	
	// 统一的错误处理函数
	let handle_error = move |message: String| {
		error_message.set(Some(message));
	};
	
	// 统一的加载函数
	let load_identities = {
		let identities_signal = identities.clone();
		let error_signal = error_message.clone();
		move |query: Option<String>| {
			let mut identities = identities_signal.clone();
			let mut error_message = error_signal.clone();
			spawn(async move {
				let result = if let Some(ref q) = query {
					AddressBook::search_identities(&q)
				} else {
					AddressBook::get_all_identities()
				};
				
				match result {
					Ok(identities_list) => {
						identities.set(identities_list);
					}
					Err(e) => {
						let error_msg = if query.is_some() {
							format!("搜索失败: {}", e)
						} else {
							format!("加载白名单失败: {}", e)
						};
						error_message.set(Some(error_msg));
					}
				}
			});
		}
	};
	
	// 加载身份标识列表
	use_effect(use_reactive((&refresh_trigger,), move |_| {
		load_identities(None);
	}));
	
	// 处理搜索
	let handle_search = move |_| {
		let query = search_query.read().clone();
		if query.is_empty() {
			load_identities(None);
		} else {
			load_identities(Some(query));
		}
	};
	
	// 处理编辑身份标识
	let mut handle_edit_identity = move |identity: crate::core::db::IdentityEntry| {
		selected_identity.set(Some(identity));
		show_edit_modal.set(true);
	};
	
	// 处理保存编辑
	let handle_save_edit = move |id: i64, identity: String, alias: String| {
		let load_identities = load_identities.clone();
		let mut handle_error = handle_error.clone();
		spawn(async move {
			match AddressBook::update_identity(id, &identity, &alias) {
				Ok(()) => {
					// 使用统一的加载函数重新加载列表
					load_identities(None);
				}
				Err(e) => {
					handle_error(format!("更新身份标识失败: {}", e));
				}
			}
		});
	};
	
	// 处理删除身份标识
	let handle_delete_identity = move |id: i64| {
		let load_identities = load_identities.clone();
		let mut handle_error = handle_error.clone();
		spawn(async move {
			match AddressBook::delete_identity(id) {
				Ok(()) => {
					// 使用统一的加载函数重新加载列表
					load_identities(None);
				}
				Err(e) => {
					handle_error(format!("删除身份标识失败: {}", e));
				}
			}
		});
	};
	
	// 克隆身份标识列表以避免生命周期问题
	let identities_list = identities.read().clone();
	
	rsx! {
        div {
            style: "
                padding: 20px;
                height: 90%;
                display: flex;
                flex-direction: column;
            ",
            
            // 标题和搜索栏
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 20px;
                    padding-bottom: 15px;
                    border-bottom: 1px solid #e5e7eb;
                ",
                
                h2 {
                    style: "margin: 0; color: #1f2937;",
                    "白名单"
                }
                
                div {
                    style: "display: flex; gap: 10px; align-items: center;",
                    
                    input {
                        style: "
                            padding: 8px 12px;
                            border: 1px solid #d1d5db;
                            border-radius: 6px;
                            font-size: 14px;
                            width: 250px;
                        ",
                        placeholder: "搜索...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                    
                    button {
						class: "search-button",
                        style: "
                            padding: 8px 16px;
                            background: #3b82f6;
                            color: white;
                            border: none;
                            border-radius: 6px;
                            cursor: pointer;
                        ",
                        onclick: handle_search,
                        "搜索"
                    }
                }
            }
            
            // 错误信息显示
            if let Some(error) = error_message.read().as_ref() {
                div {
                    style: "
                        background: #fee2e2;
                        border: 1px solid #fecaca;
                        color: #dc2626;
                        padding: 12px;
                        border-radius: 6px;
                        margin-bottom: 15px;
                    ",
                    "{error}"
                }
            }
            
            // 身份标识列表
            if identities_list.is_empty() {
                div {
                    style: "
                        text-align: center;
                        color: #6b7280;
                        padding: 40px;
                    ",
                    "白名单为空，点击左上角\"添加新条目\"按钮添加身份标识"
                }
            } else {
                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        gap: 8px;
                    ",
                    
                    for identity in identities_list {
                        IdentityItem {
                            identity: identity.clone(),
                            on_click: move || handle_edit_identity(identity.clone())
                        }
                    }
                }
            }
            
            // 编辑模态框
            if *show_edit_modal.read() {
                if let Some(identity) = selected_identity.read().as_ref() {
                    IdentityEditModal {
                        identity: identity.clone(),
                        on_save: move |params: (i64, String, String)| {
                            handle_save_edit(params.0, params.1, params.2);
                            show_edit_modal.set(false);
                        },
                        on_delete: move |id| {
                            handle_delete_identity(id);
                            show_edit_modal.set(false);
                        },
                        on_close: move |_| show_edit_modal.set(false),
                    }
                }
            }
        }
    }
}

#[component]
fn IdentityItem(identity: crate::core::db::IdentityEntry, on_click: EventHandler) -> Element {
	rsx! {
        div {
			class: "identity-item",
            style: "
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 16px;
                border: 1px solid #e5e7eb;
                border-radius: 8px;
                background: white;
                transition: all 0.2s;
                cursor: pointer;
            ",
            onmouseenter: move |_| {
                // 悬停效果
            },
            onclick: move |_| on_click.call(()),
            
            // 身份标识信息
            div {
                style: "flex: 1;",
                
                div {
                    style: "
                        font-weight: 500;
                        color: #1f2937;
                        margin-bottom: 4px;
                    ",
                    "{identity.alias}"
                }
                
                div {
                    style: "
                        font-family: monospace;
                        color: #6b7280;
                        font-size: 12px;
                        word-break: break-all;
                    ",
                    "{identity.identity}"
                }
            }
            
            // 点击提示
            div {
                img {
                    style: "width: 24px; height: 24px;",
                    src: asset!("assets/more-100.png"),
                }
            }
        }
    }
}

#[component]
fn IdentityEditModal(
	identity: crate::core::db::IdentityEntry,
	on_save: EventHandler<(i64, String, String)>,
	on_delete: EventHandler<i64>,
	on_close: EventHandler,
) -> Element {
	let mut identity_str = use_signal(|| identity.identity.clone());
	let mut alias = use_signal(|| identity.alias.clone());
	let mut show_confirm_delete = use_signal(|| false);
	
	rsx! {
        div {
            style: "
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1000;
            ",
            
            div {
                style: "
                    background: white;
                    padding: 24px;
                    border-radius: 12px;
                    width: 90%;
                    max-width: 500px;
                    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
                ",
                onclick: move |e| e.stop_propagation(),
                
                h3 {
                    style: "margin: 0 0 20px 0; color: #1f2937;",
                    "编辑身份标识"
                }
                
                // 表单
                div {
                    style: "display: flex; flex-direction: column; gap: 16px;",
                    
                    // 身份标识输入
                    div {
                        label {
                            style: "
                                display: block;
                                margin-bottom: 6px;
                                font-weight: 500;
                                color: #374151;
                            ",
                            "身份标识"
                        }
                        input {
                            style: "
                                width: 90%;
                                padding: 10px 12px;
                                border: 1px solid #d1d5db;
                                border-radius: 6px;
                                font-size: 14px;
                                font-family: monospace;
                            ",
                            value: "{identity_str}",
                            oninput: move |e| identity_str.set(e.value())
                        }
                    }
                    
                    // 别名输入
                    div {
                        label {
                            style: "
                                display: block;
                                margin-bottom: 6px;
                                font-weight: 500;
                                color: #374151;
                            ",
                            "备注"
                        }
                        input {
                            style: "
                                width: 90%;
                                padding: 10px 12px;
                                border: 1px solid #d1d5db;
                                border-radius: 6px;
                                font-size: 14px;
                            ",
                            value: "{alias}",
                            oninput: move |e| alias.set(e.value())
                        }
                    }
                }
                
                // 按钮区域
                div {
                    style: "
                        display: flex;
                        justify-content: space-between;
                        margin-top: 24px;
                        gap: 12px;
                    ",
                    
                    // 删除按钮
                    if !show_confirm_delete() {
                        button {
							class: "modal-danger-button",
                            style: "
                                padding: 10px 16px;
                                background: #ef4444;
                                color: white;
                                border: none;
                                border-radius: 6px;
                                cursor: pointer;
                                font-size: 14px;
                            ",
                            onclick: move |_| show_confirm_delete.set(true),
                            "删除"
                        }
                    } else {
                        div {
                            style: "display: flex; gap: 8px; align-items: center;",
                            
                            span {
                                style: "color: #6b7280; font-size: 14px;",
                                "确认删除?"
                            }
                            
                            button {
								class: "confirm-delete-button",
                                style: "
                                    padding: 6px 12px;
                                    background: #ef4444;
                                    color: white;
                                    border: none;
                                    border-radius: 4px;
                                    cursor: pointer;
                                    font-size: 12px;
                                ",
                                onclick: move |_| {
                                    on_delete.call(identity.id);
                                },
                                "确认删除"
                            }
                            
                            button {
								class: "cancel-delete-button",
                                style: "
                                    padding: 6px 12px;
                                    background: #6b7280;
                                    color: white;
                                    border: none;
                                    border-radius: 4px;
                                    cursor: pointer;
                                    font-size: 12px;
                                ",
                                onclick: move |_| show_confirm_delete.set(false),
                                "取消"
                            }
                        }
                    }
                    
                    // 保存和取消按钮
                    div {
                        style: "display: flex; gap: 12px;",
                        
                        button {
							class: "modal-secondary-button",
                            style: "
                                padding: 10px 20px;
                                background: #6b7280;
                                color: white;
                                border: none;
                                border-radius: 6px;
                                cursor: pointer;
                                font-size: 14px;
                            ",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                        
                        button {
							class: "modal-button",
                            style: "
                                padding: 10px 20px;
                                background: #3b82f6;
                                color: white;
                                border: none;
                                border-radius: 6px;
                                cursor: pointer;
                                font-size: 14px;
                            ",
                            onclick: move |_| {
                                on_save.call((identity.id, identity_str.read().clone(), alias.read().clone()));
                            },
                            "保存"
                        }
                    }
                }
            }
        }
    }
}