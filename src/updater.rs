use serde::Deserialize;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum UpdateCheckResult {
    UpdateAvailable {
        version: String,
        url: String,
        notes: String,
    },
    UpToDate,
    NoReleasesFound,
    Error(String),
}

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn check_for_updates() -> UpdateCheckResult {
    let url = "https://api.github.com/repos/kosherplay-betatester/LangSwitch/releases/latest";

    #[cfg(windows)]
    let body_str = {
        let mut response = match ureq::get(url)
            .header("User-Agent", "LangSwitch-App")
            .header("Accept", "application/vnd.github.v3+json")
            .call()
        {
            Ok(res) => res,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("404") {
                    return UpdateCheckResult::NoReleasesFound;
                }
                return UpdateCheckResult::Error(format!("Network error: {}", e));
            }
        };

        let mut body = String::new();
        if let Err(e) = response.body_mut().as_reader().read_to_string(&mut body) {
            return UpdateCheckResult::Error(format!("Failed reading response: {}", e));
        }
        body
    };

    #[cfg(not(windows))]
    let body_str = {
        let output = match std::process::Command::new("curl")
            .args(["-sSL", "-H", "User-Agent: LangSwitch-App", "-H", "Accept: application/vnd.github.v3+json", url])
            .output()
        {
            Ok(out) => out,
            Err(e) => return UpdateCheckResult::Error(format!("Failed running curl: {}", e)),
        };

        if !output.status.success() {
            return UpdateCheckResult::Error("Curl request failed".to_string());
        }

        match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(e) => return UpdateCheckResult::Error(format!("UTF-8 error: {}", e)),
        }
    };

    let release: GithubRelease = match serde_json::from_str(&body_str) {
        Ok(r) => r,
        Err(e) => return UpdateCheckResult::Error(format!("Failed parsing release info: {}", e)),
    };

    let latest_tag = release.tag_name.trim_start_matches('v').trim();
    let current_ver = CURRENT_VERSION.trim_start_matches('v').trim();

    if is_newer_version(latest_tag, current_ver) {
        UpdateCheckResult::UpdateAvailable {
            version: release.tag_name,
            url: release.html_url,
            notes: release.body.unwrap_or_default(),
        }
    } else {
        UpdateCheckResult::UpToDate
    }
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .map(|part| part.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let latest_nums = parse(latest);
    let current_nums = parse(current);

    latest_nums > current_nums
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert!(is_newer_version("1.0.0", "0.1.0"));
        assert!(is_newer_version("0.2.0", "0.1.9"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.2.0"));
    }
}
