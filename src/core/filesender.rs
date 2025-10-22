// src/core/filesender.rs
use std::error;
use std::net::SocketAddrV6;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::core::db::AddressBook;
use log::{info, error, debug};

pub struct FileSender;

impl FileSender {
    // 执行 Noise 协议握手（作为发起者）
    async fn perform_noise_handshake(stream: &mut TcpStream) -> Result<snow::TransportState, Box<dyn error::Error>> {
        info!("开始 Noise 协议握手...");
        
        // 先解析 Noise 参数
        let noise_params = match "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse() {
            Ok(params) => {
                info!("成功解析 Noise 参数: Noise_XX_25519_ChaChaPoly_BLAKE2s");
                params
            },
            Err(e) => {
                error!("解析 Noise 参数失败: {}", e);
                return Err(Box::new(e));
            }
        };
        
        // 创建发起者
        let builder = snow::Builder::new(noise_params);
        let static_key = match builder.generate_keypair() {
            Ok(kp) => {
                info!("成功生成密钥对");
                kp.private
            },
            Err(e) => {
                error!("生成密钥对失败: {}", e);
                return Err(e.into());
            }
        };
        
        let mut noise = match builder
            .local_private_key(&static_key)
            .build_initiator() {
            Ok(n) => {
                info!("成功构建 Noise 发起者");
                n
            },
            Err(e) => {
                error!("构建 Noise 发起者失败: {}", e);
                return Err(e.into());
            }
        };
        
        // 发送第一条握手消息
        info!("准备发送第一条握手消息...");
        let mut handshake_buffer1 = vec![0u8; 65535];
        let len = match noise.write_message(&[], &mut handshake_buffer1) {
            Ok(l) => {
                debug!("第一条握手消息长度: {} 字节", l);
                l
            },
            Err(e) => {
                error!("写入第一条握手消息失败: {}", e);
                return Err(e.into());
            }
        };
        
        if let Err(e) = stream.write_u16(len as u16).await {
            error!("发送第一条握手消息长度失败: {}", e);
            return Err(e.into());
        }
        if let Err(e) = stream.write_all(&handshake_buffer1[..len]).await {
            error!("发送第一条握手消息内容失败: {}", e);
            return Err(e.into());
        }
        info!("成功发送第一条握手消息");
        
        // 接收响应消息
        info!("等待接收响应消息...");
        let len = match stream.read_u16().await {
            Ok(l) => {
                let length = l as usize;
                debug!("响应消息长度: {} 字节", length);
                length
            },
            Err(e) => {
                error!("读取响应消息长度失败: {}", e);
                return Err(e.into());
            }
        };
        
        let mut msg = vec![0u8; len];
        if let Err(e) = stream.read_exact(&mut msg).await {
            error!("读取响应消息内容失败: {}", e);
            return Err(e.into());
        }
        info!("成功接收响应消息");
        
        // 读取响应消息
        let mut handshake_buffer2 = vec![0u8; 65535];
        if let Err(e) = noise.read_message(&msg, &mut handshake_buffer2) {
            error!("处理响应消息失败: {}", e);
            return Err(e.into());
        }
        info!("成功处理响应消息");
        
        // 发送第三条握手消息
        info!("准备发送第三条握手消息...");
        let mut handshake_buffer3 = vec![0u8; 65535];
        let len = match noise.write_message(&[], &mut handshake_buffer3) {
            Ok(l) => {
                debug!("第三条握手消息长度: {} 字节", l);
                l
            },
            Err(e) => {
                error!("写入第三条握手消息失败: {}", e);
                return Err(e.into());
            }
        };
        
        if let Err(e) = stream.write_u16(len as u16).await {
            error!("发送第三条握手消息长度失败: {}", e);
            return Err(e.into());
        }
        if let Err(e) = stream.write_all(&handshake_buffer3[..len]).await {
            error!("发送第三条握手消息内容失败: {}", e);
            return Err(e.into());
        }
        info!("成功发送第三条握手消息");
        
        // 转换为传输模式
        let transport = match noise.into_transport_mode() {
            Ok(t) => {
                info!("成功转换为传输模式");
                t
            },
            Err(e) => {
                error!("转换为传输模式失败: {}", e);
                return Err(e.into());
            }
        };
        
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
    async fn write_encrypted(transport: &mut snow::TransportState, stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn error::Error>> {
        let mut buffer = vec![0u8; 65535];
        
        // 加密数据
        let len = match transport.write_message(data, &mut buffer) {
            Ok(l) => l,
            Err(e) => {
                error!("加密数据失败: {}", e);
                return Err(e.into());
            }
        };
        
        // 发送加密数据的长度和数据
        if let Err(e) = stream.write_u16(len as u16).await {
            error!("发送加密数据长度失败: {}", e);
            return Err(e.into());
        }
        if let Err(e) = stream.write_all(&buffer[..len]).await {
            error!("发送加密数据内容失败: {}", e);
            return Err(e.into());
        }
        
        Ok(())
    }
    
    // 等待传输完成确认
    async fn wait_for_transfer_complete(stream: &mut TcpStream) -> Result<(), Box<dyn error::Error>> {
        info!("等待接收方的传输完成确认...");
        
        // 读取结束信号（长度为0的数据包）
        let end_signal = match stream.read_u16().await {
            Ok(signal) => signal,
            Err(e) => {
                error!("读取传输结束信号失败: {}", e);
                return Err(e.into());
            }
        };
        
        if end_signal == 0 {
            info!("接收方已确认传输完成");
            Ok(())
        } else {
            error!("无效的传输结束信号: {}", end_signal);
            Err("无效的传输结束信号".into())
        }
    }
    
    // 发送传输结束信号
    async fn send_transfer_complete(stream: &mut TcpStream) -> Result<(), Box<dyn error::Error>> {
        info!("发送传输结束信号...");
        
        // 发送长度为0的数据包表示传输结束
        if let Err(e) = stream.write_u16(0).await {
            error!("发送传输结束信号失败: {}", e);
            return Err(e.into());
        }
        if let Err(e) = stream.flush().await {
            error!("刷新传输结束信号失败: {}", e);
            return Err(e.into());
        }
        
        info!("传输结束信号已发送");
        Ok(())
    }
    
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
    
    pub async fn select_files() -> Result<Vec<String>, Box<dyn error::Error>> {
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