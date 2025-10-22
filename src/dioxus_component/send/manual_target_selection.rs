// src/dioxus_component/send/target_select.rs
use dioxus::prelude::*;

#[component]
pub fn ManualTargetSelect(
	target_ip: Signal<String>,
	disabled: bool,
) -> Element {
	rsx! {
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
                disabled: disabled,
                oninput: move |e| target_ip.set(e.value()),
            }
            p {
                style: "margin: 8px 0 0 0; color: #6b7280; font-size: 12px;",
                "留空将默认使用 ::1 (本地回环地址)"
            }
        }
    }
}