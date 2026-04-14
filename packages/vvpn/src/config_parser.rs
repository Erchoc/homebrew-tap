use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ConfigReport {
    pub subscription_url: Option<String>,
    pub subscription_provider: Option<String>,
    pub proxy_count: usize,
    pub group_count: usize,
    pub rule_count: usize,
    /// region -> count
    pub nodes_by_region: HashMap<String, usize>,
    /// tier (标准/高级/实验性) -> count
    pub nodes_by_tier: HashMap<String, usize>,
    /// protocol type -> count
    pub nodes_by_type: HashMap<String, usize>,
    /// cipher -> count
    pub nodes_by_cipher: HashMap<String, usize>,
    /// proxy group names
    pub group_names: Vec<String>,
    /// traffic info
    pub traffic_info: Option<String>,
    /// expiry info
    pub expiry_info: Option<String>,
    /// port config
    pub ports: HashMap<String, u16>,
    /// dns servers
    pub dns_servers: Vec<String>,
    /// mode
    pub mode: String,
    /// ipv6 enabled
    pub ipv6: bool,
    /// allow-lan
    pub allow_lan: bool,
    /// server addresses (unique)
    pub unique_servers: Vec<String>,
    /// all cipher are same?
    pub single_cipher: bool,
    /// all password are same?
    pub single_password: bool,
}

pub fn parse_config(stash_dir: &str) -> ConfigReport {
    let config_path = Path::new(stash_dir).join("config.yaml");
    let content = fs::read_to_string(&config_path).unwrap_or_default();

    let mut report = ConfigReport {
        subscription_url: None,
        subscription_provider: None,
        proxy_count: 0,
        group_count: 0,
        rule_count: 0,
        nodes_by_region: HashMap::new(),
        nodes_by_tier: HashMap::new(),
        nodes_by_type: HashMap::new(),
        nodes_by_cipher: HashMap::new(),
        group_names: Vec::new(),
        traffic_info: None,
        expiry_info: None,
        ports: HashMap::new(),
        dns_servers: Vec::new(),
        mode: String::new(),
        ipv6: false,
        allow_lan: false,
        unique_servers: Vec::new(),
        single_cipher: true,
        single_password: true,
    };

    // Extract subscription URL from comment
    for line in content.lines() {
        if line.starts_with("#SUBSCRIBED") {
            let url = line.trim_start_matches("#SUBSCRIBED").trim().to_string();
            report.subscription_provider = extract_provider(&url);
            report.subscription_url = Some(url);
            break;
        }
    }

    // Parse YAML
    let yaml: Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return report,
    };

    // Basic settings
    if let Some(mode) = yaml.get("mode").and_then(|v| v.as_str()) {
        report.mode = mode.to_string();
    }
    report.ipv6 = yaml
        .get("ipv6")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    report.allow_lan = yaml
        .get("allow-lan")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Ports
    for key in &["port", "socks-port", "redir-port", "mixed-port"] {
        if let Some(p) = yaml.get(*key).and_then(|v| v.as_u64()) {
            report.ports.insert(key.to_string(), p as u16);
        }
    }

    // DNS
    if let Some(dns) = yaml.get("dns") {
        if let Some(servers) = dns.get("nameserver").and_then(|v| v.as_sequence()) {
            for s in servers {
                if let Some(sv) = s.as_str() {
                    report.dns_servers.push(sv.to_string());
                }
            }
        }
    }

    // Proxies
    let mut ciphers = std::collections::HashSet::new();
    let mut passwords = std::collections::HashSet::new();
    let mut servers = std::collections::HashSet::new();

    if let Some(proxies) = yaml.get("proxies").and_then(|v| v.as_sequence()) {
        report.proxy_count = proxies.len();

        for proxy in proxies {
            let name = proxy.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let ptype = proxy
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let cipher = proxy
                .get("cipher")
                .and_then(|v| v.as_str())
                .unwrap_or("none");
            let password = proxy
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let server = proxy
                .get("server")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Traffic / Expiry info nodes
            if name.starts_with("Traffic:") {
                report.traffic_info = Some(name.to_string());
                continue;
            }
            if name.starts_with("Expire:") {
                report.expiry_info = Some(name.to_string());
                continue;
            }

            // Region extraction
            let region = extract_region(name);
            *report.nodes_by_region.entry(region).or_insert(0) += 1;

            // Tier extraction
            let tier = extract_tier(name);
            *report.nodes_by_tier.entry(tier).or_insert(0) += 1;

            *report
                .nodes_by_type
                .entry(ptype.to_uppercase())
                .or_insert(0) += 1;
            *report
                .nodes_by_cipher
                .entry(cipher.to_string())
                .or_insert(0) += 1;

            ciphers.insert(cipher.to_string());
            passwords.insert(password.to_string());
            if !server.is_empty() {
                servers.insert(server.to_string());
            }
        }
    }

    report.single_cipher = ciphers.len() <= 1;
    report.single_password = passwords.len() <= 1;
    report.unique_servers = servers.into_iter().collect();
    report.unique_servers.sort();

    // Proxy groups
    if let Some(groups) = yaml.get("proxy-groups").and_then(|v| v.as_sequence()) {
        report.group_count = groups.len();
        for g in groups {
            if let Some(name) = g.get("name").and_then(|v| v.as_str()) {
                report.group_names.push(name.to_string());
            }
        }
    }

    // Rules
    if let Some(rules) = yaml.get("rules").and_then(|v| v.as_sequence()) {
        report.rule_count = rules.len();
    }

    report
}

fn extract_region(name: &str) -> String {
    let regions = [
        ("香港", "香港"),
        ("日本", "日本"),
        ("新加坡", "新加坡"),
        ("台湾", "台湾"),
        ("美国", "美国"),
        ("韩国", "韩国"),
        ("德国", "德国"),
        ("英国", "英国"),
        ("法国", "法国"),
        ("荷兰", "荷兰"),
        ("加拿大", "加拿大"),
        ("澳大利亚", "澳大利亚"),
        ("印度", "印度"),
        ("摩尔多瓦", "摩尔多瓦"),
        ("乌克兰", "乌克兰"),
        ("意大利", "意大利"),
        ("匈牙利", "匈牙利"),
        ("西班牙", "西班牙"),
        ("土耳其", "土耳其"),
        ("阿根廷", "阿根廷"),
        ("巴西", "巴西"),
        ("智利", "智利"),
        ("新西兰", "新西兰"),
        ("印尼", "印尼"),
        ("泰国", "泰国"),
        ("越南", "越南"),
        ("巴基斯坦", "巴基斯坦"),
        ("以色列", "以色列"),
        ("阿联酋", "阿联酋"),
        ("菲律宾", "菲律宾"),
        ("马来西亚", "马来西亚"),
        ("埃及", "埃及"),
        ("尼日利亚", "尼日利亚"),
    ];

    for (keyword, region) in &regions {
        if name.contains(keyword) {
            return region.to_string();
        }
    }
    "其他".to_string()
}

fn extract_tier(name: &str) -> String {
    if name.contains("高级") {
        "高级".to_string()
    } else if name.contains("实验性") {
        "实验性".to_string()
    } else if name.contains("标准") {
        "标准".to_string()
    } else {
        "其他".to_string()
    }
}

fn extract_provider(url: &str) -> Option<String> {
    if url.contains("huacloud") || url.contains("233netboom") || url.contains("xmancdn") {
        Some("233netboom (花云)".to_string())
    } else if url.contains("sub-store") {
        Some("Sub-Store".to_string())
    } else {
        // Try to extract domain
        url.split("//")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .map(|s| s.to_string())
    }
}
