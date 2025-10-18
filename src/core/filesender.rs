// src/core/filesend.rs
use std::error;
use std::net::SocketAddrV6;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::core::db::AddressBook;
use log::{info, error};

pub struct FileSender;

impl FileSender {
    pub async fn send_file(
        ipv6_addr: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn error::Error>> {
        // 如果 IP 地址为空，默认使用本地地址 (::1)
        let actual_ip = if ipv6_addr.is_empty() {
            "::1"
        } else {
            ipv6_addr
        };
        
        // 解析IPv6地址和端口
        let socket_addr = format!("[{}]:6789", actual_ip);
        let addr: SocketAddrV6 = socket_addr.parse()?;
        
        info!("正在连接到接收方: {}", addr);
        
        // 连接到接收方
        let mut stream = TcpStream::connect(addr).await?;
        info!("已连接到接收方: {}", addr);
        
        // 获取自己的身份码
        let my_identity = AddressBook::get_my_identity()?;
        if my_identity.len() != 64 {
            return Err("身份码长度不正确，必须为64字符".into());
        }
        
        info!("使用身份码: {}", my_identity);
        
        // 异步打开要发送的文件
        let mut file = File::open(file_path).await?;
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        // 获取文件大小
        let file_size = file.metadata().await?.len();
        info!("开始发送文件: {} ({} 字节)", file_name, file_size);
        
        // 首先发送身份码 (64字符固定长度)
        stream.write_all(my_identity.as_bytes()).await?;
        info!("已发送身份码");
        
        // 发送文件名长度和文件名
        let file_name_bytes = file_name.as_bytes();
        let file_name_len = file_name_bytes.len() as u64;
        
        stream.write_u64(file_name_len).await?;
        stream.write_all(file_name_bytes).await?;
        info!("已发送文件名: {}", file_name);
        
        // 发送文件大小
        stream.write_u64(file_size).await?;
        info!("已发送文件大小: {} 字节", file_size);
        
        // 使用缓冲区异步发送文件内容
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB 缓冲区
        let mut total_sent = 0;
        
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            // 异步写入网络流
            stream.write_all(&buffer[..bytes_read]).await?;
            total_sent += bytes_read;
            
            // 每发送 1MB 打印一次进度，避免频繁打印
            if total_sent % (1024 * 1024) < 64 * 1024 || total_sent == file_size as usize {
                info!("已发送: {}/{} 字节 ({:.1}%)",
                         total_sent, file_size,
                         (total_sent as f64 / file_size as f64) * 100.0);
            }
        }
        
        // 确保所有数据都被刷新
        stream.flush().await?;
        
        info!("文件发送完成: {}", file_name);
        Ok(())
    }
    
    pub async fn select_file() -> Result<Option<String>, Box<dyn error::Error>> {
        // 使用rfd选择文件
        let file_handle = rfd::AsyncFileDialog::new()
            .set_title("选择要发送的文件")
            .pick_file()
            .await;
        
        if let Some(file) = file_handle {
            let file_path = file.path().to_string_lossy().to_string();
            Ok(Some(file_path))
        } else {
            error!("未选择文件");
            Ok(None)
        }
    }
}