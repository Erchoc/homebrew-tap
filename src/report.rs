use crate::config_parser::ConfigReport;
use crate::diagnosis::{Issue, Severity};
use crate::log_parser::LogReport;

pub fn render(logs: &LogReport, config: &ConfigReport, issues: &[Issue]) -> String {
    let top_issues = if issues.len() >= 3 {
        &issues[..3]
    } else {
        issues
    };
    let minor_issues = if issues.len() > 3 {
        &issues[3..]
    } else {
        &[]
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>vvpn 诊断报告</title>
<style>
{css}
</style>
</head>
<body>
<div class="container">

<header>
  <h1>VPN 环境诊断报告</h1>
  <p class="subtitle">由 vvpn 生成于 {now} · 数据来源: Stash for macOS</p>
</header>

<!-- Top 3 Issues -->
<section class="top-issues">
  <h2>最需要关注的问题</h2>
  <div class="issue-cards">
    {top_issues_html}
  </div>
</section>

<!-- Dashboard -->
<section class="dashboard">
  <h2>环境概览</h2>
  <div class="grid">
    {subscription_card}
    {stats_card}
    {ports_card}
  </div>
</section>

<!-- Error Distribution -->
<section class="charts">
  <h2>错误分布</h2>
  <div class="grid">
    <div class="card">
      <h3>按类型分布</h3>
      <div class="bar-chart">
        {error_pattern_bars}
      </div>
    </div>
    <div class="card">
      <h3>按模块分布</h3>
      <div class="bar-chart">
        {error_module_bars}
      </div>
    </div>
  </div>
</section>

<!-- Node Distribution -->
<section class="nodes">
  <h2>节点分布</h2>
  <div class="grid">
    <div class="card">
      <h3>按地区</h3>
      <div class="bar-chart">
        {region_bars}
      </div>
    </div>
    <div class="card">
      <h3>按线路等级</h3>
      <div class="bar-chart">
        {tier_bars}
      </div>
    </div>
  </div>
</section>

<!-- Node Detail -->
<section class="node-detail">
  <h2>节点技术详情</h2>
  <div class="grid">
    <div class="card">
      <h3>协议 &amp; 加密</h3>
      <table>
        <tr><td>协议类型</td><td>{protocol_info}</td></tr>
        <tr><td>加密方式</td><td>{cipher_info}</td></tr>
        <tr><td>独立服务器</td><td>{server_count} 台</td></tr>
        <tr><td>密码统一</td><td>{password_uniform}</td></tr>
      </table>
    </div>
    <div class="card">
      <h3>DNS 配置</h3>
      <ul class="dns-list">
        {dns_list}
      </ul>
    </div>
  </div>
</section>

<!-- Minor Issues -->
{minor_issues_html}

<!-- Recent Errors -->
<section class="recent-errors">
  <details>
    <summary>最近的错误日志 ({recent_count} 条)</summary>
    <div class="log-lines">
      {recent_errors_html}
    </div>
  </details>
</section>

<footer>
  <p>vvpn v0.1.0 · Stash VPN 环境诊断工具</p>
</footer>

</div>
</body>
</html>"##,
        css = CSS,
        now = now,
        top_issues_html = render_top_issues(top_issues),
        subscription_card = render_subscription_card(config),
        stats_card = render_stats_card(logs, config),
        ports_card = render_ports_card(config),
        error_pattern_bars = render_bar_chart(&logs.error_patterns, "#ef4444"),
        error_module_bars = render_bar_chart(&logs.errors_by_module, "#f59e0b"),
        region_bars = render_bar_chart(&config.nodes_by_region, "#3b82f6"),
        tier_bars = render_bar_chart(&config.nodes_by_tier, "#8b5cf6"),
        protocol_info = render_map_inline(&config.nodes_by_type),
        cipher_info = render_map_inline(&config.nodes_by_cipher),
        server_count = config.unique_servers.len(),
        password_uniform = if config.single_password { "是" } else { "否" },
        dns_list = render_dns_list(&config.dns_servers),
        minor_issues_html = render_minor_issues(minor_issues),
        recent_count = logs.recent_errors.len(),
        recent_errors_html = render_recent_errors(&logs.recent_errors),
    )
}

fn render_top_issues(issues: &[Issue]) -> String {
    let mut html = String::new();
    for (i, issue) in issues.iter().enumerate() {
        let num = i + 1;
        let icon = match issue.severity {
            Severity::Critical => "!!",
            Severity::Warning => "!",
            Severity::Info => "i",
        };
        html.push_str(&format!(
            r#"<div class="issue-card" style="border-left: 4px solid {color}">
  <div class="issue-header">
    <span class="issue-num">#{num}</span>
    <span class="issue-badge" style="background:{color}">{icon} {severity}</span>
  </div>
  <h3>{title}</h3>
  <p class="issue-desc">{desc}</p>
  <div class="issue-solution">
    <strong>解决方案:</strong>
    <pre>{solution}</pre>
  </div>
</div>"#,
            color = issue.severity.color(),
            severity = issue.severity.label(),
            title = html_escape(&issue.title),
            desc = html_escape(&issue.description),
            solution = html_escape(&issue.solution),
        ));
    }
    html
}

fn render_subscription_card(config: &ConfigReport) -> String {
    let provider = config
        .subscription_provider
        .as_deref()
        .unwrap_or("未知");
    let traffic = config
        .traffic_info
        .as_deref()
        .unwrap_or("未知");
    let expiry = config.expiry_info.as_deref().unwrap_or("未知");

    format!(
        r#"<div class="card highlight">
  <h3>订阅信息</h3>
  <table>
    <tr><td>机场</td><td><strong>{provider}</strong></td></tr>
    <tr><td>流量</td><td>{traffic}</td></tr>
    <tr><td>到期</td><td>{expiry}</td></tr>
    <tr><td>模式</td><td>{mode}</td></tr>
  </table>
</div>"#,
        provider = html_escape(provider),
        traffic = html_escape(traffic),
        expiry = html_escape(expiry),
        mode = html_escape(&config.mode),
    )
}

fn render_stats_card(logs: &LogReport, config: &ConfigReport) -> String {
    format!(
        r#"<div class="card">
  <h3>统计概览</h3>
  <div class="stat-grid">
    <div class="stat"><span class="stat-num">{}</span><span class="stat-label">节点数</span></div>
    <div class="stat"><span class="stat-num">{}</span><span class="stat-label">代理组</span></div>
    <div class="stat"><span class="stat-num">{}</span><span class="stat-label">规则数</span></div>
    <div class="stat"><span class="stat-num">{}</span><span class="stat-label">日志文件</span></div>
    <div class="stat"><span class="stat-num err">{}</span><span class="stat-label">错误</span></div>
    <div class="stat"><span class="stat-num warn">{}</span><span class="stat-label">警告</span></div>
  </div>
</div>"#,
        config.proxy_count,
        config.group_count,
        config.rule_count,
        logs.file_count,
        logs.error_count,
        logs.warn_count,
    )
}

fn render_ports_card(config: &ConfigReport) -> String {
    let mut rows = String::new();
    let mut ports: Vec<_> = config.ports.iter().collect();
    ports.sort_by_key(|(k, _)| (*k).clone());
    for (name, port) in ports {
        rows.push_str(&format!("<tr><td>{name}</td><td>{port}</td></tr>"));
    }
    format!(
        r#"<div class="card">
  <h3>端口配置</h3>
  <table>{rows}</table>
  <table>
    <tr><td>IPv6</td><td>{ipv6}</td></tr>
    <tr><td>局域网共享</td><td>{lan}</td></tr>
  </table>
</div>"#,
        ipv6 = if config.ipv6 { "启用" } else { "禁用" },
        lan = if config.allow_lan { "启用" } else { "禁用" },
    )
}

fn render_bar_chart(data: &std::collections::HashMap<String, usize>, color: &str) -> String {
    let mut items: Vec<_> = data.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));
    items.truncate(10);

    let max_val = items.first().map(|(_, v)| **v).unwrap_or(1).max(1);

    let mut html = String::new();
    for (label, count) in &items {
        let pct = (**count as f64 / max_val as f64 * 100.0) as u32;
        html.push_str(&format!(
            r#"<div class="bar-row">
  <span class="bar-label" title="{label}">{short_label}</span>
  <div class="bar-track"><div class="bar-fill" style="width:{pct}%;background:{color}"></div></div>
  <span class="bar-value">{count}</span>
</div>"#,
            label = html_escape(label),
            short_label = html_escape(&truncate_str(label, 20)),
            count = count,
        ));
    }
    html
}

fn render_minor_issues(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return String::new();
    }
    let mut items = String::new();
    for issue in issues {
        items.push_str(&format!(
            r#"<details class="minor-issue">
  <summary>
    <span class="issue-badge small" style="background:{color}">{severity}</span>
    {title}
  </summary>
  <div class="minor-body">
    <p>{desc}</p>
    <div class="issue-solution">
      <strong>建议:</strong>
      <pre>{solution}</pre>
    </div>
  </div>
</details>"#,
            color = issue.severity.color(),
            severity = issue.severity.label(),
            title = html_escape(&issue.title),
            desc = html_escape(&issue.description),
            solution = html_escape(&issue.solution),
        ));
    }
    format!(
        r#"<section class="minor-issues">
  <h2>其他问题 (点击展开)</h2>
  {items}
</section>"#
    )
}

fn render_recent_errors(errors: &[String]) -> String {
    errors
        .iter()
        .rev()
        .map(|e| format!("<div class=\"log-line\">{}</div>", html_escape(e)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_dns_list(servers: &[String]) -> String {
    servers
        .iter()
        .map(|s| format!("<li>{}</li>", html_escape(s)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_map_inline(map: &std::collections::HashMap<String, usize>) -> String {
    let items: Vec<_> = map
        .iter()
        .map(|(k, v)| format!("{k} ({v})"))
        .collect();
    html_escape(&items.join(", "))
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let short: String = s.chars().take(max - 1).collect();
        format!("{short}…")
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CSS: &str = r#"
:root {
  --bg: #0f172a;
  --surface: #1e293b;
  --surface2: #334155;
  --text: #e2e8f0;
  --text-dim: #94a3b8;
  --accent: #38bdf8;
  --red: #ef4444;
  --amber: #f59e0b;
  --blue: #3b82f6;
  --purple: #8b5cf6;
  --green: #22c55e;
  --radius: 12px;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}

.container {
  max-width: 960px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

header {
  text-align: center;
  margin-bottom: 3rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--surface2);
}

header h1 {
  font-size: 2rem;
  font-weight: 700;
  background: linear-gradient(135deg, var(--accent), var(--purple));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 0.5rem;
}

.subtitle {
  color: var(--text-dim);
  font-size: 0.875rem;
}

h2 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: var(--text);
}

h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
  color: var(--text-dim);
}

section {
  margin-bottom: 2.5rem;
}

/* Cards */
.card {
  background: var(--surface);
  border-radius: var(--radius);
  padding: 1.25rem;
}

.card.highlight {
  border: 1px solid var(--accent);
  background: linear-gradient(135deg, rgba(56,189,248,0.08), rgba(139,92,246,0.05));
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1rem;
}

/* Tables */
table { width: 100%; border-collapse: collapse; }
td {
  padding: 0.4rem 0;
  border-bottom: 1px solid var(--surface2);
  font-size: 0.875rem;
}
td:first-child { color: var(--text-dim); width: 40%; }
td:last-child { text-align: right; }

/* Stats */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
  text-align: center;
}

.stat-num {
  display: block;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--accent);
}

.stat-num.err { color: var(--red); }
.stat-num.warn { color: var(--amber); }

.stat-label {
  font-size: 0.75rem;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Issue cards */
.issue-cards {
  display: grid;
  gap: 1rem;
}

.issue-card {
  background: var(--surface);
  border-radius: var(--radius);
  padding: 1.25rem;
  transition: transform 0.2s;
}

.issue-card:hover { transform: translateY(-2px); }

.issue-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.issue-num {
  font-size: 1.5rem;
  font-weight: 800;
  color: var(--text-dim);
}

.issue-badge {
  display: inline-block;
  padding: 0.15rem 0.6rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  color: #fff;
}

.issue-badge.small {
  font-size: 0.7rem;
  padding: 0.1rem 0.5rem;
}

.issue-card h3 {
  color: var(--text);
  font-size: 1.05rem;
  margin-bottom: 0.5rem;
}

.issue-desc {
  color: var(--text-dim);
  font-size: 0.875rem;
  margin-bottom: 1rem;
}

.issue-solution {
  background: var(--bg);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  font-size: 0.85rem;
}

.issue-solution strong {
  color: var(--green);
  display: block;
  margin-bottom: 0.4rem;
}

.issue-solution pre {
  white-space: pre-wrap;
  font-family: inherit;
  color: var(--text-dim);
  font-size: 0.825rem;
  line-height: 1.7;
}

/* Bar chart */
.bar-chart { display: flex; flex-direction: column; gap: 0.5rem; }

.bar-row {
  display: grid;
  grid-template-columns: 140px 1fr 50px;
  align-items: center;
  gap: 0.5rem;
}

.bar-label {
  font-size: 0.8rem;
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bar-track {
  height: 20px;
  background: var(--bg);
  border-radius: 4px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.6s ease;
  min-width: 4px;
}

.bar-value {
  font-size: 0.8rem;
  color: var(--text);
  text-align: right;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

/* Minor issues */
.minor-issues details {
  background: var(--surface);
  border-radius: var(--radius);
  margin-bottom: 0.5rem;
  overflow: hidden;
}

.minor-issues summary {
  padding: 0.75rem 1rem;
  cursor: pointer;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  user-select: none;
}

.minor-issues summary:hover { background: var(--surface2); }

.minor-body {
  padding: 0 1rem 1rem;
  font-size: 0.85rem;
  color: var(--text-dim);
}

.minor-body p { margin-bottom: 0.75rem; }

/* DNS */
.dns-list {
  list-style: none;
  font-size: 0.85rem;
  font-family: "SF Mono", "Fira Code", monospace;
}
.dns-list li {
  padding: 0.3rem 0;
  border-bottom: 1px solid var(--surface2);
  color: var(--text-dim);
}

/* Recent errors */
.recent-errors details {
  background: var(--surface);
  border-radius: var(--radius);
  overflow: hidden;
}

.recent-errors summary {
  padding: 0.75rem 1rem;
  cursor: pointer;
  font-size: 0.9rem;
  color: var(--text-dim);
}

.log-lines {
  max-height: 400px;
  overflow-y: auto;
  padding: 0 1rem 1rem;
}

.log-line {
  font-family: "SF Mono", "Fira Code", monospace;
  font-size: 0.75rem;
  padding: 0.25rem 0;
  border-bottom: 1px solid rgba(255,255,255,0.03);
  color: var(--text-dim);
  word-break: break-all;
}

/* Footer */
footer {
  text-align: center;
  padding-top: 2rem;
  border-top: 1px solid var(--surface2);
  color: var(--text-dim);
  font-size: 0.8rem;
}

/* Responsive */
@media (max-width: 640px) {
  .container { padding: 1rem; }
  header h1 { font-size: 1.5rem; }
  .grid { grid-template-columns: 1fr; }
  .bar-row { grid-template-columns: 100px 1fr 40px; }
  .stat-grid { grid-template-columns: repeat(2, 1fr); }
  .issue-card { padding: 1rem; }
}
"#;
