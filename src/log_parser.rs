use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LogReport {
    pub file_count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    /// module -> count
    pub errors_by_module: HashMap<String, usize>,
    /// error message pattern -> count
    pub error_patterns: HashMap<String, usize>,
    /// date string -> error count
    pub errors_by_date: HashMap<String, usize>,
    /// crash file summaries
    pub crash_summaries: Vec<String>,
    /// raw recent errors (last ~20)
    pub recent_errors: Vec<String>,
}

/// 从文件名提取日期（如 `2026-04-11-111015.log` → `2026-04-11`）
fn date_from_filename(path: &Path) -> Option<NaiveDate> {
    let stem = path.file_stem()?.to_str()?;
    let date_str = stem.get(..10)?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

/// `since` 为 None 时扫描全部日志，否则只扫描 >= since 的日志
pub fn scan_logs(stash_dir: &str, since: Option<NaiveDate>) -> LogReport {
    let mut report = LogReport {
        file_count: 0,
        error_count: 0,
        warn_count: 0,
        info_count: 0,
        errors_by_module: HashMap::new(),
        error_patterns: HashMap::new(),
        errors_by_date: HashMap::new(),
        crash_summaries: Vec::new(),
        recent_errors: Vec::new(),
    };

    // Parse core logs
    let logs_dir = Path::new(stash_dir).join("logs");
    if logs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&logs_dir) {
            let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            files.sort_by_key(|e| e.file_name());
            for entry in &files {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("log") {
                    continue;
                }
                if let Some(min_date) = since {
                    if let Some(file_date) = date_from_filename(&path) {
                        if file_date < min_date {
                            continue;
                        }
                    }
                }
                report.file_count += 1;
                parse_core_log(&path, &mut report);
            }
        }
    }

    // Parse crash logs (filtered by file modification time when since is set)
    let crashes_dir = Path::new(stash_dir).join("crashes");
    if crashes_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&crashes_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("log") {
                    continue;
                }
                if let Some(min_date) = since {
                    // crash logs 文件名格式不一定带日期，用修改时间过滤
                    let dominated = path
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let dt: chrono::DateTime<chrono::Local> = t.into();
                            dt.date_naive()
                        });
                    if let Some(file_date) = dominated {
                        if file_date < min_date {
                            continue;
                        }
                    }
                }
                report.file_count += 1;
                parse_crash_log(&path, &mut report);
            }
        }
    }

    // Keep only top recent errors
    if report.recent_errors.len() > 30 {
        let len = report.recent_errors.len();
        report.recent_errors = report.recent_errors[len - 30..].to_vec();
    }

    report
}

fn parse_core_log(path: &Path, report: &mut LogReport) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Extract date from filename like 2026-04-11-111015.log
    let date_from_filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.get(..10))
        .unwrap_or("unknown")
        .to_string();

    let re = Regex::new(r"^\[(INFO|WARN|ERRO)\] \[(\d{2}:\d{2}:\d{2})\] \[([^\]]+)\] (.+)$")
        .unwrap();

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            let level = &caps[1];
            let module = caps[3].to_string();
            let message = caps[4].to_string();

            match level {
                "ERRO" => {
                    report.error_count += 1;
                    *report.errors_by_module.entry(module.clone()).or_insert(0) += 1;
                    *report
                        .errors_by_date
                        .entry(date_from_filename.clone())
                        .or_insert(0) += 1;

                    // Normalize the error pattern
                    let pattern = normalize_error(&message);
                    *report.error_patterns.entry(pattern).or_insert(0) += 1;

                    report
                        .recent_errors
                        .push(format!("[{date_from_filename} {}] [{module}] {message}", &caps[2]));
                }
                "WARN" => {
                    report.warn_count += 1;
                    // Also track warn patterns that look like errors
                    if message.contains("no proxy available") {
                        let pattern = normalize_error(&message);
                        *report.error_patterns.entry(pattern).or_insert(0) += 1;
                        *report
                            .errors_by_module
                            .entry(format!("{module}(WARN)"))
                            .or_insert(0) += 1;
                    }
                }
                "INFO" => report.info_count += 1,
                _ => {}
            }
        }
    }
}

fn parse_crash_log(path: &Path, report: &mut LogReport) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let re_error =
        Regex::new(r"(?i)(error|ERROR|请求超时|TLS错误|failed|WARNING)").unwrap();

    let mut errors_in_file = 0;
    let mut sample_errors: Vec<String> = Vec::new();

    for line in content.lines() {
        if re_error.is_match(line) {
            errors_in_file += 1;
            report.error_count += 1;

            // Extract error category
            if line.contains("请求超时") || line.contains("Code=-1001") {
                *report
                    .error_patterns
                    .entry("网络请求超时 (NSURLErrorDomain -1001)".to_string())
                    .or_insert(0) += 1;
            } else if line.contains("TLS错误") || line.contains("Code=-1200") {
                *report
                    .error_patterns
                    .entry("TLS 安全连接失败 (-1200)".to_string())
                    .or_insert(0) += 1;
            } else if line.contains("WARNING") && line.contains("reentrant") {
                *report
                    .error_patterns
                    .entry("NSTableView reentrant 操作警告".to_string())
                    .or_insert(0) += 1;
            } else if line.contains("mach port") {
                *report
                    .error_patterns
                    .entry("IMK mach port 通信错误".to_string())
                    .or_insert(0) += 1;
            }

            if sample_errors.len() < 3 {
                let short = if line.len() > 120 {
                    format!("{}...", &line[..120])
                } else {
                    line.to_string()
                };
                sample_errors.push(short);
            }
        }
    }

    if errors_in_file > 0 {
        report.crash_summaries.push(format!(
            "{filename}: {errors_in_file} 条错误/警告"
        ));
    }
}

fn normalize_error(msg: &str) -> String {
    if msg.contains("StashLink server is down") {
        return "StashLink 服务崩溃重启".to_string();
    }
    if msg.contains("failed to get nat type") {
        return "NAT 类型检测失败 (UDP)".to_string();
    }
    if msg.contains("default interface not found") {
        return "默认网络接口丢失".to_string();
    }
    if msg.contains("no proxy available") {
        return "代理组无可用节点".to_string();
    }
    if msg.contains("connection refused") {
        return "连接被拒绝".to_string();
    }
    if msg.contains("timeout") || msg.contains("timed out") {
        return "连接超时".to_string();
    }
    if msg.contains("i/o timeout") {
        return "I/O 超时".to_string();
    }

    // Truncate long messages
    let short: String = msg.chars().take(60).collect();
    short
}
