# Development

此软件是基于ipv6地址,TCP连接的文件传输工具

### 项目结构:

```
mohong@mohongdeMacBook-Air Nearby % tree -I assets -I target 
.
├── Cargo.lock
├── Cargo.toml
├── Dioxus.toml
├── LICENSE
├── README.md
├── clippy.toml
└── src
    ├── core
    │   ├── create_identity.rs
    │   ├── db.rs
    │   ├── filereceiver.rs
    │   ├── filesender.rs
    │   └── mod.rs
    ├── dioxus_component
    │   ├── addressbook
    │   │   ├── add_member.rs
    │   │   ├── addressbook.rs
    │   │   ├── friends.rs
    │   │   ├── mod.rs
    │   │   └── whitelist.rs
    │   ├── app.rs
    │   ├── mod.rs
    │   ├── receivefile
    │   │   ├── mod.rs
    │   │   └── receivefile.rs
    │   ├── send
    │   │   ├── mod.rs
    │   │   └── send.rs
    │   └── settings
    │       ├── mod.rs
    │       └── settings.rs
    └── main.rs

8 directories, 25 files
```

### 运行软件

在项目根目录下运行此命令

```bash
cargo run
```



