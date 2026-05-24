use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut fingerprint = String::new();
    let mut tier = String::from("pro");
    let mut expiry: Option<String> = None;
    let mut key_file = String::from("private_key.pem");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--fingerprint" | "-f" => {
                i += 1;
                fingerprint = args.get(i).cloned().unwrap_or_default();
            }
            "--tier" | "-t" => {
                i += 1;
                tier = args.get(i).cloned().unwrap_or_else(|| "pro".into());
            }
            "--expiry" | "-e" => {
                i += 1;
                expiry = args.get(i).cloned();
            }
            "--key-file" | "-k" => {
                i += 1;
                key_file = args.get(i).cloned().unwrap_or_else(|| "private_key.pem".into());
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("未知参数: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if fingerprint.is_empty() {
        eprintln!("错误: 必须提供 --fingerprint 参数");
        print_help();
        std::process::exit(1);
    }

    let private_key_pem = match fs::read_to_string(&key_file) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("错误: 无法读取私钥文件 '{}': {}", key_file, e);
            std::process::exit(1);
        }
    };

    match wasm_license::sign_license(
        &fingerprint,
        &tier,
        expiry.as_deref(),
        &[],
        &private_key_pem,
    ) {
        Ok(license_key) => {
            println!("{}", license_key);
            eprintln!();
            eprintln!("  授权等级: {}", tier);
            eprintln!("  指纹: {}", fingerprint);
            eprintln!("  过期: {}", expiry.as_deref().unwrap_or("永久"));
        }
        Err(e) => {
            eprintln!("签名失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!(
        "QuantVault 许可证生成器

用法:
  license-gen --fingerprint <SHA256哈希> [选项]

选项:
  -f, --fingerprint <HASH>   机器指纹 (SHA-256, 必填)
  -t, --tier <pro|std>       授权等级 (默认: pro)
  -e, --expiry <YYYY-MM-DD>  过期日期 (默认: 永久)
  -k, --key-file <PATH>      私钥 PEM 文件路径 (默认: private_key.pem)
  -h, --help                 显示此帮助

示例:
  license-gen -f abc123def456 -t pro
  license-gen -f abc123def456 -t pro -k /path/to/private_key.pem
  license-gen -f abc123def456 -t pro -e 2027-06-30
"
    );
}
