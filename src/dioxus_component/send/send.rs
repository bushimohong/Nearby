use dioxus::prelude::*;
use crate::core::filesender::FileSender;

#[component]
pub fn Send() -> Element {
    let mut target_ip = use_signal(|| String::from("::1"));
    let mut status_message = use_signal(|| String::from("准备就绪"));
    let mut selected_files = use_signal(|| Vec::<String>::new());
    let is_sending = use_signal(|| false);
    
    rsx! {
        div {
            style: "
                height: 100%;
                display: flex;
                flex-direction: column;
                background-color: #f8fafc;
                overflow: hidden;
            ",
            
            // 可滚动的内容区域
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: 20px;
                ",
                
                // 标题区域
                div {
                    style: "
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        padding: 24px;
                        border-radius: 12px;
                        margin-bottom: 24px;
                        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                    ",
                    h1 {
                        style: "margin: 0 0 8px 0; font-size: 24px; font-weight: 700;",
                        "发送文件"
                    }
                    p {
                        style: "margin: 0; opacity: 0.9; font-size: 14px;",
                        "目标端口: 6789 • 支持多文件传输"
                    }
                }
                
                // 主内容卡片
                div {
                    style: "
                        background-color: white;
                        padding: 24px;
                        border-radius: 12px;
                        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
                        margin-bottom: 24px;
                    ",
                    
                    // 文件选择区域
                    div {
                        style: "margin-bottom: 24px;",
                        label {
                            style: "
                                display: block;
                                font-weight: 600;
                                margin-bottom: 12px;
                                color: #374151;
                                font-size: 16px;
                            ",
                            "选择文件"
                        }
                        
                        div {
                            style: "
                                display: flex;
                                align-items: center;
                                gap: 12px;
                                margin-bottom: 16px;
                            ",
                            button {
                                style: "
                                    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
                                    color: white;
                                    padding: 12px 24px;
                                    border: none;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-size: 14px;
                                    font-weight: 600;
                                    transition: all 0.2s;
                                    box-shadow: 0 2px 4px rgba(16, 185, 129, 0.3);
                                ",
                                disabled: *is_sending.read(),
                                onclick: move |_| {
                                    to_owned![selected_files, status_message];
                                    async move {
                                        status_message.set("选择文件中...".to_string());
                                        match FileSender::select_file().await {
                                            Ok(Some(file_path)) => {
                                                let mut files = selected_files.write();
                                                if !files.contains(&file_path) {
                                                    files.push(file_path.clone());
                                                    status_message.set(format!("📁 已添加文件: {}", file_path));
                                                } else {
                                                    status_message.set("文件已存在列表中".to_string());
                                                }
                                            }
                                            Ok(None) => status_message.set("未选择文件".to_string()),
                                            Err(e) => status_message.set(format!("选择文件失败: {}", e)),
                                        }
                                    }
                                },
                                "添加文件"
                            }
                            
                            button {
                                style: "
                                    background-color: #ef4444;
                                    color: white;
                                    padding: 12px 24px;
                                    border: none;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    font-size: 14px;
                                    font-weight: 600;
                                    transition: all 0.2s;
                                ",
                                disabled: *is_sending.read(),
                                onclick: move |_| {
                                    selected_files.write().clear();
                                    status_message.set("已清空文件列表".to_string());
                                },
                                "清空列表"
                            }
                            
                            span {
                                style: "color: #6b7280; font-size: 14px;",
                                "已选择 {selected_files.read().len()} 个文件"
                            }
                        }
                    }

                    // 文件列表
                    if !selected_files.read().is_empty() {
                        div {
                            style: "
                                background-color: #f9fafb;
                                border: 1px solid #e5e7eb;
                                border-radius: 8px;
                                padding: 16px;
                                margin-bottom: 24px;
                                max-height: 300px;
                                overflow-y: auto;
                            ",
                            h3 {
                                style: "
                                    margin: 0 0 12px 0;
                                    font-size: 14px;
                                    font-weight: 600;
                                    color: #374151;
                                ",
                                "已选择的文件"
                            }
                            
                            div {
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                for (index, file_path) in selected_files.read().iter().enumerate() {
                                    div {
                                        key: "{index}",
                                        style: "
                                            display: flex;
                                            justify-content: space-between;
                                            align-items: center;
                                            padding: 8px 12px;
                                            background-color: white;
                                            border-radius: 6px;
                                            border: 1px solid #e5e7eb;
                                        ",
                                        span {
                                            style: "font-size: 14px; color: #374151;",
                                            "{file_path}"
                                        }
                                        button {
                                            style: "
                                                background-color: #ef4444;
                                                color: white;
                                                border: none;
                                                border-radius: 4px;
                                                padding: 4px 8px;
                                                cursor: pointer;
                                                font-size: 12px;
                                            ",
                                            disabled: *is_sending.read(),
                                            onclick: move |_| {
                                                selected_files.write().remove(index);
                                                status_message.set("已移除文件".to_string());
                                            },
                                            "移除"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // IPv6 输入区域
                    div {
                        style: "margin-bottom: 24px;",
                        label {
                            style: "
                                display: block;
                                font-weight: 600;
                                margin-bottom: 8px;
                                color: #374151;
                                font-size: 16px;
                            ",
                            "目标 IPv6 地址"
                        }
                        input {
                            style: "
                                width: 100%;
                                padding: 12px;
                                border: 1px solid #d1d5db;
                                border-radius: 8px;
                                box-sizing: border-box;
                                outline: none;
                                font-size: 14px;
                                transition: border-color 0.2s;
                            ",
                            r#type: "text",
                            placeholder: "例如: ::1 或 其他 IPv6 地址",
                            value: "{target_ip}",
                            oninput: move |e| target_ip.set(e.value()),
                        }
                        p {
                            style: "margin: 8px 0 0 0; color: #6b7280; font-size: 12px;",
                            "留空将默认使用 ::1 (本地回环地址)"
                        }
                    }

                    // 发送按钮
                    button {
                        style: "
                            background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
                            color: white;
                            padding: 14px 28px;
                            border: none;
                            border-radius: 8px;
                            cursor: pointer;
                            font-size: 16px;
                            font-weight: 600;
                            transition: all 0.2s;
                            box-shadow: 0 2px 4px rgba(59, 130, 246, 0.3);
                            width: 100%;
                        ",
                        disabled: selected_files.read().is_empty() || *is_sending.read(),
                        onclick: move |_| {
                            to_owned![target_ip, selected_files, status_message, is_sending];
                            let ip = target_ip.read().clone();
                            let files = selected_files.read().clone();

                            async move {
                                is_sending.set(true);
                                
                                if files.is_empty() {
                                    status_message.set("请先选择文件".to_string());
                                    is_sending.set(false);
                                    return;
                                }

                                let target = if ip.is_empty() { "::1" } else { &ip };
                                status_message.set(format!("📦 准备发送 {} 个文件到 {}...", files.len(), target));

                                let mut success_count = 0;
                                let mut fail_count = 0;

                                for (index, file_path) in files.iter().enumerate() {
                                    status_message.set(format!("正在发送文件 {}/{}: {}", index + 1, files.len(), file_path));
                                    
                                    match FileSender::send_file(&ip, file_path).await {
                                        Ok(_) => {
                                            println!("发送成功: {}", file_path);
                                            success_count += 1;
                                        },
                                        Err(e) => {
                                            println!("发送失败: {} - {}", file_path, e);
                                            fail_count += 1;
                                        },
                                    }
                                }

                                if fail_count == 0 {
                                    status_message.set(format!("✅ 所有文件发送完成 ({} 个文件)", success_count));
                                } else {
                                    status_message.set(format!("⚠️ 发送完成: {} 成功, {} 失败", success_count, fail_count));
                                }
                                
                                is_sending.set(false);
                            }
                        },
                        if *is_sending.read() {
                            "发送中..."
                        } else {
                            "发送所有文件"
                        }
                    }
                }

                // 状态栏
                div {
                    style: "
                        background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
                        padding: 16px;
                        border-radius: 8px;
                        border-left: 4px solid #3b82f6;
                    ",
                    h3 {
                        style: "
                            margin: 0 0 8px 0;
                            color: #1e293b;
                            font-size: 14px;
                            font-weight: 600;
                        ",
                        "传输状态"
                    }
                    p {
                        style: "
                            margin: 0;
                            color: #334155;
                            font-size: 14px;
                            min-height: 20px;
                        ",
                        "{status_message}"
                    }
                }
            }
        }
    }
}