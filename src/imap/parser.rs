//! RFC822/MIME email parser using the `mail-parser` crate.

use mail_parser::MimeHeaders;

/// A parsed email ready for database insertion.
#[derive(Debug)]
pub struct ParsedEmail {
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub subject: String,
    pub from_address: String,
    pub from_name: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub date: Option<chrono::DateTime<chrono::Utc>>,
    pub is_read: bool,
    pub attachments: Vec<ParsedAttachment>,
}

/// A parsed email attachment.
#[derive(Debug)]
pub struct ParsedAttachment {
    pub file_name: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Parse a raw RFC822 email message into a `ParsedEmail`.
///
/// Returns None if the message is completely unparsable.
pub fn parse_rfc822(raw: &[u8], is_seen: bool) -> Option<ParsedEmail> {
    let message = mail_parser::MessageParser::default().parse(raw)?;

    let message_id = message.message_id().map(|s| s.to_string());

    // Threading: check In-Reply-To, then fall back to last entry in References
    let in_reply_to = message
        .in_reply_to()
        .as_text()
        .map(|s| s.to_string())
        .or_else(|| {
            message
                .references()
                .as_text_list()
                .and_then(|refs| refs.last().map(|s| s.to_string()))
        });

    let subject = message.subject().unwrap_or("(No Subject)").to_string();

    // Extract from address
    let (from_address, from_name) = message
        .from()
        .and_then(|addr| addr.first())
        .map(|a| {
            (
                a.address().unwrap_or("").to_string(),
                a.name().unwrap_or("").to_string(),
            )
        })
        .unwrap_or_default();

    let to = extract_addresses(message.to());
    let cc = extract_addresses(message.cc());
    let bcc = extract_addresses(message.bcc());

    let body_html = message.body_html(0).map(|s| s.to_string());
    let body_text = message.body_text(0).map(|s| s.to_string());

    let date = message
        .date()
        .and_then(|dt| chrono::DateTime::from_timestamp(dt.to_timestamp(), 0));

    let mut attachments = Vec::new();
    for part in message.attachments() {
        let file_name = part.attachment_name().unwrap_or("attachment").to_string();
        let content_type = part
            .content_type()
            .map(|ct| {
                if let Some(sub) = ct.subtype() {
                    format!("{}/{}", ct.ctype(), sub)
                } else {
                    ct.ctype().to_string()
                }
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let data = part.contents().to_vec();

        if !data.is_empty() {
            attachments.push(ParsedAttachment {
                file_name,
                content_type,
                data,
            });
        }
    }

    Some(ParsedEmail {
        message_id,
        in_reply_to,
        subject,
        from_address,
        from_name,
        to,
        cc,
        bcc,
        body_html,
        body_text,
        date,
        is_read: is_seen,
        attachments,
    })
}

/// Extract email addresses from an optional Address header.
fn extract_addresses(header: Option<&mail_parser::Address<'_>>) -> Vec<String> {
    match header {
        Some(addr) => addr
            .iter()
            .filter_map(|a| a.address().map(|s| s.to_string()))
            .collect(),
        None => vec![],
    }
}
