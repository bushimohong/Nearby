// src/core/filereceiver.rs
use std::error;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, Semaphore};
use std::sync::Mutex;
use pnet::datalink;
use log::{info, error, warn};

pub struct FileReceiver;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static STOP_NOTIFY: tokio::sync::OnceCell<Arc<Notify>> = tokio::sync::OnceCell::const_new();
static ACTIVE_CONNECTIONS: tokio::sync::OnceCell<Arc<Semaphore>> = tokio::sync::OnceCell::const_new();

// 添加接收状态管理
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiveStatus {
    Closed,    // 关闭状态 - 不接收任何文件
    Open,      // 开启状态 - 接收所有文件
    Collect    // 收藏状态 - 只接收白名单中的文件
}

static RECEIVE_STATUS: Mutex<ReceiveStatus> = Mutex::new(ReceiveStatus::Closed);

impl FileReceiver {
    // 返回ipv6地址
    pub fn get_ipv6_addr() -> Vec<Ipv6Addr> {
        datalink::interfaces()
            .iter()
            .filter(|iface| iface.is_up() && !iface.is_loopback())
            .flat_map(|iface| &iface.ips)
            .filter_map(|ip_network| {
                if let pnet::ipnetwork::IpNetwork::V6(ipv6_network) = ip_network {
                    let ip = ipv6_network.ip();
                    // 过滤掉链路本地地址和其他特殊地址
                    if !is_special_ipv6_address(ip) {
                        Some(ip)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 设置接收状态
    pub fn set_receive_status(status: ReceiveStatus) -> Result<(), Box<dyn error::Error>> {
        let mut current_status = RECEIVE_STATUS.lock().unwrap();
        *current_status = status;
        
        match &*current_status {
            ReceiveStatus::Closed => {
                info!("接收功能已关闭");
                // 如果服务器正在运行，停止它
                if SERVER_RUNNING.load(Ordering::SeqCst) {
                    tokio::spawn(async {
                        if let Err(e) = Self::stop_server().await {
                            error!("停止服务器时出错: {}", e);
                        }
                    });
                }
            }
            ReceiveStatus::Open | ReceiveStatus::Collect => {
                info!("接收功能已开启 - 接收所有文件");
                // 如果服务器未运行，启动它
                if !SERVER_RUNNING.load(Ordering::SeqCst) {
                    tokio::spawn(async {
                        if let Err(e) = Self::start_server().await {
                            error!("启动服务器时出错: {}", e);
                        }
                    });
                }
            }
        }
        
        Ok(())
    }
    
    // 获取当前接收状态
    pub fn get_receive_status() -> ReceiveStatus {
        let status = RECEIVE_STATUS.lock().unwrap();
        status.clone()
    }
    
    // 检查身份是否在白名单中
    async fn check_identity_in_whitelist(identity: &str) -> bool {
        // 使用 db.rs 的 search_identities 验证身份码是否存在
        match crate::core::db::AddressBook::search_identities(identity).ok() {
            Some(identities) => {
                // 如果找到匹配的身份码，返回 true
                !identities.is_empty()
            }
            None => false
        }
    }
    
    // 执行 Noise 协议握手（作为响应者）
    async fn perform_noise_handshake(stream: &mut TcpStream) -> Result<snow::TransportState, Box<dyn error::Error>> {
        info!("开始 Noise 协议握手...");
        
        // 创建响应者 - 使用正确的 API
        let builder = snow::Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?);
        let static_key = builder.generate_keypair()?.private;
        let mut noise = builder
            .local_private_key(&static_key)
            .build_responder()?;
        
        // 接收第一条消息
        let len = stream.read_u16().await? as usize;
        let mut msg = vec![0u8; len];
        stream.read_exact(&mut msg).await?;
        
        // 读取并消费第一条消息
        let mut handshake_buffer1 = vec![0u8; 65535];
        let _ = noise.read_message(&msg, &mut handshake_buffer1)?;
        
        // 发送响应消息
        let mut handshake_buffer2 = vec![0u8; 65535];
        let len = noise.write_message(&[], &mut handshake_buffer2)?;
        stream.write_u16(len as u16).await?;
        stream.write_all(&handshake_buffer2[..len]).await?;
        
        // 接收第三条消息
        let len = stream.read_u16().await? as usize;
        let mut msg = vec![0u8; len];
        stream.read_exact(&mut msg).await?;
        
        // 读取第三条消息完成握手
        let mut handshake_buffer3 = vec![0u8; 65535];
        let _ = noise.read_message(&msg, &mut handshake_buffer3)?;
        
        // 转换为传输模式
        let transport = noise.into_transport_mode()?;
        
        info!("Noise 协议握手完成");
        Ok(transport)
    }
    
    // 使用加密通道读取数据
    async fn read_encrypted(transport: &mut snow::TransportState, stream: &mut TcpStream, buffer: &mut [u8]) -> Result<usize, Box<dyn error::Error>> {
        // 先读取加密数据的长度
        let encrypted_len = stream.read_u16().await? as usize;
        if encrypted_len == 0 {
            // 长度为0表示传输结束
            return Ok(0);
        }
        if encrypted_len > 65535 {
            error!("错误的加密数据长度");
            return Err("无效的加密数据长度".into());
        }
        let mut encrypted_data = vec![0u8; encrypted_len];
        stream.read_exact(&mut encrypted_data).await?;
        
        // 解密数据
        let len = transport.read_message(&encrypted_data, buffer)?;
        Ok(len)
    }
    
    // 使用加密通道写入数据
    #[allow(dead_code)]
    async fn write_encrypted(transport: &mut snow::TransportState, stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn error::Error>> {
        let mut buffer = vec![0u8; 65535];
        
        // 加密数据
        let len = transport.write_message(data, &mut buffer)?;
        if len == 0 || len > 65535 {
            error!("错误的加密数据长度");
            return Err("无效的加密数据长度".into());
        }
        
        // 发送加密数据的长度和数据
        stream.write_u16(len as u16).await?;
        stream.write_all(&buffer[..len]).await?;
        
        Ok(())
    }
    
    // 发送传输结束信号
    async fn send_transfer_complete(stream: &mut TcpStream) -> Result<(), Box<dyn error::Error>> {
        // 发送长度为0的数据包表示传输结束
        stream.write_u16(0).await?;
        stream.flush().await?;
        Ok(())
    }
    
    pub async fn start_server() -> Result<(), Box<dyn error::Error>> {
        // 检查当前状态，如果是关闭状态则不启动服务器
        let status = Self::get_receive_status();
        if status == ReceiveStatus::Closed {
            info!("接收功能已关闭，不启动服务器");
            return Ok(());
        }
        
        // 设置服务器运行标志
        SERVER_RUNNING.store(true, Ordering::SeqCst);
        
        // 初始化信号量，限制最大并发连接数
        let _ = ACTIVE_CONNECTIONS.set(Arc::new(Semaphore::new(10))); // 最多10个并发连接
        
        // 创建停止通知
        let stop_notify = Arc::new(Notify::new());
        let _ = STOP_NOTIFY.set(stop_notify.clone());
        
        // 绑定到所有IPv6地址的6789端口
        let addr = SocketAddrV6::new("::".parse()?, 6789, 0, 0);
        let listener = TcpListener::bind(addr).await?;
        
        info!("文件接收服务器启动，监听在: {}", addr);
        info!("当前接收模式: {:?}", status);
        info!("使用 Noise 协议加密传输");
        info!("等待连接... (按停止按钮可关闭服务器)");
        
        // 使用 tokio::select! 来同时监听连接和停止信号
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            info!("接收到来自 {} 的连接", peer_addr);
                            
                            // 在处理连接前再次检查状态
                            let current_status = Self::get_receive_status();
                            if current_status == ReceiveStatus::Closed {
                                warn!("接收功能已关闭，拒绝连接");
                                continue;
                            }

                            // 获取连接许可，如果达到最大连接数会等待
                            let connections = ACTIVE_CONNECTIONS.get().unwrap().clone();
                            let permit = connections.acquire_owned().await;

                            // 为每个连接生成一个异步任务
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client(stream, current_status, peer_addr).await {
                                    error!("处理客户端时出错: {}", e);
                                }
                                // permit 在这里被 drop，释放连接计数
                                drop(permit);
                            });
                        }
                        Err(e) => {
                            error!("接受连接时出错: {}", e);
                        }
                    }
                }
                _ = stop_notify.notified() => {
                    info!("收到停止信号，关闭接收服务器...");
                    break;
                }
            }
        }
        
        // 重置运行标志
        SERVER_RUNNING.store(false, Ordering::SeqCst);
        info!("接收服务器已安全关闭");
        Ok(())
    }
    
    pub async fn stop_server() -> Result<(), Box<dyn error::Error>> {
        if SERVER_RUNNING.load(Ordering::SeqCst) {
            if let Some(notify) = STOP_NOTIFY.get() {
                notify.notify_one();
                // 等待一小段时间让服务器完全关闭
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }
    
    async fn handle_client(
        mut stream: TcpStream,
        current_status: ReceiveStatus,
        peer_addr: SocketAddr
    ) -> Result<(), Box<dyn error::Error>> {
        // 首先进行 Noise 协议握手
        let mut transport = Self::perform_noise_handshake(&mut stream).await?;
        
        // 接收身份标识（64字符固定长度）
        let mut identity_bytes = vec![0u8; 64];
        let identity_len = Self::read_encrypted(&mut transport, &mut stream, &mut identity_bytes).await?;
        if identity_len == 0 {
            info!("接收到传输结束信号，连接正常关闭");
            return Ok(());
        }
        let identity = String::from_utf8(identity_bytes)?;
        
        info!("接收到身份标识: {}", identity);
        
        // 根据当前状态决定是否接收文件
        match current_status {
            ReceiveStatus::Closed => {
                info!("接收功能已关闭，拒绝接收文件");
                // 发送拒绝信号
                Self::send_transfer_complete(&mut stream).await?;
                return Ok(());
            }
            ReceiveStatus::Open => {
                info!("开启模式，接收所有文件");
                // 继续处理文件接收
            }
            ReceiveStatus::Collect => {
                // 检查身份是否在白名单中
                if !Self::check_identity_in_whitelist(&identity).await {
                    warn!("身份 {} 不在白名单中，拒绝接收文件", identity);
                    // 发送拒绝信号
                    Self::send_transfer_complete(&mut stream).await?;
                    return Ok(());
                }
                info!("身份 {} 在白名单中，允许接收文件", identity);
            }
        }
        
        // 接收文件名长度
        let mut file_name_len_bytes = vec![0u8; 8];
        let file_name_len_size = Self::read_encrypted(&mut transport, &mut stream, &mut file_name_len_bytes).await?;
        if file_name_len_size == 0 {
            info!("接收到传输结束信号，连接正常关闭");
            return Ok(());
        }
        let file_name_len = u64::from_be_bytes(file_name_len_bytes.try_into().unwrap());
        
        // 接收文件名
        let mut file_name_bytes = vec![0u8; file_name_len as usize];
        let file_name_size = Self::read_encrypted(&mut transport, &mut stream, &mut file_name_bytes).await?;
        if file_name_size == 0 {
            info!("接收到传输结束信号，连接正常关闭");
            return Ok(());
        }
        let file_name = String::from_utf8(file_name_bytes)?;
        
        info!("接收文件: {}", file_name);
        
        // 接收文件大小
        let mut file_size_bytes = vec![0u8; 8];
        let file_size_size = Self::read_encrypted(&mut transport, &mut stream, &mut file_size_bytes).await?;
        if file_size_size == 0 {
            info!("接收到传输结束信号，连接正常关闭");
            return Ok(());
        }
        let file_size = u64::from_be_bytes(file_size_bytes.try_into().unwrap());
        
        info!("文件大小: {} 字节", file_size);
        
        // 创建 downloads 目录
        let downloads_dir = Self::get_downloads_dir().await?;
        tokio::fs::create_dir_all(&downloads_dir).await?;
        
        // 构建保存路径
        let save_path = downloads_dir.join(&file_name);
        
        // 处理文件名冲突
        let final_save_path = Self::get_unique_filename(save_path).await;
        
        info!("保存文件到: {}", final_save_path.display());
        
        // 创建文件
        let mut file = File::create(&final_save_path).await?;
        
        // 使用缓冲区异步接收文件内容
        let mut received = 0;
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB 缓冲区
        
        while received < file_size {
            let bytes_to_read = std::cmp::min(buffer.len() as u64, file_size - received) as usize;
            let bytes_read = Self::read_encrypted(&mut transport, &mut stream, &mut buffer[..bytes_to_read]).await?;
            
            if bytes_read == 0 {
                // 传输结束信号
                if received == file_size {
                    info!("文件传输正常结束");
                    break;
                } else {
                    error!("文件传输中断: 已接收 {}/{} 字节", received, file_size);
                    return Err("文件传输中断".into());
                }
            }
            
            // 异步写入文件
            file.write_all(&buffer[..bytes_read]).await?;
            received += bytes_read as u64;
            
            // 每接收 1MB 打印一次进度，避免频繁打印
            if received % (1024 * 1024) < 64 * 1024 || received == file_size {
                let progress = (received as f64 / file_size as f64) * 100.0;
                info!("已接收: {}/{} 字节 ({:.1}%)", received, file_size, progress);
            }
        }
        
        info!("文件接收完成: {}", final_save_path.display());
        
        // 发送传输完成确认
        Self::send_transfer_complete(&mut stream).await?;
        
        if let Err(e) = crate::core::db::AddressBook::add_file_receive_record(
            &file_name,
            file_size,
            &peer_addr.ip().to_string(),
            &identity,
            &final_save_path.to_string_lossy(),
        ) {
            error!("记录文件接收信息失败: {}", e);
        } else {
            info!("文件接收记录已保存到数据库");
        }
        
        Ok(())
    }
    
    /// 获取 downloads 目录路径
    async fn get_downloads_dir() -> Result<PathBuf, Box<dyn error::Error>> {
        // 首先尝试获取用户目录下的 Downloads
        if let Some(mut downloads_dir) = dirs::download_dir() {
            downloads_dir.push("Nearby-receive");
            return Ok(downloads_dir);
        }
        
        // 如果无法获取系统 Downloads 目录，使用当前目录下的 downloads 文件夹
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join("downloads"))
    }
    
    /// 处理文件名冲突，如果文件已存在则添加数字后缀
    async fn get_unique_filename(mut path: PathBuf) -> PathBuf {
        let original_path = path.clone();
        let mut counter = 1;
        
        // 检查文件是否已存在
        while path.exists() {
            let stem = original_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let extension = original_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            
            let new_filename = if extension.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, extension)
            };
            
            path = original_path.with_file_name(new_filename);
            counter += 1;
        }
        
        path
    }
}

/// 判断是否为特殊IPv6地址（链路本地、多播等）
fn is_special_ipv6_address(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    // 链路本地地址 (fe80::/10)
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    // 多播地址 (ff00::/8)
    if segments[0] & 0xff00 == 0xff00 {
        return true;
    }
    // 唯一本地地址 (fc00::/7)
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }
    false
}