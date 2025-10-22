// src/dioxus_component/send/friends_target_selection.rs
use std::rc::Rc;
use dioxus::prelude::*;
use crate::core::db::AddressBook;

#[component]
pub fn FriendsTargetSelection(
	selected_targets: Signal<Vec<String>>,
	disabled: bool,
) -> Element {
	let friends = use_signal(|| Vec::new());
	let search_query = use_signal(|| String::new());
	let error_message = use_signal(|| None::<String>);
	let mut show_friends_modal = use_signal(|| false);
	
	// 加载好友列表
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
							format!("搜索好友失败: {}", e)
						} else {
							format!("加载好友列表失败: {}", e)
						};
						error_message.set(Some(error_msg));
					}
				}
			});
		}
	};
	
	// 初始加载好友列表
	use_effect(use_reactive((), move |_| {
		load_friends(None);
	}));
	
	// 处理搜索
	let handle_search = {
		let load_friends = load_friends.clone();
		move |_| {
			let query = search_query.read().clone();
			if query.is_empty() {
				load_friends(None);
			} else {
				load_friends(Some(query));
			}
		}
	};
	
	// 切换好友选择状态
	let mut toggle_friend_selection = {
		let mut selected_targets = selected_targets.clone();
		move |address: String| {
			let mut current_targets = selected_targets.write();
			if current_targets.contains(&address) {
				// 如果已经选中，则移除
				current_targets.retain(|addr| addr != &address);
			} else {
				// 如果未选中，则添加
				current_targets.push(address);
			}
		}
	};
	
	// 检查好友是否被选中
	let is_friend_selected: Rc<dyn Fn(&str) -> bool> = Rc::new(move |address: &str| -> bool {
		selected_targets.read().contains(&address.to_string())
	});
	
	// 获取选中好友数量
	let _selected_count = selected_targets.read().len();
	
	// 清除所有选择
	let clear_selection = {
		let mut selected_targets = selected_targets.clone();
		move |_| {
			selected_targets.write().clear();
		}
	};
	
	let selected_targets_owned: Vec<String> = selected_targets.read().clone();
	let selected_targets_pairs: Vec<(String, String)> = selected_targets_owned
		.into_iter()
		.map(|t| {
			// t 是 String（owned），这里再 clone 一份用于事件闭包
			let for_click = t.clone();
			(t, for_click)
		})
		.collect();
	
	let selected_count = selected_targets_pairs.len();
	
	rsx! {
        div {
            style: "margin-bottom: 24px;",
            
            // 标题和选择信息
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 12px;
                ",
                
                label {
                    style: "
                        display: block;
                        font-weight: 600;
                        color: #374151;
                        font-size: 16px;
                    ",
                    "选择好友作为目标"
                }
                
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    
                    if selected_count > 0 {
                        span {
                            style: "
                                color: #059669;
                                font-size: 14px;
                                font-weight: 500;
                            ",
                            "已选择 {selected_count} 个好友"
                        }
                        
                        button {
                            class: "clear-selection-button",
                            style: "
                                padding: 4px 8px;
                                background: #ef4444;
                                color: white;
                                border: none;
                                border-radius: 4px;
                                cursor: pointer;
                                font-size: 12px;
                            ",
                            onclick: clear_selection,
                            disabled: disabled,
                            "清除选择"
                        }
                    }
                }
            }
            
            // 选择好友按钮
            button {
                class: "select-friends-button",
                style: "
                    width: 100%;
                    padding: 12px;
                    border: 2px dashed #d1d5db;
                    border-radius: 8px;
                    background: #f9fafb;
                    color: #374151;
                    cursor: pointer;
                    transition: all 0.2s;
                    text-align: center;
                    margin-bottom: 12px;
                ",
                onclick: move |_| {
                    if !disabled {
                        show_friends_modal.set(true);
                    }
                },
                disabled: disabled,
                
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 4px;",
                    
                    span {
                        style: "font-size: 14px;",
                        if selected_count == 0 {
                            "点击选择好友"
                        } else {
                            "点击管理已选择的好友"
                        }
                    }
                    
                    if selected_count > 0 {
                        span {
                            style: "font-size: 12px; color: #6b7280;",
                            "已选择 {selected_count} 个好友"
                        }
                    }
                }
            }
            
            // 选中的好友预览
            if selected_count > 0 {
                div {
                    style: "
                        background: #f0f9ff;
                        border: 1px solid #bae6fd;
                        border-radius: 6px;
                        padding: 12px;
                    ",
                    
                    h4 {
                        style: "
                            margin: 0 0 8px 0;
                            font-size: 14px;
                            color: #0369a1;
                        ",
                        "已选择的好友:"
                    }
                    
                    div {
                        style: "
                            display: flex;
                            flex-wrap: wrap;
                            gap: 6px;
                        ",
                        
                        for (display, click_val) in selected_targets_pairs.into_iter() {
                            div {
                                key: "{display}",
                                style: "
                                    display: inline-flex;
                                    align-items: center;
                                    background: white;
                                    border: 1px solid #7dd3fc;
                                    border-radius: 16px;
                                    padding: 4px 8px;
                                    font-size: 12px;
                                ",

                                span { style: "margin-right: 6px;", "{display}" }

                                button {
                                    style: "
                                        background: none;
                                        border: none;
                                        color: #ef4444;
                                        cursor: pointer;
                                        padding: 2px;
                                        border-radius: 50%;
                                        width: 16px;
                                        height: 16px;
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                    ",
                                    // 使用 move 闭包并把 click_val（owned）移动进闭包，安全且不会借用临时
                                    onclick: move |_| {
                                        toggle_friend_selection(click_val.clone());
                                    },
                                    disabled: disabled,
                                    "×"
                                }
                            }
                        }
                    }
                }
            }
            
            // 好友选择模态框
            if *show_friends_modal.read() {
                FriendsSelectionModal {
                    friends: friends.read().clone(),
                    selected_targets: selected_targets.clone(),
                    search_query: search_query.clone(),
                    on_search: move |_| handle_search(()),
                    on_toggle_selection: toggle_friend_selection,
                    is_friend_selected: is_friend_selected.clone(),
                    on_close: move |_| show_friends_modal.set(false),
                    disabled: disabled,
                }
            }
        }
    }
}

#[derive(Props, Clone)]
struct FriendsSelectionModalProps {
	friends: Vec<crate::core::db::FriendEntry>,
	selected_targets: Signal<Vec<String>>,
	search_query: Signal<String>,
	on_search: EventHandler,
	on_toggle_selection: EventHandler<String>,
	is_friend_selected: Rc<dyn Fn(&str) -> bool>,
	on_close: EventHandler,
	disabled: bool,
}

#[component]
fn FriendsSelectionModal(props: FriendsSelectionModalProps) -> Element {
	let FriendsSelectionModalProps {
		friends,
		mut selected_targets,
		mut search_query,
		on_search,
		on_toggle_selection,
		is_friend_selected,
		on_close,
		disabled,
	} = props;
	
	let selected_count = selected_targets.read().len();
	
	// 全选/取消全选
	let friends_clone = friends.clone();
	let toggle_select_all = move |_| {
		if disabled { return; }
		
		let all_addresses: Vec<String> = friends_clone.iter()
			.map(|f| f.address.clone())
			.collect();
		
		let mut current_selection = selected_targets.write();
		
		// 如果已经全选，则清空；否则选择所有
		if current_selection.len() == all_addresses.len() {
			current_selection.clear();
		} else {
			*current_selection = all_addresses;
		}
	};
	
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
                    max-width: 600px;
                    max-height: 80vh;
                    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
                    display: flex;
                    flex-direction: column;
                ",
                onclick: move |e| e.stop_propagation(),
                
                // 模态框头部
                div {
                    style: "margin-bottom: 20px;",
                    
                    h3 {
                        style: "margin: 0 0 16px 0; color: #1f2937;",
                        "选择好友"
                    }
                    
                    // 搜索栏
                    div {
                        style: "display: flex; gap: 12px; margin-bottom: 16px;",
                        
                        input {
                            style: "
                                flex: 1;
                                padding: 10px 12px;
                                border: 1px solid #d1d5db;
                                border-radius: 6px;
                                font-size: 14px;
                            ",
                            placeholder: "搜索好友...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value())
                        }
                        
                        button {
                            class: "search-button",
                            style: "
                                padding: 10px 16px;
                                background: #3b82f6;
                                color: white;
                                border: none;
                                border-radius: 6px;
                                cursor: pointer;
                            ",
                            onclick: move |_| on_search.call(()),
                            disabled: disabled,
                            "搜索"
                        }
                    }
                    
                    // 选择信息
                    div {
                        style: "
                            display: flex;
                            justify-content: space-between;
                            align-items: center;
                            padding: 8px 0;
                            border-bottom: 1px solid #e5e7eb;
                        ",
                        
                        span {
                            style: "color: #6b7280; font-size: 14px;",
                            if selected_count == 0 {
                                "未选择任何好友"
                            } else {
                                "已选择 {selected_count} 个好友"
                            }
                        }
                        
                        button {
                            class: "select-all-button",
                            style: "
                                padding: 6px 12px;
                                background: #6b7280;
                                color: white;
                                border: none;
                                border-radius: 4px;
                                cursor: pointer;
                                font-size: 12px;
                            ",
                            onclick: toggle_select_all,
                            disabled: disabled,
                            if selected_count == friends.len() && !friends.is_empty() {
                                "取消全选"
                            } else {
                                "全选"
                            }
                        }
                    }
                }
                
                // 好友列表
                div {
                    style: "
                        flex: 1;
                        overflow-y: auto;
                        max-height: 400px;
                    ",
                    
                    if friends.is_empty() {
                        div {
                            style: "
                                text-align: center;
                                color: #6b7280;
                                padding: 40px;
                            ",
                            "暂无好友"
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            
                            for friend in friends {
                                FriendSelectionItem {
                                    friend: friend.clone(),
                                    is_selected: is_friend_selected(&friend.address),
                                    on_toggle: move || on_toggle_selection.call(friend.address.clone()),
                                    disabled: disabled,
                                }
                            }
                        }
                    }
                }
                
                // 底部按钮
                div {
                    style: "
                        display: flex;
                        justify-content: flex-end;
                        margin-top: 20px;
                        padding-top: 16px;
                        border-top: 1px solid #e5e7eb;
                    ",
                    
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
                        onclick: move |_| on_close.call(()),
                        "完成"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FriendSelectionItemProps {
	friend: crate::core::db::FriendEntry,
	is_selected: bool,
	on_toggle: EventHandler,
	disabled: bool,
}

#[component]
fn FriendSelectionItem(props: FriendSelectionItemProps) -> Element {
	let FriendSelectionItemProps {
		friend,
		is_selected,
		on_toggle,
		disabled,
	} = props;
	
	rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                padding: 12px;
                border: 1px solid #e5e7eb;
                border-radius: 8px;
                background: white;
                transition: all 0.2s;
                cursor: pointer;
            ",
            onclick: move |_| {
                if !disabled {
                    on_toggle.call(());
                }
            },
            
            // 选择框
            input {
                r#type: "checkbox",
                style: "
                    margin-right: 12px;
                    width: 18px;
                    height: 18px;
                    cursor: pointer;
                ",
                checked: is_selected,
                onchange: move |e| {
                    if e.checked() != is_selected && !disabled {
                        on_toggle.call(());
                    }
                },
                disabled: disabled,
            }
            
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
            
            // 选中状态指示器
            if is_selected {
                div {
                    style: "
                        color: #10b981;
                        font-weight: bold;
                    ",
                    "✓"
                }
            }
        }
    }
}

impl PartialEq for FriendsSelectionModalProps {
	fn eq(&self, other: &Self) -> bool {
		self.friends == other.friends &&
			self.selected_targets == other.selected_targets &&
			self.search_query == other.search_query &&
			self.disabled == other.disabled
		// 注意：我们不比较 is_friend_selected，因为函数指针不能比较
	}
}