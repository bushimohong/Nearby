// src/dioxus_component/addressbook/friends.rs
use dioxus::prelude::*;
use crate::core::db::AddressBook;

#[component]
pub fn FriendsList(refresh_trigger: u32) -> Element {
	let friends = use_signal(|| Vec::new());
	let mut search_query = use_signal(|| String::new());
	let mut error_message = use_signal(|| None::<String>);
	let mut selected_friend = use_signal(|| None::<crate::core::db::FriendEntry>);
	let mut show_edit_modal = use_signal(|| false);
	
	// 统一的错误处理函数
	let handle_error = move |message: String| {
		error_message.set(Some(message));
	};
	
	// 统一的加载函数
	let load_friends = {
		let friends_signal = friends.clone();
		let error_signal = error_message.clone();
		move |query: Option<String>| {
			let mut friends = friends_signal.clone();
			let mut error_message = error_signal.clone();
			spawn(async move {
				let result = if let Some(ref q) = query {
					AddressBook::search_friends(&q)
				} else {
					AddressBook::get_all_friends()
				};
				
				match result {
					Ok(friends_list) => {
						friends.set(friends_list);
					}
					Err(e) => {
						let error_msg = if query.is_some() {
							format!("搜索失败: {}", e)
						} else {
							format!("加载好友列表失败: {}", e)
						};
						error_message.set(Some(error_msg));
					}
				}
			});
		}
	};
	
	// 加载好友列表
	use_effect(use_reactive((&refresh_trigger,), move |_| {
		load_friends(None);
	}));
	
	// 处理搜索
	let handle_search = move |_| {
		let query = search_query.read().clone();
		if query.is_empty() {
			load_friends(None);
		} else {
			load_friends(Some(query));
		}
	};
	
	// 处理编辑好友
	let mut handle_edit_friend = move |friend: crate::core::db::FriendEntry| {
		selected_friend.set(Some(friend));
		show_edit_modal.set(true);
	};
	
	// 处理保存编辑
	let handle_save_edit = move |id: i64, address: String, alias: String| {
		let load_friends = load_friends.clone();
		let mut handle_error = handle_error.clone();
		spawn(async move {
			match AddressBook::update_friend(id, &address, &alias) {
				Ok(()) => {
					// 使用统一的加载函数重新加载列表
					load_friends(None);
				}
				Err(e) => {
					handle_error(format!("更新好友失败: {}", e));
				}
			}
		});
	};
	
	// 处理删除好友
	let handle_delete_friend = move |id: i64| {
		let load_friends = load_friends.clone();
		let mut handle_error = handle_error.clone();
		spawn(async move {
			match AddressBook::delete_friend(id) {
				Ok(()) => {
					// 使用统一的加载函数重新加载列表
					load_friends(None);
				}
				Err(e) => {
					handle_error(format!("删除好友失败: {}", e));
				}
			}
		});
	};
	
	// 克隆好友列表以避免生命周期问题
	let friends_list = friends.read().clone();
	
	rsx! {
        div {
            style: "
                padding: 20px;
                height: 100%;
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
                    "好友列表"
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
                        placeholder: "搜索好友地址或备注...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                    
                    button {
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
            
            // 好友列表
            if friends_list.is_empty() {
                div {
                    style: "
                        text-align: center;
                        color: #6b7280;
                        padding: 40px;
                    ",
                    "暂无好友，点击左上角\"添加新条目\"按钮添加好友"
                }
            } else {
                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        gap: 8px;
                    ",
                    
                    for friend in friends_list {
                        FriendItem {
                            friend: friend.clone(),
                            on_click: move || handle_edit_friend(friend.clone())
                        }
                    }
                }
            }
            
            // 编辑模态框
            if *show_edit_modal.read() {
                if let Some(friend) = selected_friend.read().as_ref() {
                    FriendEditModal {
                        friend: friend.clone(),
                        on_save: move |params: (i64, String, String)| {
                            handle_save_edit(params.0, params.1, params.2);
                            show_edit_modal.set(false);
                        },
                        on_delete: move |id| {
                            handle_delete_friend(id);
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
fn FriendItem(friend: crate::core::db::FriendEntry, on_click: EventHandler) -> Element {
	rsx! {
        div {
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
            
            // 好友信息
            div {
                style: "flex: 1;",
                
                div {
                    style: "
                        font-weight: 500;
                        color: #1f2937;
                        margin-bottom: 4px;
                    ",
                    "{friend.alias}"
                }
                
                div {
                    style: "
                        font-family: monospace;
                        color: #6b7280;
                        font-size: 14px;
                    ",
                    "{friend.address}"
                }
            }
            
            // 点击提示
            div {
                style: "
                    color: #9ca3af;
                    font-size: 12px;
                ",
                "点击编辑"
            }
        }
    }
}

#[component]
fn FriendEditModal(
	friend: crate::core::db::FriendEntry,
	on_save: EventHandler<(i64, String, String)>,
	on_delete: EventHandler<i64>,
	on_close: EventHandler,
) -> Element {
	let mut address = use_signal(|| friend.address.clone());
	let mut alias = use_signal(|| friend.alias.clone());
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
                    "编辑好友"
                }
                
                // 表单
                div {
                    style: "display: flex; flex-direction: column; gap: 16px;",
                    
                    // 地址输入
                    div {
                        label {
                            style: "
                                display: block;
                                margin-bottom: 6px;
                                font-weight: 500;
                                color: #374151;
                            ",
                            "IPv6地址"
                        }
                        input {
                            style: "
                                width: 100%;
                                padding: 10px 12px;
                                border: 1px solid #d1d5db;
                                border-radius: 6px;
                                font-size: 14px;
                                font-family: monospace;
                            ",
                            value: "{address}",
                            oninput: move |e| address.set(e.value())
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
                                width: 100%;
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
                                    on_delete.call(friend.id);
                                },
                                "确认删除"
                            }
                            
                            button {
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
                                on_save.call((friend.id, address.read().clone(), alias.read().clone()));
                            },
                            "保存"
                        }
                    }
                }
            }
        }
    }
}