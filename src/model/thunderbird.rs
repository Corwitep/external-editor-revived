use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub trait EmailHeaderValue {
    fn to_header_value(&self) -> Result<String>;
    fn from_header_value(value: &str) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tab {
    pub id: i32,
    pub index: i32,
    #[serde(rename = "windowId")]
    pub window_id: i32,
    #[serde(default)]
    pub highlighted: bool,
    #[serde(default)]
    pub active: bool,
    pub status: TabStatus,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "type")]
    pub tab_type: TabType,
    #[serde(rename = "mailTab")]
    pub mail_tab: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TabStatus {
    Loading,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TabType {
    AddressBook,
    Calendar,
    CalendarEvent,
    CalendarTask,
    Chat,
    Content,
    Mail,
    MessageCompose,
    MessageDisplay,
    Special,
    Tasks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComposeDetails {
    pub from: ComposeRecipient,
    pub to: ComposeRecipientList,
    pub cc: ComposeRecipientList,
    pub bcc: ComposeRecipientList,
    #[serde(rename = "type")]
    pub compose_type: ComposeType,
    #[serde(rename = "relatedMessageId", skip_serializing_if = "Option::is_none")]
    pub related_message_id: Option<i32>,
    #[serde(rename = "replyTo")]
    pub reply_to: ComposeRecipientList,
    #[serde(rename = "followupTo")]
    pub follow_up_to: ComposeRecipientList,
    pub newsgroups: Newsgroups,
    pub subject: String,
    #[serde(rename = "isPlainText")]
    pub is_plain_text: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub body: String,
    #[serde(rename = "plainTextBody", skip_serializing_if = "String::is_empty")]
    pub plain_text_body: String,
    pub attachments: Vec<ComposeAttachment>,
}

impl ComposeDetails {
    #[cfg(not(target_os = "windows"))]
    pub fn get_body(&self) -> String {
        if self.is_plain_text {
            self.plain_text_body.replace('\n', "\r\n")
        } else {
            self.body.replace('\n', "\r\n")
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_body(&self) -> &str {
        // Thunderbird under Windows already sends CRLF
        if self.is_plain_text {
            &self.plain_text_body
        } else {
            &self.body
        }
    }

    pub fn set_body(&mut self, body: String) {
        if self.is_plain_text {
            self.plain_text_body = body;
        } else {
            self.body = body;
        }
    }

    /// Reset all ComposeRecipientList fields to empty ComposeRecipientList::Multiple
    pub fn clear_recipients(&mut self) {
        self.to = ComposeRecipientList::Multiple(Vec::new());
        self.cc = ComposeRecipientList::Multiple(Vec::new());
        self.bcc = ComposeRecipientList::Multiple(Vec::new());
        self.reply_to = ComposeRecipientList::Multiple(Vec::new());
    }

    pub fn add_to(&mut self, recipient: ComposeRecipient) {
        match &mut self.to {
            ComposeRecipientList::Single(r) => {
                self.to = ComposeRecipientList::Multiple(vec![r.clone(), recipient]);
            }
            ComposeRecipientList::Multiple(l) => l.push(recipient),
        }
    }

    pub fn add_cc(&mut self, recipient: ComposeRecipient) {
        match &mut self.cc {
            ComposeRecipientList::Single(r) => {
                self.cc = ComposeRecipientList::Multiple(vec![r.clone(), recipient]);
            }
            ComposeRecipientList::Multiple(l) => l.push(recipient),
        }
    }

    pub fn add_bcc(&mut self, recipient: ComposeRecipient) {
        match &mut self.bcc {
            ComposeRecipientList::Single(r) => {
                self.bcc = ComposeRecipientList::Multiple(vec![r.clone(), recipient]);
            }
            ComposeRecipientList::Multiple(l) => l.push(recipient),
        }
    }

    pub fn add_reply_to(&mut self, recipient: ComposeRecipient) {
        match &mut self.reply_to {
            ComposeRecipientList::Single(r) => {
                self.reply_to = ComposeRecipientList::Multiple(vec![r.clone(), recipient]);
            }
            ComposeRecipientList::Multiple(l) => l.push(recipient),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComposeType {
    Draft,
    New,
    Redirect,
    Reply,
    Forward,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComposeAttachment {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub size: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComposeRecipientNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: ComposeRecipientNodeType,
}

impl EmailHeaderValue for ComposeRecipientNode {
    fn to_header_value(&self) -> Result<String> {
        let value = serde_json::to_string_pretty(&self)?;
        Ok(value.replace(&['\n', '\r'], ""))
    }

    fn from_header_value(value: &str) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(serde_json::from_str(value)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComposeRecipientNodeType {
    Contact,
    MailingList,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ComposeRecipient {
    Email(String),
    Node(ComposeRecipientNode),
}

impl EmailHeaderValue for ComposeRecipient {
    fn to_header_value(&self) -> Result<String> {
        match &self {
            Self::Email(email) => Ok(email.to_owned()),
            Self::Node(node) => node.to_header_value(),
        }
    }

    fn from_header_value(value: &str) -> Result<Self> {
        if let Some(first_char) = value.chars().next() {
            if first_char == '{' {
                Ok(ComposeRecipient::Node(
                    ComposeRecipientNode::from_header_value(value)?,
                ))
            } else {
                Ok(ComposeRecipient::Email(value.to_owned()))
            }
        } else {
            Err(anyhow!(
                "Failed to convert empty string to ComposeRecipient"
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ComposeRecipientList {
    Single(ComposeRecipient),
    Multiple(Vec<ComposeRecipient>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Newsgroups {
    Single(String),
    Multiple(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_recipient_list_single_email_serialisation_test() {
        let single_email = ComposeRecipientList::Single(ComposeRecipient::Email(
            "someone@example.com <John Smith>".to_owned(),
        ));
        assert_eq!(
            r#""someone@example.com <John Smith>""#,
            serde_json::to_string(&single_email).unwrap()
        );
    }

    #[test]
    fn compose_recipient_list_multiple_emails_serialisation_test() {
        let multiple_emails = ComposeRecipientList::Multiple(vec![
            ComposeRecipient::Email("someone@example.com <John Smith>".to_owned()),
            ComposeRecipient::Email("another@example.com <Jane Smith>".to_owned()),
        ]);
        assert_eq!(
            r#"["someone@example.com <John Smith>","another@example.com <Jane Smith>"]"#,
            serde_json::to_string(&multiple_emails).unwrap()
        );
    }

    #[test]
    fn compose_recipient_list_single_node_serialisation_test() {
        let single_node =
            ComposeRecipientList::Single(ComposeRecipient::Node(ComposeRecipientNode {
                id: "some_id".to_owned(),
                node_type: ComposeRecipientNodeType::Contact,
            }));
        assert_eq!(
            r#"{"id":"some_id","type":"contact"}"#,
            serde_json::to_string(&single_node).unwrap()
        );
    }

    #[test]
    fn compose_recipient_list_multiple_node_serialisation_test() {
        let multiple_nodes = ComposeRecipientList::Multiple(vec![
            ComposeRecipient::Node(ComposeRecipientNode {
                id: "some_id".to_owned(),
                node_type: ComposeRecipientNodeType::Contact,
            }),
            ComposeRecipient::Node(ComposeRecipientNode {
                id: "another_id".to_owned(),
                node_type: ComposeRecipientNodeType::MailingList,
            }),
        ]);
        assert_eq!(
            r#"[{"id":"some_id","type":"contact"},{"id":"another_id","type":"mailingList"}]"#,
            serde_json::to_string(&multiple_nodes).unwrap()
        );
    }

    #[test]
    fn compose_recipient_list_multiple_composite_serialisation_test() {
        let composite = ComposeRecipientList::Multiple(vec![
            ComposeRecipient::Email("someone@example.com <John Smith>".to_owned()),
            ComposeRecipient::Node(ComposeRecipientNode {
                id: "another_id".to_owned(),
                node_type: ComposeRecipientNodeType::MailingList,
            }),
        ]);
        assert_eq!(
            r#"["someone@example.com <John Smith>",{"id":"another_id","type":"mailingList"}]"#,
            serde_json::to_string(&composite).unwrap()
        );
    }

    #[test]
    fn compose_recipient_list_multiple_composite_deserialisation_test() {
        let json =
            r#"["someone@example.com <John Smith>",{"id":"another_id","type":"mailingList"}]"#
                .to_owned();
        let composite = serde_json::from_str(&json).unwrap();
        match composite {
            ComposeRecipientList::Multiple(recipients) => {
                assert_eq!(2, recipients.len());
                assert_eq!(
                    ComposeRecipient::Email("someone@example.com <John Smith>".to_owned()),
                    recipients[0]
                );
                assert_eq!(
                    ComposeRecipient::Node(ComposeRecipientNode {
                        id: "another_id".to_owned(),
                        node_type: ComposeRecipientNodeType::MailingList
                    }),
                    recipients[1]
                );
            }
            _ => panic!("should not be ComposeRecipientList::Single"),
        }
    }

    #[test]
    fn compose_details_crlf_body_test() {
        let mut compose_details = get_blank_compose_details();
        compose_details.plain_text_body = if cfg!(target_os = "windows") {
            "Hello,\r\nworld!".to_owned()
        } else {
            "Hello,\nworld!".to_owned()
        };

        let body = compose_details.get_body();
        assert_eq!(1, body.matches("\r\n").count());
        assert_eq!(1, body.matches('\r').count());
        assert_eq!(1, body.matches('\n').count());
    }

    #[test]
    fn compose_details_add_recipient_to_single_test() {
        let mut compose_details = get_blank_compose_details();
        compose_details.add_to(ComposeRecipient::Email("hello@example.com".to_owned()));
        match compose_details.to {
            ComposeRecipientList::Single(_) => panic!("should not be ComposeRecipientList::Single"),
            ComposeRecipientList::Multiple(l) => {
                assert_eq!(2, l.len());
                assert_eq!(
                    ComposeRecipient::Email("someone@example.com".to_owned()),
                    l[0]
                );
                assert_eq!(
                    ComposeRecipient::Email("hello@example.com".to_owned()),
                    l[1]
                );
            }
        }
    }

    #[test]
    fn compose_details_add_recipient_to_multiple_test() {
        let mut compose_details = get_blank_compose_details();
        compose_details.add_cc(ComposeRecipient::Email("someone@example.com".to_owned()));
        compose_details.add_cc(ComposeRecipient::Email("hello@example.com".to_owned()));
        match compose_details.cc {
            ComposeRecipientList::Single(_) => panic!("should not be ComposeRecipientList::Single"),
            ComposeRecipientList::Multiple(l) => {
                assert_eq!(2, l.len());
                assert_eq!(
                    ComposeRecipient::Email("someone@example.com".to_owned()),
                    l[0]
                );
                assert_eq!(
                    ComposeRecipient::Email("hello@example.com".to_owned()),
                    l[1]
                );
            }
        }
    }

    fn get_blank_compose_details() -> ComposeDetails {
        ComposeDetails {
            from: ComposeRecipient::Email("someone@example.com".to_owned()),
            to: ComposeRecipientList::Single(ComposeRecipient::Email(
                "someone@example.com".to_owned(),
            )),
            cc: ComposeRecipientList::Multiple(Vec::new()),
            bcc: ComposeRecipientList::Multiple(Vec::new()),
            compose_type: ComposeType::New,
            related_message_id: None,
            reply_to: ComposeRecipientList::Multiple(Vec::new()),
            follow_up_to: ComposeRecipientList::Multiple(Vec::new()),
            newsgroups: Newsgroups::Multiple(Vec::new()),
            subject: "".to_owned(),
            is_plain_text: true,
            body: "".to_owned(),
            plain_text_body: "".to_owned(),
            attachments: Vec::new(),
        }
    }
}
