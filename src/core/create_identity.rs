// src/core/create_identity.rs
use rand::prelude::SliceRandom;

pub struct CreateIdentity {}

impl CreateIdentity {
	pub fn new() -> [char; 64] {
		const PRINTABLE_ASCII: &[char] = &[
			'!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
			'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
			':', ';', '<', '=', '>', '?', '@',
			'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
			'[', '\\', ']', '^', '_', '`',
			'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
			'{', '|', '}', '~'
		];
		
		// 使用线程局部随机数生成器
		let mut rng = rand::thread_rng();
		
		// 创建一个空数组并填充随机字符
		let mut result = ['\0'; 64];
		for i in 0..64 {
			// 从可打印字符列表中随机选择一个
			result[i] = *PRINTABLE_ASCII.choose(&mut rng).unwrap();
		}
		
		result
	}
}
