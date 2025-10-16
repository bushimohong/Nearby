// src/core/receive.rs
use std::net::SocketAddrV6;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, Semaphore};
use std::sync::Mutex;

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
    // 设置接收状态
    pub fn set_receive_status(status: ReceiveStatus) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_status = RECEIVE_STATUS.lock().unwrap();
        *current_status = status;
        
        match &*current_status {
            ReceiveStatus::Closed => {
                println!("接收功能已关闭");
                // 如果服务器正在运行，停止它
                if SERVER_RUNNING.load(Ordering::SeqCst) {
                    tokio::spawn(async {
                        if let Err(e) = Self::stop_server().await {
                            eprintln!("停止服务器时出错: {}", e);
                        }
                    });
                }
            }
            ReceiveStatus::Open | ReceiveStatus::Collect => {
                println!("接收功能已开启 - 接收所有文件");
                // 如果服务器未运行，启动它
                if !SERVER_RUNNING.load(Ordering::SeqCst) {
                    tokio::spawn(async {
                        if let Err(e) = Self::start_server().await {
                            eprintln!("启动服务器时出错: {}", e);
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
    
    pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态，如果是关闭状态则不启动服务器
        let status = Self::get_receive_status();
        if status == ReceiveStatus::Closed {
            println!("接收功能已关闭，不启动服务器");
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
        
        println!("文件接收服务器启动，监听在: {}", addr);
        println!("当前接收模式: {:?}", status);
        println!("等待连接... (按停止按钮可关闭服务器)");
        
        // 使用 tokio::select! 来同时监听连接和停止信号
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            println!("接收到来自 {} 的连接", peer_addr);
                            
                            // 在处理连接前再次检查状态
                            let current_status = Self::get_receive_status();
                            if current_status == ReceiveStatus::Closed {
                                println!("接收功能已关闭，拒绝连接");
                                continue;
                            }

                            // 获取连接许可，如果达到最大连接数会等待
                            let connections = ACTIVE_CONNECTIONS.get().unwrap().clone();
                            let permit = connections.acquire_owned().await;

                            // 为每个连接生成一个异步任务
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client(stream, current_status).await {
                                    eprintln!("处理客户端时出错: {}", e);
                                }
                                // permit 在这里被 drop，释放连接计数
                                drop(permit);
                            });
                        }
                        Err(e) => {
                            eprintln!("接受连接时出错: {}", e);
                        }
                    }
                }
                _ = stop_notify.notified() => {
                    println!("收到停止信号，关闭接收服务器...");
                    break;
                }
            }
        }
        
        // 重置运行标志
        SERVER_RUNNING.store(false, Ordering::SeqCst);
        println!("接收服务器已安全关闭");
        Ok(())
    }
    
    pub async fn stop_server() -> Result<(), Box<dyn std::error::Error>> {
        if SERVER_RUNNING.load(Ordering::SeqCst) {
            if let Some(notify) = STOP_NOTIFY.get() {
                notify.notify_one();
                // 等待一小段时间让服务器完全关闭
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }
    
    async fn handle_client(mut stream: TcpStream, current_status: ReceiveStatus) -> Result<(), Box<dyn std::error::Error>> {
        // 首先接收身份标识（64字符固定长度）
        let mut identity_bytes = vec![0u8; 64];
        stream.read_exact(&mut identity_bytes).await?;
        let identity = String::from_utf8(identity_bytes)?;
        
        println!("接收到身份标识: {}", identity);
        
        // 根据当前状态决定是否接收文件
        match current_status {
            ReceiveStatus::Closed => {
                println!("接收功能已关闭，拒绝接收文件");
                return Ok(());
            }
            ReceiveStatus::Open => {
                println!("开启模式，接收所有文件");
                // 继续处理文件接收
            }
            ReceiveStatus::Collect => {
                // 检查身份是否在白名单中
                if !Self::check_identity_in_whitelist(&identity).await {
                    println!("身份 {} 不在白名单中，拒绝接收文件", identity);
                    return Ok(());
                }
                println!("身份 {} 在白名单中，允许接收文件", identity);
            }
        }
        
        // 接收文件名长度
        let file_name_len = stream.read_u64().await?;
        
        // 接收文件名
        let mut file_name_bytes = vec![0u8; file_name_len as usize];
        stream.read_exact(&mut file_name_bytes).await?;
        let file_name = String::from_utf8(file_name_bytes)?;
        
        println!("接收文件: {}", file_name);
        
        // 接收文件大小
        let file_size = stream.read_u64().await?;
        println!("文件大小: {} 字节", file_size);
        
        // 创建 downloads 目录
        let downloads_dir = Self::get_downloads_dir().await?;
        tokio::fs::create_dir_all(&downloads_dir).await?;
        
        // 构建保存路径
        let save_path = downloads_dir.join(&file_name);
        
        // 处理文件名冲突
        let final_save_path = Self::get_unique_filename(save_path).await;
        
        println!("保存文件到: {}", final_save_path.display());
        
        // 创建文件
        let mut file = File::create(&final_save_path).await?;
        
        // 使用缓冲区异步接收文件内容
        let mut received = 0;
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB 缓冲区
        
        while received < file_size {
            let bytes_to_read = std::cmp::min(buffer.len() as u64, file_size - received) as usize;
            let bytes_read = stream.read(&mut buffer[..bytes_to_read]).await?;
            
            if bytes_read == 0 {
                break;
            }
            
            // 异步写入文件
            file.write_all(&buffer[..bytes_read]).await?;
            received += bytes_read as u64;
            
            // 每接收 1MB 打印一次进度，避免频繁打印
            if received % (1024 * 1024) < 64 * 1024 || received == file_size {
                println!("已接收: {}/{} 字节 ({:.1}%)",
                         received, file_size,
                         (received as f64 / file_size as f64) * 100.0);
            }
        }
        
        println!("文件接收完成: {}", final_save_path.display());
        Ok(())
    }
    
    /// 获取 downloads 目录路径
    async fn get_downloads_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 首先尝试获取用户目录下的 Downloads
        if let Some(mut downloads_dir) = dirs::download_dir() {
            downloads_dir.push("dioxus_file_transfer");
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