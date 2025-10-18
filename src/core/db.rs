// src/core/db.rs
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use dirs::data_dir;
use crate::core::create_identity::CreateIdentity;
use log::{info, debug}; // 添加日志功能

pub struct AddressBook;

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntry {
    pub id: i64,           // 主键ID
    pub identity: String,  // 身份标识 (64个字符)
    pub alias: String,     // 备注
}

#[derive(Debug, Clone, PartialEq)]
pub struct FriendEntry {
    pub id: i64,          // 主键ID
    pub address: String,  // IPv6地址
    pub alias: String,    // 备注
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileReceiveRecord {
    pub id: i64,                // 主键ID
    pub filename: String,       // 文件名
    pub file_size: u64,         // 文件大小（字节）
    pub sender_ipv6: String,    // 发送方IPv6地址
    pub sender_identity: String, // 发送方身份标识
    pub received_at: String,    // 接收时间
    pub save_path: String,      // 保存路径
}

impl AddressBook {
    /// 获取数据库连接
    fn get_connection() -> Result<Connection> {
        let db_path = Self::get_db_path();
        Connection::open(db_path)
    }
    
    /// 获取数据库文件路径
    pub fn get_db_path() -> PathBuf {
        if let Some(mut data_dir) = data_dir() {
            data_dir.push("Nearby");
            std::fs::create_dir_all(&data_dir).ok();
            data_dir.push("address_book.db");
            data_dir
        } else {
            PathBuf::from("address_book.db")
        }
    }
    
    /// 初始化数据库表
    pub fn init_db() -> Result<()> {
        let conn = Self::get_connection()?;
        
        // 创建身份标识表（存储身份标识）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS identities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                identity TEXT NOT NULL UNIQUE,
                alias TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 创建好友列表表（存储IPv6地址）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS friends (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL UNIQUE,
                alias TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 创建我的身份码表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS my_identity (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                identity TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 创建文件接收记录表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_receive_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                sender_ipv6 TEXT NOT NULL,
                sender_identity TEXT NOT NULL,
                save_path TEXT NOT NULL,
                received_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 验证表结构
        Self::verify_table_structure()?;
        
        // 确保存在我的身份码
        Self::ensure_my_identity()?;
        
        info!("数据库初始化完成");
        Ok(())
    }
    
    fn check_table_has_column(conn: &Connection, table_name: &str, column_name: &str, table_desc: &str) -> Result<()> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns: Vec<String> = stmt
            .query_map([], |row| {
                Ok(row.get::<_, String>(1)?) // 获取列名
            })?
            .collect::<Result<Vec<String>>>()?;
        
        debug!("{}列: {:?}", table_desc, columns);
        
        if !columns.contains(&column_name.to_string()) {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("{}缺少 {} 列", table_desc, column_name)
            ));
        }
        
        Ok(())
    }
    
    /// 验证表结构是否正确
    fn verify_table_structure() -> Result<()> {
        let conn = Self::get_connection()?;
        
        // 检查 identities 表结构
        Self::check_table_has_column(&conn, "identities", "identity", "身份标识表")?;
        
        // 检查 friends 表结构
        Self::check_table_has_column(&conn, "friends", "address", "好友表")?;
        
        // 检查文件接收记录表结构
        Self::check_table_has_column(&conn, "file_receive_records", "filename", "文件接收记录表")?;
        Self::check_table_has_column(&conn, "file_receive_records", "file_size", "文件接收记录表")?;
        Self::check_table_has_column(&conn, "file_receive_records", "sender_ipv6", "文件接收记录表")?;
        Self::check_table_has_column(&conn, "file_receive_records", "sender_identity", "文件接收记录表")?;
        Self::check_table_has_column(&conn, "file_receive_records", "save_path", "文件接收记录表")?;
        
        Ok(())
    }
    
    /// 确保存在我的身份码
    fn ensure_my_identity() -> Result<()> {
        let conn = Self::get_connection()?;
        
        // 检查是否已存在身份码
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM my_identity WHERE id = 1")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        if count == 0 {
            // 生成新的身份码
            let new_identity = CreateIdentity::new();
            let identity_str: String = new_identity.iter().collect();
            
            // 插入新的身份码
            conn.execute(
                "INSERT INTO my_identity (id, identity) VALUES (1, ?1)",
                &[&identity_str],
            )?;
            
            info!("已生成新的身份码: {}", identity_str);
        }
        
        Ok(())
    }
    
    // ===== 我的身份码操作 =====
    
    /// 获取我的身份码
    pub fn get_my_identity() -> Result<String> {
        let conn = Self::get_connection()?;
        let mut stmt = conn.prepare("SELECT identity FROM my_identity WHERE id = 1")?;
        let identity: String = stmt.query_row([], |row| row.get(0))?;
        Ok(identity)
    }
    
    /// 重置我的身份码
    pub fn reset_my_identity() -> Result<String> {
        let conn = Self::get_connection()?;
        
        // 生成新的身份码
        let new_identity = CreateIdentity::new();
        let identity_str: String = new_identity.iter().collect();
        
        // 更新身份码
        conn.execute(
            "UPDATE my_identity SET identity = ?1, created_at = CURRENT_TIMESTAMP WHERE id = 1",
            &[&identity_str],
        )?;
        
        info!("已重置身份码: {}", identity_str);
        Ok(identity_str)
    }
    
    // ===== 身份标识表操作 =====
    
    /// 添加身份标识
    pub fn add_identity(identity: &str, alias: &str) -> Result<()> {
        if identity.len() != 64 {
            return Err(rusqlite::Error::InvalidParameterName("身份标识必须为64字符".to_string()));
        }
        
        let conn = Self::get_connection()?;
        conn.execute(
            "INSERT INTO identities (identity, alias) VALUES (?1, ?2)",
            &[identity, alias],
        )?;
        info!("已添加身份标识: {} ({})", alias, identity);
        Ok(())
    }
    
    /// 更新身份标识
    pub fn update_identity(id: i64, identity: &str, alias: &str) -> Result<()> {
        if identity.len() != 64 {
            return Err(rusqlite::Error::InvalidParameterName("身份标识必须为64字符".to_string()));
        }
        
        let conn = Self::get_connection()?;
        conn.execute(
            "UPDATE identities SET identity = ?1, alias = ?2 WHERE id = ?3",
            &[identity, alias, &id.to_string()],
        )?;
        info!("已更新身份标识 ID {}: {} ({})", id, alias, identity);
        Ok(())
    }
    
    /// 删除身份标识
    pub fn delete_identity(id: i64) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute("DELETE FROM identities WHERE id = ?1", [id])?;
        info!("已删除身份标识 ID: {}", id);
        Ok(())
    }
    
    /// 获取所有身份标识
    pub fn get_all_identities() -> Result<Vec<IdentityEntry>> {
        let conn = Self::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, identity, alias FROM identities ORDER BY created_at DESC")?;
        let entries = stmt.query_map([], |row| {
            Ok(IdentityEntry {
                id: row.get(0)?,
                identity: row.get(1)?,
                alias: row.get(2)?,
            })
        })?;
        
        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }
        Ok(result)
    }
    
    // ===== 好友列表表操作 =====
    
    /// 添加IPv6地址到好友列表
    pub fn add_friend(address: &str, alias: &str) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute(
            "INSERT INTO friends (address, alias) VALUES (?1, ?2)",
            &[address, alias],
        )?;
        info!("已添加好友: {} ({})", alias, address);
        Ok(())
    }
    
    /// 更新好友信息
    pub fn update_friend(id: i64, address: &str, alias: &str) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute(
            "UPDATE friends SET address = ?1, alias = ?2 WHERE id = ?3",
            &[address, alias, &id.to_string()],
        )?;
        info!("已更新好友 ID {}: {} ({})", id, alias, address);
        Ok(())
    }
    
    /// 从好友列表删除地址
    pub fn delete_friend(id: i64) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute("DELETE FROM friends WHERE id = ?1", [id])?;
        info!("已删除好友 ID: {}", id);
        Ok(())
    }
    
    /// 获取所有好友地址
    pub fn get_all_friends() -> Result<Vec<FriendEntry>> {
        let conn = Self::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, address, alias FROM friends ORDER BY created_at DESC")?;
        let entries = stmt.query_map([], |row| {
            Ok(FriendEntry {
                id: row.get(0)?,
                address: row.get(1)?,
                alias: row.get(2)?,
            })
        })?;
        
        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }
        Ok(result)
    }
    
    // 搜索好友
    pub fn search_friends(query: &str) -> Result<Vec<FriendEntry>> {
        let conn = Self::get_connection()?;
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, address, alias FROM friends
             WHERE alias LIKE ?1 OR address LIKE ?2
             ORDER BY created_at DESC"
        )?;
        
        let entries = stmt.query_map([&search_pattern, &search_pattern], |row| {
            Ok(FriendEntry {
                id: row.get(0)?,
                address: row.get(1)?,
                alias: row.get(2)?,
            })
        })?;
        
        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }
        Ok(result)
    }
    
    // 搜索身份标识
    pub fn search_identities(query: &str) -> Result<Vec<IdentityEntry>> {
        let conn = Self::get_connection()?;
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, identity, alias FROM identities
             WHERE alias LIKE ?1 OR identity LIKE ?2
             ORDER BY created_at DESC"
        )?;
        
        let entries = stmt.query_map([&search_pattern, &search_pattern], |row| {
            Ok(IdentityEntry {
                id: row.get(0)?,
                identity: row.get(1)?,
                alias: row.get(2)?,
            })
        })?;
        
        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }
        Ok(result)
    }
    
    // ===== 文件接收记录操作 =====
    
    fn query_file_records(
        sql: &str,
        params: &[&dyn rusqlite::ToSql]
    ) -> Result<Vec<FileReceiveRecord>> {
        let conn = Self::get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        
        let entries = stmt.query_map(params, |row| {
            Ok(FileReceiveRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
                file_size: row.get(2)?,
                sender_ipv6: row.get(3)?,
                sender_identity: row.get(4)?,
                save_path: row.get(5)?,
                received_at: row.get(6)?,
            })
        })?;
        
        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }
        Ok(result)
    }
    
    /// 添加文件接收记录
    pub fn add_file_receive_record(
        filename: &str,
        file_size: u64,
        sender_ipv6: &str,
        sender_identity: &str,
        save_path: &str,
    ) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute(
            "INSERT INTO file_receive_records (filename, file_size, sender_ipv6, sender_identity, save_path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            &[filename, &file_size.to_string(), sender_ipv6, sender_identity, save_path],
        )?;
        
        info!("已记录文件接收: {} ({} 字节) 来自 {}",
              filename, file_size, sender_ipv6);
        Ok(())
    }
    
    /// 获取所有文件接收记录
    pub fn get_all_file_receive_records() -> Result<Vec<FileReceiveRecord>> {
        Self::query_file_records(
            "SELECT id, filename, file_size, sender_ipv6, sender_identity, save_path, received_at
            FROM file_receive_records
            ORDER BY received_at DESC",
            &[]
        )
    }
    
    /// 删除文件接收记录
    pub fn delete_file_receive_record(id: i64) -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute("DELETE FROM file_receive_records WHERE id = ?1", [id])?;
        info!("已删除文件接收记录 ID: {}", id);
        Ok(())
    }
    
    /// 清除所有记录
    pub fn delete_all_file_receive_records() -> Result<()> {
        let conn = Self::get_connection()?;
        conn.execute("DELETE FROM file_receive_records", [])?;
        info!("已清除所有文件接收记录");
        Ok(())
    }
}