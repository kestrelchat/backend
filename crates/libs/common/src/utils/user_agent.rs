// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use once_cell::sync::Lazy;
use uaparser::{Parser, UserAgentParser};

const REGEXES: &[u8] = include_bytes!("../../assets/ua-regexes.yaml");

static PARSER: Lazy<UserAgentParser> =
    Lazy::new(|| UserAgentParser::from_bytes(REGEXES).expect("failed to load uaparser regexes"));

#[derive(Debug, Clone)]
pub struct UserAgent {
    pub browser_family: String,
    pub browser_version: Option<String>,
    pub os_family: String,
    pub os_version: Option<String>,
    pub device_family: String,
}

pub fn parse_user_agent(ua: &str) -> UserAgent {
    let result = PARSER.parse(ua);

    UserAgent {
        browser_family: result.user_agent.family.to_string(),
        browser_version: format_version(
            result.user_agent.major.as_deref(),
            result.user_agent.minor.as_deref(),
            result.user_agent.patch.as_deref(),
        ),
        os_family: result.os.family.to_string(),
        os_version: format_version(
            result.os.major.as_deref(),
            result.os.minor.as_deref(),
            result.os.patch.as_deref(),
        ),
        device_family: result.device.family.to_string(),
    }
}

fn format_version(major: Option<&str>, minor: Option<&str>, patch: Option<&str>) -> Option<String> {
    match (major, minor, patch) {
        (Some(a), Some(b), Some(c)) => Some(format!("{a}.{b}.{c}")),
        (Some(a), Some(b), None) => Some(format!("{a}.{b}")),
        (Some(a), None, None) => Some(a.to_string()),
        _ => None,
    }
}
