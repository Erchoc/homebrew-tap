mod config_parser;
mod diagnosis;
mod log_parser;
mod report;

use std::env;
use std::process::Command;

fn stash_dir() -> String {
    let home = env::var("HOME").expect("HOME not set");
    format!("{home}/Library/Application Support/Stash/Core")
}

fn report_path() -> String {
    let home = env::var("HOME").expect("HOME not set");
    format!("{home}/Downloads/vvpn_report.html")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("scan") => cmd_scan(),
        Some("open") => cmd_open(),
        _ => print_usage(),
    }
}

fn print_usage() {
    eprintln!(
        r#"
  vvpn - Stash VPN 环境诊断工具

  用法:
    vvpn scan    扫描 Stash 目录，分析日志与配置，生成诊断报告
    vvpn open    在浏览器中打开诊断报告

  报告输出: ~/Downloads/vvpn_report.html
"#
    );
}

fn cmd_scan() {
    let stash_dir = stash_dir();
    let report_path = report_path();

    println!("扫描 Stash 目录: {stash_dir}");

    let logs = log_parser::scan_logs(&stash_dir);
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
    println!(
        "  发现问题: {} 个 (Top 3 + {} 个小问题)",
        issues.len(),
        issues.len().saturating_sub(3)
    );

    let html = report::render(&logs, &config, &issues);
    std::fs::write(&report_path, html).expect("写入报告失败");
    println!("\n报告已生成: {report_path}");
}

fn cmd_open() {
    let report_path = report_path();
    if !std::path::Path::new(&report_path).exists() {
        eprintln!("报告不存在，请先运行 vvpn scan");
        std::process::exit(1);
    }
    Command::new("open")
        .arg(&report_path)
        .status()
        .expect("打开报告失败");
}
