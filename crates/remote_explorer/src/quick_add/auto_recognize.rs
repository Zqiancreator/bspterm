//! Auto-recognition of connection strings for quick add functionality.
//!
//! This module provides a wrapper around the configurable recognition system
//! from the terminal crate, plus the UI components for the auto-recognize section.

use editor::Editor;
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window};
use i18n::t;
use terminal::{
    ParsedConnection as TerminalParsedConnection, RecognizeConfigEntity,
    RecognizeConnectionProtocol,
};
use ui::{prelude::*, h_flex, v_flex, Color, Icon, IconName, IconSize, Label, LabelSize};

/// Connection protocol for session creation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Ssh,
    Telnet,
}

impl From<RecognizeConnectionProtocol> for ConnectionProtocol {
    fn from(protocol: RecognizeConnectionProtocol) -> Self {
        match protocol {
            RecognizeConnectionProtocol::Ssh => ConnectionProtocol::Ssh,
            RecognizeConnectionProtocol::Telnet => ConnectionProtocol::Telnet,
        }
    }
}

/// A parsed connection from the input text.
#[derive(Clone, Debug)]
pub struct ParsedConnection {
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub protocol: ConnectionProtocol,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl From<TerminalParsedConnection> for ParsedConnection {
    fn from(conn: TerminalParsedConnection) -> Self {
        Self {
            name: conn.name,
            host: conn.host,
            port: conn.port,
            protocol: conn.protocol.into(),
            username: conn.username,
            password: conn.password,
        }
    }
}

impl ParsedConnection {
    pub fn telnet(host: String, port: u16) -> Self {
        Self {
            name: None,
            host,
            port,
            protocol: ConnectionProtocol::Telnet,
            username: None,
            password: None,
        }
    }

    pub fn telnet_with_credentials(
        host: String,
        port: u16,
        username: String,
        password: String,
    ) -> Self {
        Self {
            name: None,
            host,
            port,
            protocol: ConnectionProtocol::Telnet,
            username: Some(username),
            password: Some(password),
        }
    }

    pub fn ssh(host: String, port: u16) -> Self {
        Self {
            name: None,
            host,
            port,
            protocol: ConnectionProtocol::Ssh,
            username: None,
            password: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }
}

pub struct AutoRecognizeSection {
    editor: Entity<Editor>,
}

impl AutoRecognizeSection {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text(&t("remote_explorer.auto_recognize_hint"), window, cx);
            editor
        });

        Self { editor }
    }

    pub fn get_input(&self, cx: &App) -> String {
        self.editor.read(cx).text(cx)
    }

    pub fn clear_input(&mut self, window: &mut Window, cx: &mut App) {
        self.editor.update(cx, |editor, cx| {
            editor.set_text("", window, cx);
        });
    }

    pub fn editor(&self) -> &Entity<Editor> {
        &self.editor
    }

    pub fn render(&self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w_full()
            .gap_1()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Icon::new(IconName::MagnifyingGlass)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(t("remote_explorer.auto_recognize"))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(theme.colors().border)
                    .rounded_sm()
                    .px_1()
                    .py_px()
                    .child(self.editor.clone()),
            )
            .child(
                Label::new(t("remote_explorer.auto_recognize_hint"))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
    }
}

/// Parse connection text using the configured recognition rules.
///
/// This function uses the global `RecognizeConfigEntity` to parse the input text
/// with configurable rules. If the config entity is not initialized, it falls back
/// to using a default configuration.
pub fn parse_connection_text(input: &str, cx: &App) -> Vec<ParsedConnection> {
    if let Some(config_entity) = RecognizeConfigEntity::try_global(cx) {
        config_entity
            .read(cx)
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect()
    } else {
        // Fallback: create a temporary config entity with defaults
        let temp_entity = terminal::recognize_config::RecognizeConfigEntity::new_with_defaults();
        temp_entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect()
    }
}

/// Check if the input appears to be in session env info format (环境/后台 prefix).
pub fn is_session_env_info_format(input: &str, cx: &App) -> bool {
    if let Some(config_entity) = RecognizeConfigEntity::try_global(cx) {
        let config = config_entity.read(cx).config();
        input.lines().any(|line| {
            let trimmed = line.trim();
            config
                .protocol_prefixes
                .iter()
                .any(|prefix| trimmed.starts_with(&prefix.prefix))
        })
    } else {
        // Fallback: check common prefixes
        input.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("环境") || trimmed.starts_with("后台")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal::recognize_config::RecognizeConfigEntity as TerminalRecognizeConfigEntity;

    fn create_test_entity() -> TerminalRecognizeConfigEntity {
        TerminalRecognizeConfigEntity::new_with_defaults()
    }

    #[test]
    fn test_parse_single_ip() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 23);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);
    }

    #[test]
    fn test_parse_ip_with_port() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1:22")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 22);
        assert_eq!(result[0].protocol, ConnectionProtocol::Ssh);
    }

    #[test]
    fn test_parse_multiple_ports() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1:2323 2222")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 2323);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);

        assert_eq!(result[1].host, "192.168.1.1");
        assert_eq!(result[1].port, 2222);
        assert_eq!(result[1].protocol, ConnectionProtocol::Telnet);
    }

    #[test]
    fn test_parse_ip_with_credentials() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1 admin password123")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 23);
        assert_eq!(result[0].username, Some("admin".to_string()));
        assert_eq!(result[0].password, Some("password123".to_string()));
    }

    #[test]
    fn test_parse_ip_with_credentials_and_port() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1 admin password123 2323")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        // When there's a single port after credentials, it creates a connection for that port
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].port, 2323);
        assert_eq!(result[0].username, Some("admin".to_string()));
        assert_eq!(result[0].password, Some("password123".to_string()));
    }

    #[test]
    fn test_parse_multiple_ips_comma() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1, 192.168.1.2")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[1].host, "192.168.1.2");
    }

    #[test]
    fn test_parse_multiple_ips_newline() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1\n192.168.1.2")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[1].host, "192.168.1.2");
    }

    #[test]
    fn test_parse_empty() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_session_env_info_telnet() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("环境192.168.1.1\troot\tpassword")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 23);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);
        assert_eq!(result[0].username, Some("root".to_string()));
        assert_eq!(result[0].password, Some("password".to_string()));
    }

    #[test]
    fn test_parse_session_env_info_ssh() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("后台192.168.1.1\tadmin\tsecret")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 22);
        assert_eq!(result[0].protocol, ConnectionProtocol::Ssh);
        assert_eq!(result[0].username, Some("admin".to_string()));
        assert_eq!(result[0].password, Some("secret".to_string()));
    }

    #[test]
    fn test_parse_session_env_info_with_port() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("环境192.168.1.1:2323\troot\tpassword")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 2323);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);
    }

    #[test]
    fn test_parse_session_env_info_multiple() {
        let entity = create_test_entity();
        let input =
            "环境192.168.1.1\troot\tpass1\n后台192.168.1.1\tadmin\tpass2\n环境192.168.1.2\troot\tpass3";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].port, 23);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);

        assert_eq!(result[1].host, "192.168.1.1");
        assert_eq!(result[1].port, 22);
        assert_eq!(result[1].protocol, ConnectionProtocol::Ssh);

        assert_eq!(result[2].host, "192.168.1.2");
        assert_eq!(result[2].port, 23);
        assert_eq!(result[2].protocol, ConnectionProtocol::Telnet);
    }

    #[test]
    fn test_parse_with_name_prefix() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("管理网口127.0.0.1")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert!(result[0].name.as_ref().unwrap().contains("管理网口"));
        assert_eq!(result[0].host, "127.0.0.1");
        assert_eq!(result[0].port, 23);
        assert_eq!(result[0].protocol, ConnectionProtocol::Telnet);
    }

    #[test]
    fn test_parse_slash_separator() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1 user/pass")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].username, Some("user".to_string()));
        assert_eq!(result[0].password, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_mixed_format() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("管理网口127.0.0.1 root123/Root@123")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert!(result[0].name.as_ref().unwrap().contains("管理网口"));
        assert_eq!(result[0].host, "127.0.0.1");
        assert_eq!(result[0].username, Some("root123".to_string()));
        assert_eq!(result[0].password, Some("Root@123".to_string()));
    }

    #[test]
    fn test_multiline_ip_name_user_pass() {
        let entity = create_test_entity();
        let input = "6.6.62.23 slot23\nhuawei\nRouter@202508";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "6.6.62.23");
        assert!(result[0].name.as_ref().unwrap().contains("slot23"));
        assert_eq!(result[0].username, Some("huawei".to_string()));
        assert_eq!(result[0].password, Some("Router@202508".to_string()));
    }

    #[test]
    fn test_smart_credential_detection() {
        let entity = create_test_entity();
        let input = "6.6.62.23 root123 Root@123 slot23";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "6.6.62.23");
        assert_eq!(result[0].username, Some("root123".to_string()));
        assert_eq!(result[0].password, Some("Root@123".to_string()));
        assert!(result[0].name.as_ref().unwrap().contains("slot23"));
    }

    #[test]
    fn test_chinese_prefix_with_credentials() {
        let entity = create_test_entity();
        let input = "管理网口192.168.1.1 huawei Admin@123";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert!(result[0].name.as_ref().unwrap().contains("管理网口"));
        assert_eq!(result[0].host, "192.168.1.1");
        assert_eq!(result[0].username, Some("huawei".to_string()));
        assert_eq!(result[0].password, Some("Admin@123".to_string()));
    }

    #[test]
    fn test_multiline_debug_user_password() {
        let entity = create_test_entity();
        let input = "8.84.66.6\ndebug123\nRoot@123";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].host, "8.84.66.6");
        assert_eq!(result[0].username, Some("debug123".to_string()));
        assert_eq!(result[0].password, Some("Root@123".to_string()));
    }

    #[test]
    fn test_single_line_debug_user_password() {
        let entity = create_test_entity();
        let input = "8.84.66.6 debug123 Root@123";
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text(input)
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].username, Some("debug123".to_string()));
        assert_eq!(result[0].password, Some("Root@123".to_string()));
    }

    #[test]
    fn test_multiple_ports_share_credentials() {
        let entity = create_test_entity();
        let result: Vec<ParsedConnection> = entity
            .parse_connection_text("192.168.1.1:2323 2222 root Admin@123")
            .into_iter()
            .map(ParsedConnection::from)
            .collect();

        assert_eq!(result.len(), 2);
        // Both connections should have the same credentials
        assert_eq!(result[0].username, Some("root".to_string()));
        assert_eq!(result[0].password, Some("Admin@123".to_string()));
        assert_eq!(result[1].username, Some("root".to_string()));
        assert_eq!(result[1].password, Some("Admin@123".to_string()));
    }
}
