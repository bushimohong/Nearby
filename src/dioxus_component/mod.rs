mod send;
mod receive;
mod addressbook;
mod settings;
mod app;
// 新增app模块

// 导出页面组件
pub use send::send::Send;
pub use receive::receive::Receive;
pub use addressbook::addressbook::AddressBookPage;
pub use settings::settings::Settings;

// 导出App相关组件
pub use app::{App};