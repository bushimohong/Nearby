// src/dioxus_component/receive/history
use dioxus::prelude::*;
use crate::core::db::{AddressBook, FileReceiveRecord};
use chrono::{DateTime, Local, NaiveDateTime};
use humansize::{format_size, DECIMAL};

#[component]
pub fn HistoryWindow(on_close: EventHandler) -> Element {
    let mut file_records = use_signal(|| Vec::<FileReceiveRecord>::new());
    let mut selected_record = use_signal(|| None);
    let mut search_query = use_signal(|| String::new());
    let mut show_detail_dialog = use_signal(|| false);
    
    // 加载历史记录
    use_effect(move || {
        spawn(async move {
            match AddressBook::get_all_file_receive_records() {
                Ok(records) => {
                    file_records.set(records);
                }
                Err(e) => {
                    log::error!("加载历史记录失败: {}", e);
                }
            }
        });
    });
    
    // 搜索功能
    let filtered_records = use_memo(move || {
        let query = search_query.read().to_lowercase();
        let records = file_records.read().clone();
        if query.is_empty() {
            records
        } else {
            records
                .into_iter()
                .filter(|record| {
                    record.filename.to_lowercase().contains(&query) ||
                        record.sender_identity.to_lowercase().contains(&query) ||
                        record.sender_ipv6.to_lowercase().contains(&query)
                })
                .collect()
        }
    });
    
    // 格式化日期
    fn format_date(date_str: &str) -> String {
        if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            let local_datetime: DateTime<Local> = DateTime::from_naive_utc_and_offset(naive_datetime, Local::now().offset().clone());
            local_datetime.format("%Y-%m-%d %H:%M").to_string()
        } else {
            date_str.to_string()
        }
    }
    
    // 处理记录点击 - 修复：添加 mut
    let mut handle_record_click = move |record: FileReceiveRecord| {
        selected_record.set(Some(record));
        show_detail_dialog.set(true);
    };
    
    // 处理删除记录
    let handle_delete_record = move |id: i64| {
        spawn(async move {
            if let Err(e) = AddressBook::delete_file_receive_record(id) {
                log::error!("删除记录失败: {}", e);
            } else {
                // 重新加载记录
                match AddressBook::get_all_file_receive_records() {
                    Ok(records) => {
                        file_records.set(records);
                    }
                    Err(e) => {
                        log::error!("重新加载记录失败: {}", e);
                    }
                }
            }
        });
    };
    
    // 清除所有记录
    let handle_clear_all = move || {
        spawn(async move {
            if let Err(e) = AddressBook::delete_all_file_receive_records() {
                log::error!("清除所有记录失败: {}", e);
            } else {
                file_records.set(Vec::new());
            }
        });
    };
    
    rsx! {
        div {
            style: "
                position: fixed;
                top: 0;
                left: 0;
                width: 100vw;
                height: 100vh;
                background-color: white;
                z-index: 1000;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            ",
            
            // 标题栏
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 20px 24px;
                    border-bottom: 1px solid #e0e0e0;
                    background-color: #fafafa;
                    flex-shrink: 0;
                ",
               
                button {
                    class: "back-item",
                    style: "
                        background: none;
                        border: none;
                        font-size: 24px;
                        cursor: pointer;
                        padding: 6px 12px;
                        border-radius: 6px;
                        color: #666;
                        transition: all 0.2s ease;
                        width: 48px;
                        height: 48px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    ",
                    onclick: move |_| on_close.call(()),
                    img {
                        style: "width: 30px; height: 30px;",
                        src: asset!("assets/back-100.png")
                    }
                }
                
                h2 {
                    style: "margin: 0; font-size: 24px; color: #333;",
                    "历史记录"
                }
                
                button {
                    style: "
                        background: #ff4444;
                        color: white;
                        border: none;
                        padding: 8px 16px;
                        border-radius: 6px;
                        cursor: pointer;
                        font-size: 14px;
                    ",
                    onclick: move |_| handle_clear_all(),
                    "清除全部"
                }
            }

            // 搜索栏
            div {
                style: "
                    padding: 16px 24px;
                    border-bottom: 1px solid #e0e0e0;
                    background-color: #f8f8f8;
                    flex-shrink: 0;
                ",
                input {
                    style: "
                        width: 90%;
                        padding: 12px 16px;
                        border: 1px solid #ddd;
                        border-radius: 8px;
                        font-size: 14px;
                        outline: none;
                    ",
                    r#type: "text",
                    placeholder: "搜索文件名、身份码或IP地址...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value())
                }
            }
            
            // 历史记录内容区域
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: 0;
                ",
                
                if filtered_records.read().is_empty() {
                    p {
                        style: "
                            text-align: center;
                            color: #999;
                            margin-top: 50px;
                            font-size: 16px;
                        ",
                        if search_query.read().is_empty() {
                            "暂无历史记录"
                        } else {
                            "未找到匹配的记录"
                        }
                    }
                } else {
                    div {
                        style: "padding: 0;",
                        {filtered_records.read().iter().cloned().map(|record| {
                            rsx! {
                                div {
                                    key: "{record.id}",
                                    style: "
                                        display: flex;
                                        align-items: center;
                                        justify-content: space-between;
                                        padding: 16px 24px;
                                        border-bottom: 1px solid #f0f0f0;
                                        cursor: pointer;
                                        transition: all 0.2s ease;
                                        background-color: white;
                                    ",
                                    class: "history-item",
                                    onclick: move |_| handle_record_click(record.clone()),
                        
                                    // 左侧文件信息
                                    div {
                                        style: "flex: 1;",
                                        div {
                                            style: "
                                                font-size: 16px;
                                                font-weight: 500;
                                                color: #333;
                                                margin-bottom: 4px;
                                                display: flex;
                                                align-items: center;
                                            ",
                                            "{record.filename}"
                                        }
                                        div {
                                            style: "
                                                display: flex;
                                                gap: 16px;
                                                font-size: 12px;
                                                color: #666;
                                            ",
                                            span {
                                                "{format_date(&record.received_at)}"
                                            }
                                            span {
                                                "{format_size(record.file_size, DECIMAL)}"
                                            }
                                        }
                                    }
                        
                                    // 右侧更多按钮
                                    button {
                                        style: "
                                            background: none;
                                            border: none;
                                            cursor: pointer;
                                            padding: 8px;
                                            border-radius: 4px;
                                        ",
                                        class: "more-button",
                                        img {
                                            style: "width: 20px; height: 20px;",
                                            src: asset!("assets/more-100.png")
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }

        // 详细信息对话框
        if *show_detail_dialog.read() {
            // 修复：克隆 selected_record 避免生命周期问题
            if let Some(record) = selected_record.read().clone() {
                div {
                    div {
                        style: "
                            position: fixed;
                            top: 0;
                            left: 0;
                            width: 100vw;
                            height: 100vh;
                            background-color: rgba(0, 0, 0, 0.5);
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            z-index: 2000;
                        ",
                        onclick: move |_| show_detail_dialog.set(false),
                        
                        div {
                            style: "
                                background: white;
                                border-radius: 12px;
                                padding: 24px;
                                width: 90%;
                                max-width: 500px;
                                max-height: 80vh;
                                overflow-y: auto;
                                box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
                            ",
                            onclick: move |e| e.stop_propagation(),
                            
                            // 对话框标题
                            div {
                                style: "
                                    display: flex;
                                    justify-content: space-between;
                                    align-items: center;
                                    margin-bottom: 20px;
                                    padding-bottom: 16px;
                                    border-bottom: 1px solid #e0e0e0;
                                ",
                                h3 {
                                    style: "margin: 0; font-size: 20px; color: #333;",
                                    "文件详情"
                                }
                                button {
                                    style: "
                                        background: none;
                                        border: none;
                                        font-size: 24px;
                                        cursor: pointer;
                                        color: #999;
                                        padding: 4px;
                                        border-radius: 4px;
                                    ",
                                    onclick: move |_| show_detail_dialog.set(false),
                                    "×"
                                }
                            }
                            
                            // 详细信息内容
                            div {
                                style: "display: flex; flex-direction: column; gap: 12px;",
                                
                                DetailItem {
                                    label: "文件名".to_string(),
                                    value: record.filename.clone()
                                }
                                DetailItem {
                                    label: "文件大小".to_string(),
                                    value: format_size(record.file_size, DECIMAL)
                                }
                                DetailItem {
                                    label: "发送方身份码".to_string(),
                                    value: record.sender_identity.clone()
                                }
                                DetailItem {
                                    label: "发送方IP地址".to_string(),
                                    value: record.sender_ipv6.clone()
                                }
                                DetailItem {
                                    label: "接收时间".to_string(),
                                    value: format_date(&record.received_at)
                                }
                                DetailItem {
                                    label: "保存路径".to_string(),
                                    value: record.save_path.clone()
                                }
                            }
                            
                            // 操作按钮
                            div {
                                style: "
                                    display: flex;
                                    justify-content: flex-end;
                                    gap: 12px;
                                    margin-top: 24px;
                                    padding-top: 16px;
                                    border-top: 1px solid #e0e0e0;
                                ",
                                button {
                                    style: "
                                        background: #f0f0f0;
                                        color: #333;
                                        border: none;
                                        padding: 10px 20px;
                                        border-radius: 6px;
                                        cursor: pointer;
                                        font-size: 14px;
                                    ",
                                    onclick: move |_| show_detail_dialog.set(false),
                                    "关闭"
                                }
                                button {
                                    style: "
                                        background: #ff4444;
                                        color: white;
                                        border: none;
                                        padding: 10px 20px;
                                        border-radius: 6px;
                                        cursor: pointer;
                                        font-size: 14px;
                                    ",
                                    onclick: move |_| {
                                        handle_delete_record(record.id);
                                        show_detail_dialog.set(false);
                                    },
                                    "删除记录"
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
fn DetailItem(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                gap: 4px;
            ",
            div {
                style: "
                    font-size: 14px;
                    font-weight: 500;
                    color: #666;
                ",
                "{label}"
            }
            div {
                style: "
                    font-size: 16px;
                    color: #333;
                    word-break: break-all;
                    padding: 8px 12px;
                    background-color: #f8f8f8;
                    border-radius: 6px;
                    border: 1px solid #e0e0e0;
                ",
                "{value}"
            }
        }
    }
}