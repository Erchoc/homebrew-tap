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
<div class="page">

<!-- Hero -->
<header class="hero">
  <div class="hero-inner">
    <div class="hero-badge">DIAGNOSTICS</div>
    <h1>VPN 环境诊断报告</h1>
    <p class="hero-sub">vvpn · {now} · Stash for macOS</p>
  </div>
</header>

<!-- Stats Bar -->
<section class="stats-bar">
  <div class="container">
    <div class="stats-row">
      <div class="stat-item">
        <span class="stat-value">{proxy_count}</span>
        <span class="stat-name">NODES</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{group_count}</span>
        <span class="stat-name">GROUPS</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{rule_count}</span>
        <span class="stat-name">RULES</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{file_count}</span>
        <span class="stat-name">LOG FILES</span>
      </div>
      <div class="stat-item">
        <span class="stat-value accent-red">{error_count}</span>
        <span class="stat-name">ERRORS</span>
      </div>
      <div class="stat-item">
        <span class="stat-value accent-amber">{warn_count}</span>
        <span class="stat-name">WARNINGS</span>
      </div>
    </div>
  </div>
</section>

<!-- Top Issues -->
<section class="section-dark">
  <div class="container">
    <div class="section-label">TOP ISSUES</div>
    <h2>最需要关注的问题</h2>
    <div class="issues-stack">
      {top_issues_html}
    </div>
  </div>
</section>

<!-- Subscription & Config -->
<section class="section-light">
  <div class="container">
    <div class="section-label">OVERVIEW</div>
    <h2 class="heading-dark">环境概览</h2>
    <div class="grid-3">
      {subscription_card}
      {ports_card}
      {dns_card}
    </div>
  </div>
</section>

<!-- Error Distribution -->
<section class="section-dark">
  <div class="container">
    <div class="section-label">ERROR ANALYSIS</div>
    <h2>错误分布</h2>
    <div class="grid-2">
      <div class="panel">
        <h3>按类型</h3>
        <div class="bar-chart">
          {error_pattern_bars}
        </div>
      </div>
      <div class="panel">
        <h3>按模块</h3>
        <div class="bar-chart">
          {error_module_bars}
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Node Distribution -->
<section class="section-light">
  <div class="container">
    <div class="section-label">INFRASTRUCTURE</div>
    <h2 class="heading-dark">节点分布</h2>
    <div class="grid-2">
      <div class="panel-light">
        <h3 class="h3-dark">按地区</h3>
        <div class="bar-chart">
          {region_bars}
        </div>
      </div>
      <div class="panel-light">
        <h3 class="h3-dark">线路等级 &amp; 协议</h3>
        <div class="bar-chart">
          {tier_bars}
        </div>
        <div class="detail-table">
          <div class="detail-row"><span class="detail-key">协议</span><span class="detail-val">{protocol_info}</span></div>
          <div class="detail-row"><span class="detail-key">加密</span><span class="detail-val">{cipher_info}</span></div>
          <div class="detail-row"><span class="detail-key">独立服务器</span><span class="detail-val">{server_count} 台</span></div>
          <div class="detail-row"><span class="detail-key">密码统一</span><span class="detail-val">{password_uniform}</span></div>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Minor Issues -->
{minor_issues_html}

<!-- Recent Errors -->
<section class="section-dark">
  <div class="container">
    <div class="section-label">RAW LOGS</div>
    <h2>最近的错误日志</h2>
    <div class="log-panel">
      <div class="log-header">
        <span>{recent_count} 条记录</span>
      </div>
      <div class="log-body">
        {recent_errors_html}
      </div>
    </div>
  </div>
</section>

<footer>
  <div class="container">
    <p>vvpn v0.1.0</p>
  </div>
</footer>

</div>
</body>
</html>"##,
        css = CSS,
        now = now,
        proxy_count = config.proxy_count,
        group_count = config.group_count,
        rule_count = config.rule_count,
        file_count = logs.file_count,
        error_count = logs.error_count,
        warn_count = logs.warn_count,
        top_issues_html = render_top_issues(top_issues),
        subscription_card = render_subscription_card(config),
        ports_card = render_ports_card(config),
        dns_card = render_dns_card(config),
        error_pattern_bars = render_bar_chart(&logs.error_patterns, "#DA291C"),
        error_module_bars = render_bar_chart(&logs.errors_by_module, "#8F8F8F"),
        region_bars = render_bar_chart_dark(&config.nodes_by_region, "#181818"),
        tier_bars = render_bar_chart_dark(&config.nodes_by_tier, "#DA291C"),
        protocol_info = render_map_inline(&config.nodes_by_type),
        cipher_info = render_map_inline(&config.nodes_by_cipher),
        server_count = config.unique_servers.len(),
        password_uniform = if config.single_password { "是" } else { "否" },
        minor_issues_html = render_minor_issues(minor_issues),
        recent_count = logs.recent_errors.len(),
        recent_errors_html = render_recent_errors(&logs.recent_errors),
    )
}

fn render_top_issues(issues: &[Issue]) -> String {
    let mut html = String::new();
    for (i, issue) in issues.iter().enumerate() {
        let num = i + 1;
        let (badge_class, badge_text) = match issue.severity {
            Severity::Critical => ("badge-critical", "CRITICAL"),
            Severity::Warning => ("badge-warning", "WARNING"),
            Severity::Info => ("badge-info", "INFO"),
        };

        // Split solution into numbered steps
        let steps_html: String = issue
            .solution
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return String::new();
                }
                format!(
                    "<div class=\"step\">{}</div>",
                    html_escape(trimmed)
                )
            })
            .collect();

        html.push_str(&format!(
            r#"<div class="issue">
  <div class="issue-top">
    <span class="issue-num">#{num}</span>
    <span class="badge {badge_class}">{badge_text}</span>
  </div>
  <h3 class="issue-title">{title}</h3>
  <p class="issue-desc">{desc}</p>
  <div class="action-box">
    <div class="action-label">ACTION REQUIRED</div>
    <div class="action-steps">
      {steps_html}
    </div>
  </div>
</div>"#,
            title = html_escape(&issue.title),
            desc = html_escape(&issue.description),
        ));
    }
    html
}

fn render_subscription_card(config: &ConfigReport) -> String {
    let provider = config.subscription_provider.as_deref().unwrap_or("未知");
    let traffic = config.traffic_info.as_deref().unwrap_or("未知");
    let expiry = config.expiry_info.as_deref().unwrap_or("未知");

    format!(
        r#"<div class="info-card accent-border">
  <div class="card-label">SUBSCRIPTION</div>
  <div class="card-body">
    <div class="kv"><span class="k">机场</span><span class="v strong">{provider}</span></div>
    <div class="kv"><span class="k">流量</span><span class="v">{traffic}</span></div>
    <div class="kv"><span class="k">到期</span><span class="v">{expiry}</span></div>
    <div class="kv"><span class="k">模式</span><span class="v">{mode}</span></div>
  </div>
</div>"#,
        provider = html_escape(provider),
        traffic = html_escape(traffic),
        expiry = html_escape(expiry),
        mode = html_escape(&config.mode),
    )
}

fn render_ports_card(config: &ConfigReport) -> String {
    let mut rows = String::new();
    let mut ports: Vec<_> = config.ports.iter().collect();
    ports.sort_by_key(|(k, _)| (*k).clone());
    for (name, port) in ports {
        rows.push_str(&format!(
            "<div class=\"kv\"><span class=\"k\">{name}</span><span class=\"v\">{port}</span></div>"
        ));
    }

    format!(
        r#"<div class="info-card">
  <div class="card-label">PORTS</div>
  <div class="card-body">
    {rows}
    <div class="kv"><span class="k">IPv6</span><span class="v">{ipv6}</span></div>
    <div class="kv"><span class="k">局域网</span><span class="v">{lan}</span></div>
  </div>
</div>"#,
        ipv6 = if config.ipv6 { "启用" } else { "禁用" },
        lan = if config.allow_lan { "启用" } else { "禁用" },
    )
}

fn render_dns_card(config: &ConfigReport) -> String {
    let items: String = config
        .dns_servers
        .iter()
        .map(|s| format!("<div class=\"dns-item\">{}</div>", html_escape(s)))
        .collect();

    format!(
        r#"<div class="info-card">
  <div class="card-label">DNS</div>
  <div class="card-body">
    {items}
  </div>
</div>"#
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
  <span class="bar-label" title="{full}">{short}</span>
  <div class="bar-track"><div class="bar-fill" style="width:{pct}%;background:{color}"></div></div>
  <span class="bar-value">{count}</span>
</div>"#,
            full = html_escape(label),
            short = html_escape(&truncate_str(label, 22)),
        ));
    }
    html
}

fn render_bar_chart_dark(data: &std::collections::HashMap<String, usize>, color: &str) -> String {
    let mut items: Vec<_> = data.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));
    items.truncate(10);
    let max_val = items.first().map(|(_, v)| **v).unwrap_or(1).max(1);

    let mut html = String::new();
    for (label, count) in &items {
        let pct = (**count as f64 / max_val as f64 * 100.0) as u32;
        html.push_str(&format!(
            r#"<div class="bar-row">
  <span class="bar-label dark-label" title="{full}">{short}</span>
  <div class="bar-track light-track"><div class="bar-fill" style="width:{pct}%;background:{color}"></div></div>
  <span class="bar-value dark-label">{count}</span>
</div>"#,
            full = html_escape(label),
            short = html_escape(&truncate_str(label, 22)),
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
        let (badge_class, badge_text) = match issue.severity {
            Severity::Critical => ("badge-critical", "CRITICAL"),
            Severity::Warning => ("badge-warning", "WARNING"),
            Severity::Info => ("badge-info", "INFO"),
        };

        let steps_html: String = issue
            .solution
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return String::new();
                }
                format!("<div class=\"step\">{}</div>", html_escape(trimmed))
            })
            .collect();

        items.push_str(&format!(
            r#"<div class="minor-item">
  <details>
    <summary>
      <span class="badge {badge_class} badge-sm">{badge_text}</span>
      <span class="minor-title">{title}</span>
    </summary>
    <div class="minor-body">
      <p>{desc}</p>
      <div class="action-box">
        <div class="action-label">SUGGESTION</div>
        <div class="action-steps">{steps_html}</div>
      </div>
    </div>
  </details>
</div>"#,
            title = html_escape(&issue.title),
            desc = html_escape(&issue.description),
        ));
    }
    format!(
        r#"<section class="section-dark">
  <div class="container">
    <div class="section-label">OTHER ISSUES</div>
    <h2>其他问题</h2>
    <div class="minor-stack">{items}</div>
  </div>
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

fn render_map_inline(map: &std::collections::HashMap<String, usize>) -> String {
    let items: Vec<_> = map.iter().map(|(k, v)| format!("{k} ({v})")).collect();
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
/* ── Reset ─────────────────────────────────── */
*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

/* ── Hide scrollbar, keep scroll ───────────── */
html { overflow-y: scroll; scrollbar-width: none; }
html::-webkit-scrollbar { display: none; }

/* ── Base ──────────────────────────────────── */
body {
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Arial, sans-serif;
  background: #000;
  color: #fff;
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}

.page { overflow: hidden; }

.container {
  max-width: 960px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ── Typography ────────────────────────────── */
h2 {
  font-size: 26px;
  font-weight: 500;
  color: #fff;
  margin-bottom: 32px;
  line-height: 1.2;
}

h2.heading-dark { color: #181818; }

h3 {
  font-size: 13px;
  font-weight: 600;
  color: #8F8F8F;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 16px;
}

h3.h3-dark { color: #666; }

.section-label {
  font-size: 11px;
  font-weight: 400;
  color: #8F8F8F;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 8px;
}

/* ── Sections ──────────────────────────────── */
.section-dark {
  background: #000;
  padding: 64px 0;
}

.section-light {
  background: #fff;
  padding: 64px 0;
}

.section-light .section-label { color: #969696; }

/* ── Hero ──────────────────────────────────── */
.hero {
  background: #000;
  padding: 80px 0 48px;
  text-align: center;
}

.hero-inner { max-width: 960px; margin: 0 auto; padding: 0 24px; }

.hero-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 2px;
  color: #DA291C;
  border: 1px solid #DA291C;
  border-radius: 2px;
  padding: 4px 16px;
  margin-bottom: 24px;
}

.hero h1 {
  font-size: 36px;
  font-weight: 500;
  color: #fff;
  line-height: 1.15;
  margin-bottom: 12px;
}

.hero-sub {
  font-size: 13px;
  color: #8F8F8F;
  letter-spacing: 0.5px;
}

/* ── Stats Bar ─────────────────────────────── */
.stats-bar {
  background: #303030;
  padding: 32px 0;
  border-top: 1px solid #444;
  border-bottom: 1px solid #444;
}

.stats-row {
  display: flex;
  justify-content: space-between;
  text-align: center;
}

.stat-item { flex: 1; }

.stat-value {
  display: block;
  font-size: 28px;
  font-weight: 700;
  color: #fff;
  line-height: 1;
  margin-bottom: 6px;
  font-variant-numeric: tabular-nums;
}

.stat-value.accent-red { color: #DA291C; }
.stat-value.accent-amber { color: #F6E500; }

.stat-name {
  font-size: 11px;
  font-weight: 400;
  color: #8F8F8F;
  letter-spacing: 1px;
  text-transform: uppercase;
}

/* ── Issues ─────────────────────────────────── */
.issues-stack { display: flex; flex-direction: column; gap: 20px; }

.issue {
  background: #181818;
  border-radius: 2px;
  padding: 32px;
  border-left: 3px solid #DA291C;
}

.issue-top {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.issue-num {
  font-size: 24px;
  font-weight: 700;
  color: #303030;
}

.badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 3px 10px;
  border-radius: 2px;
}

.badge-sm { font-size: 10px; padding: 2px 8px; }

.badge-critical { background: #DA291C; color: #fff; }
.badge-warning { background: #F6E500; color: #181818; }
.badge-info { background: #4C98B9; color: #fff; }

.issue-title {
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  line-height: 1.3;
  margin-bottom: 8px;
}

.issue-desc {
  font-size: 14px;
  color: #8F8F8F;
  line-height: 1.7;
  margin-bottom: 20px;
}

/* ── Action Box ────────────────────────────── */
.action-box {
  background: #000;
  border: 1px solid #303030;
  border-radius: 2px;
  padding: 20px 24px;
}

.action-label {
  font-size: 11px;
  font-weight: 700;
  color: #DA291C;
  letter-spacing: 1px;
  margin-bottom: 12px;
}

.action-steps { display: flex; flex-direction: column; gap: 6px; }

.step {
  font-size: 14px;
  color: #D2D2D2;
  line-height: 1.6;
  padding-left: 8px;
  border-left: 2px solid #303030;
}

/* ── Info Cards ─────────────────────────────── */
.grid-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.grid-2 {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.info-card {
  background: #f7f7f7;
  border-radius: 2px;
  overflow: hidden;
}

.info-card.accent-border { border-top: 3px solid #DA291C; }

.card-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #969696;
  padding: 16px 20px 0;
}

.card-body { padding: 12px 20px 20px; }

.kv {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #e5e5e5;
  font-size: 13px;
}

.kv:last-child { border-bottom: none; }

.k { color: #8F8F8F; }
.v { color: #181818; text-align: right; }
.v.strong { font-weight: 700; color: #181818; }

.dns-item {
  font-family: "SF Mono", "Fira Code", ui-monospace, monospace;
  font-size: 12px;
  color: #666;
  padding: 4px 0;
  border-bottom: 1px solid #e5e5e5;
}

.dns-item:last-child { border-bottom: none; }

/* ── Panels (dark bg) ──────────────────────── */
.panel {
  background: #181818;
  border-radius: 2px;
  padding: 24px;
}

.panel-light {
  background: #f7f7f7;
  border-radius: 2px;
  padding: 24px;
}

.detail-table {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #e5e5e5;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  font-size: 13px;
}

.detail-key { color: #8F8F8F; }
.detail-val { color: #181818; text-align: right; }

/* ── Bar Chart ─────────────────────────────── */
.bar-chart { display: flex; flex-direction: column; gap: 8px; }

.bar-row {
  display: grid;
  grid-template-columns: 150px 1fr 44px;
  align-items: center;
  gap: 10px;
}

.bar-label {
  font-size: 12px;
  color: #8F8F8F;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bar-label.dark-label { color: #666; }

.bar-track {
  height: 18px;
  background: #303030;
  border-radius: 2px;
  overflow: hidden;
}

.bar-track.light-track { background: #e0e0e0; }

.bar-fill {
  height: 100%;
  border-radius: 2px;
  min-width: 3px;
}

.bar-value {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.bar-value.dark-label { color: #181818; }

/* ── Minor Issues ──────────────────────────── */
.minor-stack { display: flex; flex-direction: column; gap: 4px; }

.minor-item {
  background: #181818;
  border-radius: 2px;
  overflow: hidden;
}

.minor-item summary {
  padding: 14px 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: #D2D2D2;
  list-style: none;
}

.minor-item summary::-webkit-details-marker { display: none; }
.minor-item summary::before {
  content: "+";
  font-size: 16px;
  font-weight: 700;
  color: #666;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}
.minor-item details[open] summary::before { content: "−"; }

.minor-item summary:hover { background: #222; }

.minor-title { flex: 1; }

.minor-body {
  padding: 0 20px 20px 48px;
}

.minor-body p {
  font-size: 13px;
  color: #8F8F8F;
  line-height: 1.7;
  margin-bottom: 16px;
}

/* ── Log Panel ─────────────────────────────── */
.log-panel {
  background: #181818;
  border-radius: 2px;
  overflow: hidden;
}

.log-header {
  padding: 12px 20px;
  font-size: 12px;
  color: #8F8F8F;
  letter-spacing: 0.5px;
  border-bottom: 1px solid #303030;
}

.log-body {
  max-height: 360px;
  overflow-y: auto;
  scrollbar-width: none;
  padding: 12px 20px;
}

.log-body::-webkit-scrollbar { display: none; }

.log-line {
  font-family: "SF Mono", "Fira Code", ui-monospace, monospace;
  font-size: 11px;
  color: #8F8F8F;
  padding: 3px 0;
  border-bottom: 1px solid rgba(255,255,255,0.03);
  word-break: break-all;
  line-height: 1.5;
}

/* ── Footer ────────────────────────────────── */
footer {
  background: #303030;
  padding: 24px 0;
  text-align: center;
  font-size: 12px;
  color: #8F8F8F;
  letter-spacing: 0.5px;
}

/* ── Responsive ────────────────────────────── */
@media (max-width: 768px) {
  .hero h1 { font-size: 26px; }
  .hero { padding: 48px 0 32px; }
  .section-dark, .section-light { padding: 40px 0; }
  h2 { font-size: 22px; margin-bottom: 24px; }
  .grid-3 { grid-template-columns: 1fr; }
  .grid-2 { grid-template-columns: 1fr; }
  .stats-row { flex-wrap: wrap; gap: 16px; }
  .stat-item { flex: 0 0 calc(33.33% - 12px); }
  .stat-value { font-size: 22px; }
  .issue { padding: 20px; }
  .bar-row { grid-template-columns: 100px 1fr 36px; }
  .action-box { padding: 16px; }
}
"#;
