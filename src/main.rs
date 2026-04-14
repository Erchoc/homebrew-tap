mod config_parser;
mod diagnosis;
mod log_parser;
mod report;

use chrono::{Days, Local, Months, NaiveDate};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn stash_dir() -> String {
    let home = env::var("HOME").expect("HOME not set");
    format!("{home}/Library/Application Support/Stash")
}

fn stash_core_dir() -> String {
    format!("{}/Core", stash_dir())
}

fn report_path() -> String {
    // 开发模式：输出到项目目录下的 output/
    // 安装后：输出到 ~/Downloads/
    if cfg!(debug_assertions) {
        let dir = "output";
        std::fs::create_dir_all(dir).expect("创建 output 目录失败");
        format!("{dir}/vvpn_report.html")
    } else {
        let home = env::var("HOME").expect("HOME not set");
        format!("{home}/Downloads/vvpn_report.html")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("cd") => cmd_cd(),
        Some("scan") => cmd_scan(&args[2..]),
        Some("open") => cmd_open(),
        Some("clean") => cmd_clean(),
        _ => print_usage(),
    }
}

fn print_usage() {
    eprintln!(
        r#"
  vvpn - Stash VPN 环境诊断工具

  用法:
    vvpn cd                               进入 Stash 目录（启动子 shell，exit 退出）
    vvpn scan [--today|--week|--month]    扫描并生成报告（默认全部日志）
    vvpn open                             在浏览器中打开诊断报告
    vvpn clean                            清除 Stash 日志

  时间范围:
    --today    仅分析今天的日志
    --week     仅分析最近 7 天的日志
    --month    仅分析最近 30 天的日志

  Stash 数据: ~/Library/Application Support/Stash/Core
"#
    );
}

fn check_stash_dir(stash_dir: &str) {
    if !Path::new(stash_dir).exists() {
        eprintln!("错误: Stash 数据目录不存在");
        eprintln!("  路径: {stash_dir}");
        eprintln!();
        eprintln!("  可能的原因:");
        eprintln!("    1. 尚未安装 Stash for macOS");
        eprintln!("    2. Stash 安装后从未运行过");
        eprintln!("    3. 数据目录被移动或删除");
        eprintln!();
        eprintln!("  请先安装并运行一次 Stash: https://stash.ws");
        std::process::exit(1);
    }
}

fn parse_since(args: &[String]) -> Option<NaiveDate> {
    let today = Local::now().date_naive();
    for arg in args {
        match arg.as_str() {
            "--today" => return Some(today),
            "--week" => return today.checked_sub_days(Days::new(6)),
            "--month" => return today.checked_sub_months(Months::new(1)),
            other => {
                eprintln!("未知参数: {other}");
                eprintln!("可用: --today, --week, --month");
                std::process::exit(1);
            }
        }
    }
    None
}

fn cmd_cd() {
    let dir = stash_dir();
    if !Path::new(&dir).exists() {
        eprintln!("错误: Stash 目录不存在: {dir}");
        std::process::exit(1);
    }

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    eprintln!("进入 Stash 目录（exit 退出）: {dir}");
    let status = Command::new(&shell)
        .current_dir(&dir)
        .status()
        .expect("启动 shell 失败");

    std::process::exit(status.code().unwrap_or(1));
}

fn cmd_scan(args: &[String]) {
    let stash_dir = stash_core_dir();
    let report_path = report_path();
    let since = parse_since(args);

    check_stash_dir(&stash_dir);

    let range_label = match since {
        Some(d) => format!("{d} 至今"),
        None => "全部".to_string(),
    };
    println!("扫描 Stash 目录: {stash_dir}");
    println!("  时间范围: {range_label}");

    let logs = log_parser::scan_logs(&stash_dir, since);
    if logs.file_count == 0 {
        println!("  未找到日志文件，没什么可分析的 ✌️");
        return;
    }
    println!(
        "  日志文件: {} 个, 错误: {} 条, 警告: {} 条",
        logs.file_count, logs.error_count, logs.warn_count
    );

    let config = config_parser::parse_config(&stash_dir);
    println!(
        "  节点: {} 个, 代理组: {} 个, 规则: {} 条",
        config.proxy_count, config.group_count, config.rule_count
    );

    let issues = diagnosis::diagnose(&logs, &config);
    let problem_count = issues
        .iter()
        .filter(|i| i.severity != diagnosis::Severity::Info)
        .count();
    let info_count = issues.len() - problem_count;
    if problem_count == 0 {
        println!("  未发现问题 👍");
    } else {
        println!("  发现问题: {problem_count} 个");
    }
    if info_count > 0 {
        println!("  信息提示: {info_count} 条");
    }

    let html = report::render(&logs, &config, &issues);
    std::fs::write(&report_path, html).expect("写入报告失败");
    println!("\n报告已生成: {report_path}");

    print!("是否立即查看报告? [Y/n] ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let answer = input.trim().to_lowercase();

    if answer.is_empty() || answer == "y" || answer == "yes" {
        open_report(&report_path);
    }
}

fn cmd_clean() {
    let stash_dir = stash_core_dir();
    check_stash_dir(&stash_dir);

    let dirs = ["logs", "crashes"];
    let mut total_files = 0u64;
    let mut total_bytes = 0u64;

    for name in &dirs {
        let dir = Path::new(&stash_dir).join(name);
        if !dir.is_dir() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    total_bytes += meta.len();
                }
                if std::fs::remove_file(&path).is_ok() {
                    total_files += 1;
                }
            }
        }
    }

    if total_files == 0 {
        println!("没有需要清理的日志文件");
    } else {
        let size = if total_bytes >= 1024 * 1024 {
            format!("{:.1} MB", total_bytes as f64 / 1024.0 / 1024.0)
        } else if total_bytes >= 1024 {
            format!("{:.1} KB", total_bytes as f64 / 1024.0)
        } else {
            format!("{total_bytes} B")
        };
        println!("已清理 {total_files} 个日志文件，释放 {size}");
    }
}

fn cmd_open() {
    let report_path = report_path();
    if !Path::new(&report_path).exists() {
        eprintln!("报告不存在，请先运行 vvpn scan");
        std::process::exit(1);
    }
    open_report(&report_path);
}

fn open_report(path: &str) {
    Command::new("open")
        .arg(path)
        .status()
        .expect("打开报告失败");
}
