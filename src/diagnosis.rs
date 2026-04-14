use crate::config_parser::ConfigReport;
use crate::log_parser::LogReport;

#[derive(Debug)]
pub struct Issue {
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub solution: String,
    pub count: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl Severity {
    pub fn label(&self) -> &str {
        match self {
            Severity::Critical => "严重",
            Severity::Warning => "警告",
            Severity::Info => "提示",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Severity::Critical => "#ef4444",
            Severity::Warning => "#f59e0b",
            Severity::Info => "#3b82f6",
        }
    }
}

pub fn diagnose(logs: &LogReport, config: &ConfigReport) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check StashLink crashes
    if let Some(&count) = logs.error_patterns.get("StashLink 服务崩溃重启") {
        if count > 5 {
            issues.push(Issue {
                severity: Severity::Critical,
                title: format!("StashLink 频繁崩溃 ({count} 次)"),
                description: format!(
                    "StashLink 服务在日志期间崩溃并重启了 {count} 次。\
                     这是 Stash 的内网穿透/局域网代理功能，频繁崩溃意味着该功能基本不可用。\
                     可能是 underlying proxy 配置为空或所选节点不稳定导致。"
                ),
                solution: "1. 在 Stash 设置中检查 StashLink 的 underlying proxy 是否已正确选择一个可用节点\n\
                           2. 如果不使用 StashLink 功能，建议关闭它以减少资源消耗和日志噪音\n\
                           3. 若需要此功能，切换到更稳定的节点（如高级 IEPL 专线）"
                    .to_string(),
                count,
            });
        }
    }

    // Check NAT type detection failures
    if let Some(&count) = logs.error_patterns.get("NAT 类型检测失败 (UDP)") {
        if count > 10 {
            issues.push(Issue {
                severity: Severity::Critical,
                title: format!("UDP NAT 检测大量失败 ({count} 次)"),
                description: format!(
                    "NAT 类型检测失败了 {count} 次，通常伴随 StashLink 崩溃出现。\
                     这表明 UDP 通道不稳定，可能影响游戏加速、语音通话等实时应用。"
                ),
                solution: "1. 确认当前节点支持 UDP 转发（配置中 udp: true 已启用）\n\
                           2. 检查本地防火墙是否放行了 UDP 端口\n\
                           3. 尝试切换到不同的服务器节点测试 UDP 连通性"
                    .to_string(),
                count,
            });
        }
    }

    // Check proxy group with no proxy
    if let Some(&count) = logs.error_patterns.get("代理组无可用节点") {
        issues.push(Issue {
            severity: Severity::Critical,
            title: "代理组缺少可用节点".to_string(),
            description: format!(
                "有 {count} 次日志记录显示代理组找不到可用的代理节点，直接回退到 DIRECT 直连。\
                 这意味着在这些时段，你的翻墙完全失效了。"
            ),
            solution: "1. 检查 proxy-groups 中的 PROXY 组是否关联了有效节点\n\
                       2. 确认订阅是否过期或节点是否全部下线\n\
                       3. 手动更新订阅: Stash → 配置 → 更新订阅"
                .to_string(),
            count,
        });
    }

    // Check default interface lost
    if let Some(&count) = logs.error_patterns.get("默认网络接口丢失") {
        if count > 3 {
            issues.push(Issue {
                severity: Severity::Warning,
                title: format!("网络接口频繁丢失 ({count} 次)"),
                description: "默认网络接口（通常是 Wi-Fi en0）多次丢失，可能导致短暂断网。\
                             这通常发生在睡眠唤醒、切换 Wi-Fi 网络、或 VPN 重连时。"
                    .to_string(),
                solution: "1. 这通常是 macOS 网络切换的正常现象，不需要特别处理\n\
                           2. 如果频繁发生，检查 Wi-Fi 信号稳定性\n\
                           3. 确认没有其他 VPN 软件与 Stash 冲突"
                    .to_string(),
                count,
            });
        }
    }

    // Check TLS errors
    if let Some(&count) = logs.error_patterns.get("TLS 安全连接失败 (-1200)") {
        issues.push(Issue {
            severity: Severity::Warning,
            title: format!("TLS 连接错误 ({count} 次)"),
            description: "与 app-analytics-services.com 的 TLS 握手失败。\
                         这是 Stash 内置的 Google Analytics 遥测请求被代理规则拦截导致的。"
                .to_string(),
            solution: "1. 这是一个无害的错误，不影响代理功能\n\
                       2. 可以在规则中添加 DOMAIN,app-analytics-services.com,DIRECT 来消除此错误"
                .to_string(),
            count,
        });
    }

    // Check timeout errors
    if let Some(&count) = logs
        .error_patterns
        .get("网络请求超时 (NSURLErrorDomain -1001)")
    {
        issues.push(Issue {
            severity: Severity::Warning,
            title: format!("网络请求超时 ({count} 次)"),
            description: "多次发生网络请求超时，目标同样是 analytics 服务。\
                         表明代理链路对某些 HTTPS 请求的处理存在延迟。"
                .to_string(),
            solution: "1. 检查代理节点延迟，切换到更快的节点\n\
                       2. 在规则中将 analytics 域名设为 DIRECT 直连"
                .to_string(),
            count,
        });
    }

    // Config-level issues
    if config.single_cipher && config.single_password && config.proxy_count > 10 {
        issues.push(Issue {
            severity: Severity::Info,
            title: "所有节点使用相同密码和加密方式".to_string(),
            description: format!(
                "全部 {} 个节点使用相同的 cipher (aes-128-gcm) 和同一密码。\
                 这是机场订阅的正常配置，但意味着一旦密码泄露，所有节点都会受影响。",
                config.proxy_count
            ),
            solution: "1. 定期在机场面板更换密码/token\n\
                       2. 不要将订阅链接分享给他人"
                .to_string(),
            count: config.proxy_count,
        });
    }

    if !config.ipv6 {
        issues.push(Issue {
            severity: Severity::Info,
            title: "IPv6 已禁用".to_string(),
            description: "当前配置禁用了 IPv6。在某些网络环境下，\
                         启用 IPv6 可能改善连接速度。但大多数代理场景下禁用是合理的。"
                .to_string(),
            solution: "如果你的 ISP 支持 IPv6 且节点也支持，可以在配置中启用 ipv6: true"
                .to_string(),
            count: 0,
        });
    }

    if config.rule_count == 0 {
        issues.push(Issue {
            severity: Severity::Warning,
            title: "规则列表为空".to_string(),
            description:
                "未检测到分流规则，所有流量可能走同一出口（取决于 mode 配置）。\
                 当前模式为 rule 但规则数为 0，可能是解析问题或配置遗漏。"
                    .to_string(),
            solution: "1. 检查 config.yaml 中 rules 部分是否正常\n\
                       2. 建议使用成熟的规则集（如 blackmatrix7 分流规则）"
                .to_string(),
            count: 0,
        });
    }

    // Sort: Critical first, then by count
    issues.sort_by(|a, b| a.severity.cmp(&b.severity).then(b.count.cmp(&a.count)));

    issues
}
