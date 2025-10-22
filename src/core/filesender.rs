// src/core/filesender.rs
use std::error;
use std::net::SocketAddrV6;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::core::db::AddressBook;
use log::{info, error};
use tokio::sync::Semaphore;

const CONCURRENT_TRANSFERS: usize = 5;
type SendError = Box<dyn error::Error + Send + Sync>;

pub struct FileSender;

impl FileSender {
    // 执行 Noise 协议握手（作为发起者）
    async fn perform_noise_handshake(stream: &mut TcpStream) -> Result<snow::TransportState, SendError> {
        info!("开始 Noise 协议握手...");
        
        // 创建发起者
        let builder = snow::Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?);
        let static_key = builder.generate_keypair()?.private;
        let mut noise = builder
            .local_private_key(&static_key)
            .build_initiator()?;
        
        // 发送第一条握手消息
        info!("准备发送第一条握手消息...");
        let mut handshake_buffer1 = vec![0u8; 65535];
        let len = noise.write_message(&[], &mut handshake_buffer1)?;
        
        stream.write_u16(len as u16).await?;
        stream.write_all(&handshake_buffer1[..len]).await?;
        info!("成功发送第一条握手消息");
        
        // 接收响应消息
        info!("等待接收响应消息...");
        let len = stream.read_u16().await? as usize;
        let mut msg = vec![0u8; len];
        stream.read_exact(&mut msg).await?;
        info!("成功接收响应消息");
        
        // 读取响应消息
        let mut handshake_buffer2 = vec![0u8; 65535];
        noise.read_message(&msg, &mut handshake_buffer2)?;
        info!("成功处理响应消息");
        
        // 发送第三条握手消息
        info!("准备发送第三条握手消息...");
        let mut handshake_buffer3 = vec![0u8; 65535];
        let len = noise.write_message(&[], &mut handshake_buffer3)?;
        
        stream.write_u16(len as u16).await?;
        stream.write_all(&handshake_buffer3[..len]).await?;
        info!("成功发送第三条握手消息");
        
        // 转换为传输模式
        let transport = noise.into_transport_mode()?;
        info!("Noise 协议握手完成");
        
        Ok(transport)
    }
    
    // 使用加密通道读取数据
    #[allow(dead_code)]
    async fn read_encrypted(transport: &mut snow::TransportState, stream: &mut TcpStream, buffer: &mut [u8]) -> Result<usize, Box<dyn error::Error>> {
        // 先读取加密数据的长度
        let encrypted_len = match stream.read_u16().await {
            Ok(len) => len as usize,
            Err(e) => {
                error!("读取加密数据长度失败: {}", e);
                return Err(e.into());
            }
        };
        
        if encrypted_len == 0 {
            // 长度为0表示传输结束
            return Ok(0);
        }
        
        let mut encrypted_data = vec![0u8; encrypted_len];
        if let Err(e) = stream.read_exact(&mut encrypted_data).await {
            error!("读取加密数据内容失败: {}", e);
            return Err(e.into());
        }
        
        // 解密数据
        let len = match transport.read_message(&encrypted_data, buffer) {
            Ok(l) => l,
            Err(e) => {
                error!("解密数据失败: {}", e);
                return Err(e.into());
            }
        };
        
        Ok(len)
    }
    
    // 使用加密通道写入数据
    async fn write_encrypted(transport: &mut snow::TransportState, stream: &mut TcpStream, data: &[u8]) -> Result<(), SendError> {
        let mut buffer = vec![0u8; 65535];
        
        // 加密数据
        let len = transport.write_message(data, &mut buffer)?;
        
        // 发送加密数据的长度和数据
        stream.write_u16(len as u16).await?;
        stream.write_all(&buffer[..len]).await?;
        
        Ok(())
    }
    
    // 等待传输完成确认
    async fn wait_for_transfer_complete(stream: &mut TcpStream) -> Result<(), SendError> {
        info!("等待接收方的传输完成确认...");
        
        // 读取结束信号（长度为0的数据包）
        let end_signal = stream.read_u16().await?;
        
        if end_signal == 0 {
            info!("接收方已确认传输完成");
            Ok(())
        } else {
            error!("无效的传输结束信号: {}", end_signal);
            Err("无效的传输结束信号".into())
        }
    }
    
    // 发送传输结束信号
    async fn send_transfer_complete(stream: &mut TcpStream) -> Result<(), SendError> {
        info!("发送传输结束信号...");
        
        // 发送长度为0的数据包表示传输结束
        stream.write_u16(0).await?;
        stream.flush().await?;
        
        info!("传输结束信号已发送");
        Ok(())
    }
    
    // 发送单个文件的内部实现
    async fn send_single_file(
        ipv6_addr: &str,
        file_path: &str,
    ) -> Result<(), SendError> {
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
        
        // 进行 Noise 协议握手
        let mut transport = Self::perform_noise_handshake(&mut stream).await?;
        
        // 获取自己的身份码
        let my_identity = AddressBook::get_my_identity()?;
        
        if my_identity.len() != 64 {
            error!("身份码长度不正确: {} (期望64字符)", my_identity.len());
            return Err("身份码长度不正确，必须为64字符".into());
        }
        
        info!("使用身份码: {}", my_identity);
        
        // 异步打开要发送的文件
        let mut file = File::open(file_path).await?;
        
        let file_name = match std::path::Path::new(file_path).file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                error!("无法从路径获取文件名: {}", file_path);
                return Err("无效的文件路径".into());
            }
        };
        
        // 获取文件大小
        let file_size = file.metadata().await?.len();
        
        info!("开始发送文件: {} ({} 字节)", file_name, file_size);
        
        // 首先发送身份码 (64字符固定长度) - 使用加密通道
        Self::write_encrypted(&mut transport, &mut stream, my_identity.as_bytes()).await?;
        info!("已发送身份码");
        
        // 发送文件名长度和文件名 - 使用加密通道
        let file_name_bytes = file_name.as_bytes();
        let file_name_len = file_name_bytes.len() as u64;
        
        Self::write_encrypted(&mut transport, &mut stream, &file_name_len.to_be_bytes()).await?;
        Self::write_encrypted(&mut transport, &mut stream, file_name_bytes).await?;
        info!("已发送文件名: {}", file_name);
        
        // 发送文件大小 - 使用加密通道
        Self::write_encrypted(&mut transport, &mut stream, &file_size.to_be_bytes()).await?;
        info!("已发送文件大小: {} 字节", file_size);
        
        // 使用缓冲区异步发送文件内容 - 使用加密通道
        let mut buffer = vec![0u8; 32 * 1024]; // 减少缓冲区大小到32KB，避免加密缓冲区溢出
        let mut total_sent = 0;
        
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            
            if bytes_read == 0 {
                break;
            }
            
            // 使用加密通道发送数据 - 只发送实际读取的数据
            Self::write_encrypted(&mut transport, &mut stream, &buffer[..bytes_read]).await?;
            
            total_sent += bytes_read;
            
            // 每发送 1MB 打印一次进度，避免频繁打印
            if total_sent % (1024 * 1024) < 32 * 1024 || total_sent == file_size as usize {
                info!("已发送: {}/{} 字节 ({:.1}%)",
                         total_sent, file_size,
                         (total_sent as f64 / file_size as f64) * 100.0);
            }
        }
        
        // 确保所有数据都被刷新
        stream.flush().await?;
        
        info!("文件数据发送完成，发送传输结束信号...");
        
        // 发送传输结束信号
        Self::send_transfer_complete(&mut stream).await?;
        
        info!("等待接收方的传输完成确认...");
        
        // 等待接收方的传输完成确认
        Self::wait_for_transfer_complete(&mut stream).await?;
        
        info!("文件传输完成: {}", file_name);
        Ok(())
    }
    
    // 并发传输多个文件
    pub async fn send_files(
        ipv6_addr: &str,
        file_paths: &[String],
    ) -> Result<Vec<(String, Result<(), SendError>)>, SendError> {
        if file_paths.is_empty() {
            return Ok(Vec::new());
        }
        
        info!("开始并发发送 {} 个文件到 {}", file_paths.len(), ipv6_addr);
        
        // 创建信号量限制并发数量
        let semaphore = Arc::new(Semaphore::new(CONCURRENT_TRANSFERS));
        let mut tasks = Vec::new();
        
        // 为每个文件创建异步任务
        for file_path in file_paths {
            let ip = ipv6_addr.to_string();
            let path = file_path.clone();
            let semaphore = semaphore.clone();
            
            let task = tokio::spawn(async move {
                // 在任务内部获取许可
                let _permit = semaphore.acquire().await;
                let result = Self::send_single_file(&ip, &path).await;
                (path, result)
            });
            
            tasks.push(task);
        }
        
        // 等待所有任务完成
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("任务执行失败: {}", e);
                    results.push(("unknown".to_string(), Err("任务执行失败".into())));
                }
            }
        }
        
        // 统计成功和失败的数量
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = results.len() - success_count;
        
        info!("并发发送完成: {} 成功, {} 失败", success_count, fail_count);
        
        Ok(results)
    }
    
    #[allow(dead_code)]
    pub async fn send_file(
        ipv6_addr: &str,
        file_path: &str,
    ) -> Result<(), SendError> {
        // 如果 IP 地址为空，默认使用本地地址 (::1)
        let actual_ip = if ipv6_addr.is_empty() {
            "::1"
        } else {
            ipv6_addr
        };
        
        // 解析IPv6地址和端口
        let socket_addr = format!("[{}]:6789", actual_ip);
        let addr: SocketAddrV6 = match socket_addr.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("解析 IPv6 地址失败: {} - 错误: {}", socket_addr, e);
                return Err(e.into());
            }
        };
        
        info!("正在连接到接收方: {}", addr);
        
        // 连接到接收方
        let mut stream = match TcpStream::connect(addr).await {
            Ok(s) => s,
            Err(e) => {
                error!("连接到接收方失败: {} - 错误: {}", addr, e);
                return Err(e.into());
            }
        };
        info!("已连接到接收方: {}", addr);
        
        // 进行 Noise 协议握手
        let mut transport = match Self::perform_noise_handshake(&mut stream).await {
            Ok(t) => t,
            Err(e) => {
                error!("Noise 协议握手失败: {}", e);
                return Err(e.into());
            }
        };
        
        // 获取自己的身份码
        let my_identity = match AddressBook::get_my_identity() {
            Ok(id) => id,
            Err(e) => {
                error!("获取身份码失败: {}", e);
                return Err(e.into());
            }
        };
        
        if my_identity.len() != 64 {
            error!("身份码长度不正确: {} (期望64字符)", my_identity.len());
            return Err("身份码长度不正确，必须为64字符".into());
        }
        
        info!("使用身份码: {}", my_identity);
        
        // 异步打开要发送的文件
        let mut file = match File::open(file_path).await {
            Ok(f) => f,
            Err(e) => {
                error!("打开文件失败: {} - 错误: {}", file_path, e);
                return Err(e.into());
            }
        };
        
        let file_name = match std::path::Path::new(file_path).file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                error!("无法从路径获取文件名: {}", file_path);
                return Err("无效的文件路径".into());
            }
        };
        
        // 获取文件大小
        let file_size = match file.metadata().await {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                error!("获取文件元数据失败: {} - 错误: {}", file_path, e);
                return Err(e.into());
            }
        };
        
        info!("开始发送文件: {} ({} 字节)", file_name, file_size);
        
        // 首先发送身份码 (64字符固定长度) - 使用加密通道
        if let Err(e) = Self::write_encrypted(&mut transport, &mut stream, my_identity.as_bytes()).await {
            error!("发送身份码失败: {}", e);
            return Err(e);
        }
        info!("已发送身份码");
        
        // 发送文件名长度和文件名 - 使用加密通道
        let file_name_bytes = file_name.as_bytes();
        let file_name_len = file_name_bytes.len() as u64;
        
        if let Err(e) = Self::write_encrypted(&mut transport, &mut stream, &file_name_len.to_be_bytes()).await {
            error!("发送文件名长度失败: {}", e);
            return Err(e);
        }
        
        if let Err(e) = Self::write_encrypted(&mut transport, &mut stream, file_name_bytes).await {
            error!("发送文件名失败: {}", e);
            return Err(e);
        }
        info!("已发送文件名: {}", file_name);
        
        // 发送文件大小 - 使用加密通道
        if let Err(e) = Self::write_encrypted(&mut transport, &mut stream, &file_size.to_be_bytes()).await {
            error!("发送文件大小失败: {}", e);
            return Err(e);
        }
        info!("已发送文件大小: {} 字节", file_size);
        
        // 使用缓冲区异步发送文件内容 - 使用加密通道
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB 缓冲区
        let mut total_sent = 0;
        
        loop {
            let bytes_read = match file.read(&mut buffer).await {
                Ok(read) => read,
                Err(e) => {
                    error!("读取文件内容失败: {} - 错误: {}", file_path, e);
                    return Err(e.into());
                }
            };
            
            if bytes_read == 0 {
                break;
            }
            
            // 使用加密通道发送数据
            if let Err(e) = Self::write_encrypted(&mut transport, &mut stream, &buffer[..bytes_read]).await {
                error!("发送文件数据失败: {}", e);
                return Err(e);
            }
            
            total_sent += bytes_read;
            
            // 每发送 1MB 打印一次进度，避免频繁打印
            if total_sent % (1024 * 1024) < 64 * 1024 || total_sent == file_size as usize {
                info!("已发送: {}/{} 字节 ({:.1}%)",
                         total_sent, file_size,
                         (total_sent as f64 / file_size as f64) * 100.0);
            }
        }
        
        // 确保所有数据都被刷新
        if let Err(e) = stream.flush().await {
            error!("刷新流数据失败: {}", e);
            return Err(e.into());
        }
        
        info!("文件数据发送完成，发送传输结束信号...");
        
        // 发送传输结束信号
        Self::send_transfer_complete(&mut stream).await?;
        
        info!("等待接收方的传输完成确认...");
        
        // 等待接收方的传输完成确认
        Self::wait_for_transfer_complete(&mut stream).await?;
        
        info!("文件传输完成: {}", file_name);
        Ok(())
    }
    
    pub async fn select_files() -> Result<Vec<String>, SendError> {
        // 使用 rfd 选择多个文件
        let file_handles = rfd::AsyncFileDialog::new()
            .set_title("选择要发送的文件（可多选）")
            .pick_files()
            .await;
        
        if let Some(files) = file_handles {
            let file_paths: Vec<String> = files
                .iter()
                .map(|file| file.path().to_string_lossy().to_string())
                .collect();
            
            info!("成功选择了 {} 个文件", file_paths.len());
            Ok(file_paths)
        } else {
            info!("用户取消了文件选择");
            Ok(Vec::new())
        }
    }
}