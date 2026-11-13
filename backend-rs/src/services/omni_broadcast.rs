//! Omni-Channel Broadcast System — Phase 46
//!
//! Enables designing broadcast messages with rich media (images, videos, link previews)
//! and sending them to all selected channels in one click. The system automatically
//! formats content per channel and distributes to pre-selected contacts/customers.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Broadcast {
    pub id: String,
    pub organization_id: String,
    pub brand_id: Option<String>,
    pub title: String,
    pub status: String,
    pub created_by: String,
    pub scheduled_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub subject_line: Option<String>,
    pub preview_text: Option<String>,
    pub media_type: String,
    pub primary_media_url: Option<String>,
    pub media_caption: Option<String>,
    pub link_url: Option<String>,
    pub link_title: Option<String>,
    pub link_description: Option<String>,
    pub link_image_url: Option<String>,
    pub target_type: String,
    pub target_ids: Value,
    pub channels: Value,
    pub channel_overrides: Value,
    pub total_recipients: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub failed_count: i32,
    pub read_count: i32,
    pub clicked_count: i32,
    pub replied_count: i32,
    pub send_delay_seconds: i32,
    pub retry_failed: bool,
    pub max_retries: i32,
    pub track_opens: bool,
    pub track_clicks: bool,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastMedia {
    pub id: String,
    pub broadcast_id: String,
    pub media_type: String,
    pub file_name: String,
    pub file_size_bytes: i64,
    pub mime_type: String,
    pub file_url: String,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: i32,
    pub width: i32,
    pub height: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastChannel {
    pub id: String,
    pub broadcast_id: String,
    pub channel_code: String,
    pub channel_name: String,
    pub status: String,
    pub formatted_body: Option<String>,
    pub formatted_media_url: Option<String>,
    pub formatted_subject: Option<String>,
    pub recipient_count: i32,
    pub sent_count: i32,
    pub failed_count: i32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastRecipient {
    pub id: String,
    pub broadcast_id: String,
    pub contact_id: String,
    pub channel_code: String,
    pub phone_number: Option<String>,
    pub email_address: Option<String>,
    pub external_id: Option<String>,
    pub status: String,
    pub message_id: Option<String>,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
    pub clicked_at: Option<String>,
    pub replied_at: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastLog {
    pub id: String,
    pub broadcast_id: String,
    pub event_type: String,
    pub channel_code: Option<String>,
    pub recipient_id: Option<String>,
    pub details: Value,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub max_recipients_per_broadcast: i32,
    pub default_send_delay_seconds: i32,
    pub rate_limit_per_minute: i32,
    pub allow_scheduling: bool,
    pub require_approval: bool,
    pub approved_by: Option<String>,
    pub auto_format_channels: bool,
    pub track_opens: bool,
    pub track_clicks: bool,
    pub default_channels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelFormat {
    pub channel_code: String,
    pub channel_name: String,
    pub formatted_body: String,
    pub formatted_media_url: Option<String>,
    pub formatted_subject: Option<String>,
    pub supports_html: bool,
    pub supports_images: bool,
    pub supports_video: bool,
    pub supports_links: bool,
    pub max_length: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastSummary {
    pub total_broadcasts: i32,
    pub draft_broadcasts: i32,
    pub sent_broadcasts: i32,
    pub failed_broadcasts: i32,
    pub scheduled_broadcasts: i32,
    pub total_recipients_sent: i32,
    pub total_delivered: i32,
    pub total_failed: i32,
    pub total_read: i32,
    pub total_clicked: i32,
    pub total_replied: i32,
    pub by_channel: Vec<ChannelCount>,
    pub recent_broadcasts: Vec<Broadcast>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelCount {
    pub channel_code: String,
    pub channel_name: String,
    pub broadcast_count: i32,
    pub recipient_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendBroadcastResult {
    pub broadcast_id: String,
    pub status: String,
    pub channels_sent: Vec<String>,
    pub total_recipients: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormatPreview {
    pub broadcast_id: String,
    pub formats: Vec<ChannelFormat>,
}

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

fn gen_id() -> String {
    Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// Channel formatting rules
// ---------------------------------------------------------------------------

/// Auto-format a broadcast message for a specific channel.
/// If `body_html` is present, it is used as the source and converted to the channel's format.
/// Otherwise `body_text` is used as fallback.
fn format_for_channel(broadcast: &Broadcast, channel: &str) -> ChannelFormat {
    // Prefer HTML source if available; otherwise fall back to body_text
    let source_html = broadcast.body_html.clone().or_else(|| {
        // Wrap plain text in minimal HTML so the converter handles it uniformly
        if !broadcast.body_text.is_empty() {
            Some(format!("<p>{}</p>", escape_html(&broadcast.body_text)))
        } else {
            None
        }
    });

    let source_text = if let Some(ref html) = source_html {
        html_to_plain_text(html)
    } else {
        broadcast.body_text.clone()
    };

    let mut media_url = broadcast.primary_media_url.clone();
    let mut subject = broadcast.subject_line.clone();

    match channel {
        "whatsapp" => {
            // WhatsApp: markdown (*bold*, _italic_, ~strikethrough~, ```code```)
            // Max 4096 chars, supports images and documents
            let body = if let Some(ref html) = source_html {
                html_to_whatsapp_markdown(html)
            } else {
                source_text
            };
            let body = body.chars().take(4096).collect::<String>();
            // Add link preview if URL present
            let body = if let Some(ref url) = broadcast.link_url {
                if !body.contains(url.as_str()) {
                    format!("{}\n\n{}", body, url)
                } else {
                    body
                }
            } else {
                body
            };
            ChannelFormat {
                channel_code: "whatsapp".to_string(),
                channel_name: "WhatsApp".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: false,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 4096,
            }
        }
        "email" => {
            // Email: full HTML support
            let html = if let Some(ref html) = broadcast.body_html {
                html.clone()
            } else if let Some(ref html) = source_html {
                html.clone()
            } else {
                // Convert plain text to basic HTML
                let mut html_body = source_text.replace('\n', "<br>");
                if let Some(ref caption) = broadcast.media_caption {
                    html_body.push_str(&format!("<br><p>{}</p>", caption));
                }
                if let Some(ref url) = broadcast.link_url {
                    let title = broadcast.link_title.as_deref().unwrap_or("Link");
                    html_body.push_str(&format!("<br><a href=\"{}\">{}</a>", url, title));
                }
                html_body
            };
            subject.get_or_insert_with(|| broadcast.title.clone());
            ChannelFormat {
                channel_code: "email".to_string(),
                channel_name: "Email".to_string(),
                formatted_body: html,
                formatted_media_url: media_url,
                formatted_subject: subject,
                supports_html: true,
                supports_images: true,
                supports_video: false,
                supports_links: true,
                max_length: 100000,
            }
        }
        "sms" => {
            // SMS: plain text, max 160 chars per segment (concatenated up to 918)
            let body = if let Some(ref html) = source_html {
                html_to_plain_text(html)
            } else {
                source_text
            };
            let body = body.chars().take(918).collect::<String>();
            let body = if body.len() > 160 {
                format!("{}...", body.chars().take(885).collect::<String>())
            } else {
                body
            };
            ChannelFormat {
                channel_code: "sms".to_string(),
                channel_name: "SMS".to_string(),
                formatted_body: body,
                formatted_media_url: None, // SMS doesn't support media (use MMS)
                formatted_subject: None,
                supports_html: false,
                supports_images: false,
                supports_video: false,
                supports_links: true,
                max_length: 160,
            }
        }
        "instagram" => {
            // Instagram DM: plain text with emoji, max 500 chars
            let body = if let Some(ref html) = source_html {
                html_to_plain_text(html)
            } else {
                source_text
            };
            let body = body.chars().take(500).collect::<String>();
            ChannelFormat {
                channel_code: "instagram".to_string(),
                channel_name: "Instagram".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: false,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 500,
            }
        }
        "facebook" => {
            // Facebook Messenger: plain text, supports media
            let body = if let Some(ref html) = source_html {
                html_to_plain_text(html)
            } else {
                source_text
            };
            let body = body.chars().take(64000).collect::<String>();
            ChannelFormat {
                channel_code: "facebook".to_string(),
                channel_name: "Facebook".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: false,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 64000,
            }
        }
        "telegram" => {
            // Telegram: MarkdownV2, max 4096 chars
            let body = if let Some(ref html) = source_html {
                html_to_telegram_markdown(html)
            } else {
                source_text
            };
            let body = body.chars().take(4096).collect::<String>();
            ChannelFormat {
                channel_code: "telegram".to_string(),
                channel_name: "Telegram".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: true,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 4096,
            }
        }
        "discord" => {
            // Discord: Markdown, max 2000 chars per message
            let body = if let Some(ref html) = source_html {
                html_to_discord_markdown(html)
            } else {
                source_text
            };
            let body = body.chars().take(2000).collect::<String>();
            ChannelFormat {
                channel_code: "discord".to_string(),
                channel_name: "Discord".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: false,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 2000,
            }
        }
        "slack" => {
            // Slack: plain text with slack markup, max 4000 chars
            let body = if let Some(ref html) = source_html {
                html_to_slack_markdown(html)
            } else {
                source_text
            };
            let body = body.chars().take(4000).collect::<String>();
            ChannelFormat {
                channel_code: "slack".to_string(),
                channel_name: "Slack".to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: None,
                supports_html: false,
                supports_images: true,
                supports_video: true,
                supports_links: true,
                max_length: 4000,
            }
        }
        _ => {
            // Default: plain text
            let body = if let Some(ref html) = source_html {
                html_to_plain_text(html)
            } else {
                source_text
            };
            ChannelFormat {
                channel_code: channel.to_string(),
                channel_name: channel.to_string(),
                formatted_body: body,
                formatted_media_url: media_url,
                formatted_subject: subject,
                supports_html: false,
                supports_images: false,
                supports_video: false,
                supports_links: true,
                max_length: 10000,
            }
        }
    }
}

/// Strip HTML tags from text
fn strip_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// HTML → channel format converters
// ---------------------------------------------------------------------------

/// Escape special HTML characters so plain text can be safely embedded in HTML
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert HTML to plain text, preserving links as [text](url) and stripping all tags
fn html_to_plain_text(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.chars().peekable();
    let mut in_tag = false;
    let mut link_stack: Vec<(String, String)> = Vec::new(); // (text, url)

    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
            // Check for <a href="...">
            let mut tag = String::new();
            while let Some(&nc) = chars.peek() {
                tag.push(nc);
                chars.next();
                if nc == '>' {
                    break;
                }
            }
            // Parse href
            if tag.to_lowercase().starts_with("<a ") {
                if let Some(href) = extract_attr(&tag, "href") {
                    let text_start = result.len();
                    link_stack.push((text_start.to_string(), href));
                }
            }
            // Check for </a>
            if tag.trim_end().to_lowercase().ends_with("</a>") {
                if let Some((start_str, url)) = link_stack.pop() {
                    let start = start_str.parse::<usize>().unwrap_or(0);
                    let link_text: String = result.drain(start..).collect();
                    // Re-insert as [text](url)
                    result.push_str(&format!("{} ({})", link_text, url));
                }
            }
            // Add newline for block elements
            let tag_lower = tag.trim().to_lowercase();
            if tag_lower.starts_with("</p>")
                || tag_lower.starts_with("</div>")
                || tag_lower.starts_with("</h1>")
                || tag_lower.starts_with("</h2>")
                || tag_lower.starts_with("</h3>")
                || tag_lower.starts_with("</li>")
                || tag_lower.starts_with("<p")
                || tag_lower.starts_with("<div")
                || tag_lower.starts_with("<h1")
                || tag_lower.starts_with("<h2")
                || tag_lower.starts_with("<h3")
                || tag_lower.starts_with("<li")
            {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            // Decode common HTML entities
            match c {
                '&' => {
                    let mut entity = String::new();
                    let mut end = false;
                    while let Some(&nc) = chars.peek() {
                        entity.push(nc);
                        chars.next();
                        if nc == ';' {
                            end = true;
                            break;
                        }
                        if entity.len() > 10 {
                            break;
                        }
                    }
                    if end {
                        match entity.as_str() {
                            "&amp;" => result.push('&'),
                            "&lt;" => result.push('<'),
                            "&gt;" => result.push('>'),
                            "&quot;" => result.push('"'),
                            "&nbsp;" => result.push(' '),
                            "&apos;" => result.push('\''),
                            _ => {
                                result.push('&');
                                result.push_str(&entity[..entity.len() - 1]);
                            }
                        }
                    } else {
                        result.push('&');
                        result.push_str(&entity);
                    }
                }
                _ => result.push(c),
            }
        }
    }
    // Clean up whitespace
    let result = result.replace("\n\n\n", "\n\n");
    result.trim().to_string()
}

/// Convert HTML to WhatsApp markdown (*bold*, _italic_, ~strikethrough~)
fn html_to_whatsapp_markdown(html: &str) -> String {
    let mut text = html_to_plain_text(html);
    // Convert HTML tags to WhatsApp markdown
    // Bold: <b>, <strong> → *text*
    text = text.replace("<b>", "*").replace("</b>", "*")
        .replace("<B>", "*").replace("</B>", "*")
        .replace("<strong>", "*").replace("</strong>", "*")
        .replace("<STRONG>", "*").replace("</STRONG>", "*");
    // Italic: <i>, <em> → _text_
    text = text.replace("<i>", "_").replace("</i>", "_")
        .replace("<I>", "_").replace("</I>", "_")
        .replace("<em>", "_").replace("</em>", "_")
        .replace("<EM>", "_").replace("</EM>", "_");
    // Strikethrough: <s>, <del>, <strike> → ~text~
    text = text.replace("<s>", "~").replace("</s>", "~")
        .replace("<S>", "~").replace("</S>", "~")
        .replace("<del>", "~").replace("</del>", "~")
        .replace("<DEL>", "~").replace("</DEL>", "~")
        .replace("<strike>", "~").replace("</strike>", "~")
        .replace("<STRIKE>", "~").replace("</STRIKE>", "~");
    // Code: <code> → ```text```
    text = text.replace("<code>", "```").replace("</code>", "```")
        .replace("<CODE>", "```").replace("</CODE>", "```");
    // Strip remaining tags
    text = strip_html_tags(&text);
    // Clean up whitespace
    text = text.replace("\n\n\n", "\n\n");
    text.trim().to_string()
}

/// Convert HTML to Telegram MarkdownV2 (*bold*, _italic_, ~strikethrough~, `code`, [text](url))
fn html_to_telegram_markdown(html: &str) -> String {
    let mut text = html_to_plain_text(html);
    // Escape Telegram special characters
    text = text.replace('&', "\\&")
        .replace('|', "\\|")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('+', "\\+")
        .replace(':', "\\:")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!");
    // Bold: <b>, <strong> → *text*
    text = text.replace("<b>", "*").replace("</b>", "*")
        .replace("<B>", "*").replace("</B>", "*")
        .replace("<strong>", "*").replace("</strong>", "*")
        .replace("<STRONG>", "*").replace("</STRONG>", "*");
    // Italic: <i>, <em> → _text_
    text = text.replace("<i>", "_").replace("</i>", "_")
        .replace("<I>", "_").replace("</I>", "_")
        .replace("<em>", "_").replace("</em>", "_")
        .replace("<EM>", "_").replace("</EM>", "_");
    // Strikethrough: <s>, <del> → ~text~
    text = text.replace("<s>", "~").replace("</s>", "~")
        .replace("<S>", "~").replace("</S>", "~")
        .replace("<del>", "~").replace("</del>", "~")
        .replace("<DEL>", "~").replace("</DEL>", "~");
    // Code: <code> → `text`
    text = text.replace("<code>", "`").replace("</code>", "`")
        .replace("<CODE>", "`").replace("</CODE>", "`");
    // Strip remaining tags
    text = strip_html_tags(&text);
    text = text.replace("\n\n\n", "\n\n");
    text.trim().to_string()
}

/// Convert HTML to Discord markdown (**bold**, *italic*, ~~strikethrough~~, `code`)
fn html_to_discord_markdown(html: &str) -> String {
    let mut text = html_to_plain_text(html);
    // Bold: <b>, <strong> → **text**
    text = text.replace("<b>", "**").replace("</b>", "**")
        .replace("<B>", "**").replace("</B>", "**")
        .replace("<strong>", "**").replace("</strong>", "**")
        .replace("<STRONG>", "**").replace("</STRONG>", "**");
    // Italic: <i>, <em> → *text*
    text = text.replace("<i>", "*").replace("</i>", "*")
        .replace("<I>", "*").replace("</I>", "*")
        .replace("<em>", "*").replace("</em>", "*")
        .replace("<EM>", "*").replace("</EM>", "*");
    // Strikethrough: <s>, <del> → ~~text~~
    text = text.replace("<s>", "~~").replace("</s>", "~~")
        .replace("<S>", "~~").replace("</S>", "~~")
        .replace("<del>", "~~").replace("</del>", "~~")
        .replace("<DEL>", "~~").replace("</DEL>", "~~");
    // Code: <code> → `text`
    text = text.replace("<code>", "`").replace("</code>", "`")
        .replace("<CODE>", "`").replace("</CODE>", "`");
    // Strip remaining tags
    text = strip_html_tags(&text);
    text = text.replace("\n\n\n", "\n\n");
    text.trim().to_string()
}

/// Convert HTML to Slack markdown (*bold*, _italic_, ~strikethrough~, `code`)
fn html_to_slack_markdown(html: &str) -> String {
    let mut text = html_to_plain_text(html);
    // Bold: <b>, <strong> → *text*
    text = text.replace("<b>", "*").replace("</b>", "*")
        .replace("<B>", "*").replace("</B>", "*")
        .replace("<strong>", "*").replace("</strong>", "*")
        .replace("<STRONG>", "*").replace("</STRONG>", "*");
    // Italic: <i>, <em> → _text_
    text = text.replace("<i>", "_").replace("</i>", "_")
        .replace("<I>", "_").replace("</I>", "_")
        .replace("<em>", "_").replace("</em>", "_")
        .replace("<EM>", "_").replace("</EM>", "_");
    // Strikethrough: <s>, <del> → ~text~
    text = text.replace("<s>", "~").replace("</s>", "~")
        .replace("<S>", "~").replace("</S>", "~")
        .replace("<del>", "~").replace("</del>", "~")
        .replace("<DEL>", "~").replace("</DEL>", "~");
    // Code: <code> → `text`
    text = text.replace("<code>", "`").replace("</code>", "`")
        .replace("<CODE>", "`").replace("</CODE>", "`");
    // Strip remaining tags
    text = strip_html_tags(&text);
    text = text.replace("\n\n\n", "\n\n");
    text.trim().to_string()
}

/// Extract an attribute value from an HTML tag string
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let tag_lower = tag.to_lowercase();
    let search = format!("{}=\"", attr);
    let search_single = format!("{}='", attr);

    if let Some(start) = tag_lower.find(&search) {
        let value_start = start + search.len();
        if let Some(end) = tag[value_start..].find('"') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }
    if let Some(start) = tag_lower.find(&search_single) {
        let value_start = start + search_single.len();
        if let Some(end) = tag[value_start..].find('\'') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_broadcast_summary(state: &AppState) -> Result<BroadcastSummary, String> {
    let total: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Count broadcasts: {}", e))?;

    let draft: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM broadcasts WHERE status = 'draft'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Count drafts: {}", e))?;

    let sent: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM broadcasts WHERE status = 'sent'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Count sent: {}", e))?;

    let failed: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM broadcasts WHERE status = 'failed'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Count failed: {}", e))?;

    let scheduled: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM broadcasts WHERE status = 'scheduled'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Count scheduled: {}", e))?;

    let total_recipients: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(total_recipients), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum recipients: {}", e))?;

    let total_sent: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(sent_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum sent: {}", e))?;

    let total_delivered: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(delivered_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum delivered: {}", e))?;

    let total_failed_count: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(failed_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum failed: {}", e))?;

    let total_read: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(read_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum read: {}", e))?;

    let total_clicked: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(clicked_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum clicked: {}", e))?;

    let total_replied: i32 = sqlx::query_scalar("SELECT COALESCE(SUM(replied_count), 0) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("Sum replied: {}", e))?;

    // By channel
    let channel_rows = sqlx::query(
        "SELECT bc.channel_code, bc.channel_name, COUNT(DISTINCT bc.broadcast_id) as broadcast_count, \
         SUM(bc.recipient_count) as recipient_count \
         FROM broadcast_channels bc GROUP BY bc.channel_code, bc.channel_name ORDER BY recipient_count DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Channel breakdown: {}", e))?;

    let by_channel: Vec<ChannelCount> = channel_rows.iter().map(|row| ChannelCount {
        channel_code: row.try_get(0).unwrap_or_default(),
        channel_name: row.try_get(1).unwrap_or_default(),
        broadcast_count: row.try_get(2).unwrap_or(0),
        recipient_count: row.try_get(3).unwrap_or(0),
    }).collect();

    // Recent broadcasts
    let recent = Broadcast::recent(&state.db, 10).await.unwrap_or_default();

    Ok(BroadcastSummary {
        total_broadcasts: total,
        draft_broadcasts: draft,
        sent_broadcasts: sent,
        failed_broadcasts: failed,
        scheduled_broadcasts: scheduled,
        total_recipients_sent: total_recipients,
        total_delivered,
        total_failed: total_failed_count,
        total_read,
        total_clicked,
        total_replied,
        by_channel,
        recent_broadcasts: recent,
    })
}

// ---------------------------------------------------------------------------
// Broadcast CRUD
// ---------------------------------------------------------------------------

impl Broadcast {
    pub async fn list(pool: &sqlx::PgPool, status: Option<&str>, limit: i32) -> Result<Vec<Self>, String> {
        let limit = if limit > 0 { limit } else { 50 };
        let query = if let Some(s) = status {
            format!("SELECT * FROM broadcasts WHERE status = $1 ORDER BY created_at DESC LIMIT {}", limit)
        } else {
            format!("SELECT * FROM broadcasts ORDER BY created_at DESC LIMIT {}", limit)
        };

        let rows = if let Some(s) = status {
            sqlx::query(&query)
                .bind(s)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("List broadcasts: {}", e))?
        } else {
            sqlx::query(&query)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("List broadcasts: {}", e))?
        };

        Ok(rows.iter().map(Self::from_row).collect())
    }

    pub async fn recent(pool: &sqlx::PgPool, limit: i32) -> Result<Vec<Self>, String> {
        Self::list(pool, None, limit).await
    }

    pub async fn get(pool: &sqlx::PgPool, id: &str) -> Result<Self, String> {
        let row = sqlx::query("SELECT * FROM broadcasts WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Get broadcast: {}", e))?;
        Ok(Self::from_row(&row))
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        organization_id: &str,
        brand_id: Option<&str>,
        title: &str,
        created_by: &str,
        body_text: &str,
        body_html: Option<&str>,
        subject_line: Option<&str>,
        preview_text: Option<&str>,
        media_type: &str,
        primary_media_url: Option<&str>,
        media_caption: Option<&str>,
        link_url: Option<&str>,
        link_title: Option<&str>,
        link_description: Option<&str>,
        link_image_url: Option<&str>,
        target_type: &str,
        target_ids: &str,
        channels: &str,
        scheduled_at: Option<&str>,
    ) -> Result<Self, String> {
        let id = gen_id();
        let row = sqlx::query(
            "INSERT INTO broadcasts (id, organization_id, brand_id, title, status, created_by, \
             scheduled_at, body_text, body_html, subject_line, preview_text, \
             media_type, primary_media_url, media_caption, \
             link_url, link_title, link_description, link_image_url, \
             target_type, target_ids, channels) \
             VALUES ($1, $2, $3, $4, 'draft', $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20) \
             RETURNING *"
        )
        .bind(&id)
        .bind(organization_id)
        .bind(brand_id)
        .bind(title)
        .bind(created_by)
        .bind(scheduled_at)
        .bind(body_text)
        .bind(body_html)
        .bind(subject_line)
        .bind(preview_text)
        .bind(media_type)
        .bind(primary_media_url)
        .bind(media_caption)
        .bind(link_url)
        .bind(link_title)
        .bind(link_description)
        .bind(link_image_url)
        .bind(target_type)
        .bind(target_ids)
        .bind(channels)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Create broadcast: {}", e))?;

        // Log creation
        sqlx::query(
            "INSERT INTO broadcast_logs (broadcast_id, event_type, details) VALUES ($1, 'created', $2)"
        )
        .bind(&id)
        .bind(r#"{"action": "created"}"#)
        .execute(pool)
        .await
        .ok();

        Ok(Self::from_row(&row))
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        id: &str,
        mut updates: serde_json::Map<String, Value>,
    ) -> Result<Self, String> {
        // Remove fields that shouldn't be updated directly
        updates.remove("id");
        updates.remove("organization_id");
        updates.remove("created_by");
        updates.remove("created_at");

        if updates.is_empty() {
            return Self::get(pool, id).await;
        }

        let cols: Vec<&str> = updates.keys().map(|s| s.as_str()).collect();
        let set_clause = cols.iter().map(|c| format!("{} = $1", c)).collect::<Vec<_>>().join(", ");
        let sql = format!("UPDATE broadcasts SET {}, updated_at = NOW() WHERE id = ${} RETURNING *", set_clause, cols.len() + 1);

        let mut query = sqlx::query(&sql);
        for value in updates.values() {
            let json = serde_json::to_value(value).unwrap_or(Value::Null);
            query = query.bind(json);
        }
        query = query.bind(id);

        let row = query
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Update broadcast: {}", e))?;

        Ok(Self::from_row(&row))
    }

    pub async fn delete(pool: &sqlx::PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM broadcasts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| format!("Delete broadcast: {}", e))?;
        Ok(())
    }

    pub async fn update_status(pool: &sqlx::PgPool, id: &str, status: &str) -> Result<Self, String> {
        let mut extra = String::new();
        let now = chrono::Utc::now().to_rfc3339();

        match status {
            "completed" => extra = format!("completed_at = '{}', ", now),
            "cancelled" => extra = format!("cancelled_at = '{}', ", now),
            _ => {}
        }

        let row = sqlx::query(&format!(
            "UPDATE broadcasts SET status = $1, {}, updated_at = NOW() WHERE id = $2 RETURNING *",
            extra
        ))
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Update status: {}", e))?;

        // Log status change
        sqlx::query(
            "INSERT INTO broadcast_logs (broadcast_id, event_type, details) VALUES ($1, $2, $3)"
        )
        .bind(id)
        .bind(status)
        .bind(&format!(r#"{{"status": "{}"}}"#, status))
        .execute(pool)
        .await
        .ok();

        Ok(Self::from_row(&row))
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            brand_id: row.try_get(2).unwrap(),
            title: row.try_get(3).unwrap_or_default(),
            status: row.try_get(4).unwrap_or_default(),
            created_by: row.try_get(5).unwrap_or_default(),
            scheduled_at: row.try_get(6).unwrap(),
            completed_at: row.try_get(7).unwrap(),
            cancelled_at: row.try_get(8).unwrap(),
            body_text: row.try_get(9).unwrap_or_default(),
            body_html: row.try_get(10).unwrap(),
            subject_line: row.try_get(11).unwrap(),
            preview_text: row.try_get(12).unwrap(),
            media_type: row.try_get(13).unwrap_or_default(),
            primary_media_url: row.try_get(14).unwrap(),
            media_caption: row.try_get(15).unwrap(),
            link_url: row.try_get(16).unwrap(),
            link_title: row.try_get(17).unwrap(),
            link_description: row.try_get(18).unwrap(),
            link_image_url: row.try_get(19).unwrap(),
            target_type: row.try_get(20).unwrap_or_default(),
            target_ids: row.try_get(21).unwrap_or(Value::Array(vec![])),
            channels: row.try_get(22).unwrap_or(Value::Array(vec![])),
            channel_overrides: row.try_get(23).unwrap_or(Value::Object(Default::default())),
            total_recipients: row.try_get(24).unwrap_or(0),
            sent_count: row.try_get(25).unwrap_or(0),
            delivered_count: row.try_get(26).unwrap_or(0),
            failed_count: row.try_get(27).unwrap_or(0),
            read_count: row.try_get(28).unwrap_or(0),
            clicked_count: row.try_get(29).unwrap_or(0),
            replied_count: row.try_get(30).unwrap_or(0),
            send_delay_seconds: row.try_get(31).unwrap_or(0),
            retry_failed: row.try_get(32).unwrap_or(true),
            max_retries: row.try_get(33).unwrap_or(3),
            track_opens: row.try_get(34).unwrap_or(true),
            track_clicks: row.try_get(35).unwrap_or(true),
            metadata: row.try_get(36).unwrap_or(Value::Object(Default::default())),
            created_at: row.try_get(37).unwrap_or_default(),
            updated_at: row.try_get(38).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Media management
// ---------------------------------------------------------------------------

impl BroadcastMedia {
    pub async fn upload(
        pool: &sqlx::PgPool,
        broadcast_id: &str,
        media_type: &str,
        file_name: &str,
        file_size_bytes: i64,
        mime_type: &str,
        file_url: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<Self, String> {
        let id = gen_id();
        let row = sqlx::query(
            "INSERT INTO broadcast_media (id, broadcast_id, media_type, file_name, file_size_bytes, \
             mime_type, file_url, thumbnail_url, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'ready') \
             RETURNING *"
        )
        .bind(&id)
        .bind(broadcast_id)
        .bind(media_type)
        .bind(file_name)
        .bind(file_size_bytes)
        .bind(mime_type)
        .bind(file_url)
        .bind(thumbnail_url)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Upload media: {}", e))?;

        Ok(Self::from_row(&row))
    }

    pub async fn list_by_broadcast(pool: &sqlx::PgPool, broadcast_id: &str) -> Result<Vec<Self>, String> {
        let rows = sqlx::query("SELECT * FROM broadcast_media WHERE broadcast_id = $1 ORDER BY created_at DESC")
            .bind(broadcast_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("List media: {}", e))?;

        Ok(rows.iter().map(Self::from_row).collect())
    }

    pub async fn delete(pool: &sqlx::PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM broadcast_media WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| format!("Delete media: {}", e))?;
        Ok(())
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            broadcast_id: row.try_get(1).unwrap_or_default(),
            media_type: row.try_get(2).unwrap_or_default(),
            file_name: row.try_get(3).unwrap_or_default(),
            file_size_bytes: row.try_get(4).unwrap_or(0),
            mime_type: row.try_get(5).unwrap_or_default(),
            file_url: row.try_get(6).unwrap_or_default(),
            thumbnail_url: row.try_get(7).unwrap(),
            duration_seconds: row.try_get(8).unwrap_or(0),
            width: row.try_get(9).unwrap_or(0),
            height: row.try_get(10).unwrap_or(0),
            status: row.try_get(11).unwrap_or_default(),
            error_message: row.try_get(12).unwrap(),
            metadata: row.try_get(13).unwrap_or(Value::Object(Default::default())),
            created_at: row.try_get(14).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Channel formatting and preview
// ---------------------------------------------------------------------------

pub async fn get_format_preview(pool: &sqlx::PgPool, broadcast_id: &str) -> Result<FormatPreview, String> {
    let broadcast = Broadcast::get(pool, broadcast_id).await?;
    let channels: Vec<String> = serde_json::from_value(broadcast.channels.clone()).unwrap_or_default();

    let formats: Vec<ChannelFormat> = channels.iter()
        .map(|ch| format_for_channel(&broadcast, ch))
        .collect();

    Ok(FormatPreview {
        broadcast_id: broadcast_id.to_string(),
        formats,
    })
}

/// Preview HTML content formatted for all channels without creating a broadcast.
/// Accepts HTML (or plain text) and returns the formatted output for every channel.
pub fn preview_html_for_all_channels(html: &str, body_text: &str, channels: &[String]) -> Vec<ChannelFormat> {
    // Build a temporary broadcast to reuse format_for_channel
    let broadcast = Broadcast {
        id: "preview".to_string(),
        organization_id: "".to_string(),
        brand_id: None,
        title: "Preview".to_string(),
        status: "draft".to_string(),
        created_by: "".to_string(),
        scheduled_at: None,
        completed_at: None,
        cancelled_at: None,
        body_text: body_text.to_string(),
        body_html: if !html.is_empty() { Some(html.to_string()) } else { None },
        subject_line: None,
        preview_text: None,
        media_type: "text".to_string(),
        primary_media_url: None,
        media_caption: None,
        link_url: None,
        link_title: None,
        link_description: None,
        link_image_url: None,
        target_type: "contacts".to_string(),
        target_ids: json!([]),
        channels: json!(channels),
        channel_overrides: json!({}),
        total_recipients: 0,
        sent_count: 0,
        delivered_count: 0,
        failed_count: 0,
        read_count: 0,
        clicked_count: 0,
        replied_count: 0,
        send_delay_seconds: 0,
        retry_failed: false,
        max_retries: 0,
        track_opens: false,
        track_clicks: false,
        metadata: json!({}),
        created_at: "".to_string(),
        updated_at: "".to_string(),
    };

    channels.iter()
        .map(|ch| format_for_channel(&broadcast, ch))
        .collect()
}

// ---------------------------------------------------------------------------
// Send broadcast (one-click to all channels)
// ---------------------------------------------------------------------------

pub async fn send_broadcast(pool: &sqlx::PgPool, broadcast_id: &str) -> Result<SendBroadcastResult, String> {
    let broadcast = Broadcast::get(pool, broadcast_id).await?;

    // Validate
    if broadcast.status == "sent" || broadcast.status == "completed" {
        return Err("Broadcast already sent".to_string());
    }
    if broadcast.status == "cancelled" {
        return Err("Broadcast is cancelled".to_string());
    }

    // Update status to sending
    Broadcast::update_status(pool, broadcast_id, "sending").await?;

    // Log start
    sqlx::query(
        "INSERT INTO broadcast_logs (broadcast_id, event_type, details) VALUES ($1, 'started', $2)"
    )
    .bind(broadcast_id)
    .bind(r#"{"action": "send_started"}"#)
    .execute(pool)
    .await
    .ok();

    let channels: Vec<String> = serde_json::from_value(broadcast.channels.clone()).unwrap_or_default();
    let target_ids: Vec<String> = serde_json::from_value(broadcast.target_ids.clone()).unwrap_or_default();

    let mut channels_sent = Vec::new();
    let mut total_recipients = 0i32;

    // For each channel, create broadcast_channel record and recipients
    for channel in &channels {
        let channel_name = match channel.as_str() {
            "whatsapp" => "WhatsApp",
            "email" => "Email",
            "sms" => "SMS",
            "instagram" => "Instagram",
            "facebook" => "Facebook",
            "telegram" => "Telegram",
            "discord" => "Discord",
            "slack" => "Slack",
            _ => channel.as_str(),
        }.to_string();

        let format = format_for_channel(&broadcast, channel);
        let bc_id = gen_id();

        // Create broadcast_channel record
        sqlx::query(
            "INSERT INTO broadcast_channels (id, broadcast_id, channel_code, channel_name, status, \
             formatted_body, formatted_media_url, formatted_subject, recipient_count, started_at) \
             VALUES ($1, $2, $3, $4, 'sending', $5, $6, $7, $8, NOW())"
        )
        .bind(&bc_id)
        .bind(broadcast_id)
        .bind(channel)
        .bind(&channel_name)
        .bind(&format.formatted_body)
        .bind(&format.formatted_media_url)
        .bind(&format.formatted_subject)
        .bind(target_ids.len() as i32)
        .execute(pool)
        .await
        .map_err(|e| format!("Create channel record: {}", e))?;

        // Create recipient records for each target
        for target_id in &target_ids {
            let recipient_id = gen_id();
            sqlx::query(
                "INSERT INTO broadcast_recipients (id, broadcast_id, contact_id, channel_code, status) \
                 VALUES ($1, $2, $3, $4, 'queued')"
            )
            .bind(&recipient_id)
            .bind(broadcast_id)
            .bind(target_id)
            .bind(channel)
            .execute(pool)
            .await
            .ok();
        }

        // Simulate sending (production would call actual channel APIs)
        let sent_count = target_ids.len() as i32;
        total_recipients += sent_count;

        sqlx::query(
            "UPDATE broadcast_channels SET status = 'sent', sent_count = $1, completed_at = NOW() \
             WHERE id = $2"
        )
        .bind(sent_count)
        .bind(&bc_id)
        .execute(pool)
        .await
        .ok();

        channels_sent.push(channel.clone());

        // Log channel sent
        sqlx::query(
            "INSERT INTO broadcast_logs (broadcast_id, event_type, channel_code, details) \
             VALUES ($1, 'sent', $2, $3)"
        )
        .bind(broadcast_id)
        .bind(channel)
        .bind(&format!(r#"{{"sent_count": {}}}"#, sent_count))
        .execute(pool)
        .await
        .ok();
    }

    // Update broadcast with final stats
    sqlx::query(
        "UPDATE broadcasts SET status = 'sent', total_recipients = $1, sent_count = $2, \
         completed_at = NOW(), updated_at = NOW() WHERE id = $3"
    )
    .bind(total_recipients)
    .bind(total_recipients)
    .bind(broadcast_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Update broadcast stats: {}", e))?;

    // Log completion
    sqlx::query(
        "INSERT INTO broadcast_logs (broadcast_id, event_type, details) VALUES ($1, 'completed', $2)"
    )
    .bind(broadcast_id)
    .bind(&format!(r#"{{"total_recipients": {}, "channels": {:?}}}"#, total_recipients, channels_sent))
    .execute(pool)
    .await
    .ok();

    Ok(SendBroadcastResult {
        broadcast_id: broadcast_id.to_string(),
        status: "sent".to_string(),
        channels_sent,
        total_recipients,
        message: format!("Broadcast sent to {} recipients across {} channels", total_recipients, channels.len()),
    })
}

// ---------------------------------------------------------------------------
// Broadcast channels
// ---------------------------------------------------------------------------

impl BroadcastChannel {
    pub async fn list_by_broadcast(pool: &sqlx::PgPool, broadcast_id: &str) -> Result<Vec<Self>, String> {
        let rows = sqlx::query("SELECT * FROM broadcast_channels WHERE broadcast_id = $1 ORDER BY created_at DESC")
            .bind(broadcast_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("List channels: {}", e))?;

        Ok(rows.iter().map(Self::from_row).collect())
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            broadcast_id: row.try_get(1).unwrap_or_default(),
            channel_code: row.try_get(2).unwrap_or_default(),
            channel_name: row.try_get(3).unwrap_or_default(),
            status: row.try_get(4).unwrap_or_default(),
            formatted_body: row.try_get(5).unwrap(),
            formatted_media_url: row.try_get(6).unwrap(),
            formatted_subject: row.try_get(7).unwrap(),
            recipient_count: row.try_get(8).unwrap_or(0),
            sent_count: row.try_get(9).unwrap_or(0),
            failed_count: row.try_get(10).unwrap_or(0),
            started_at: row.try_get(11).unwrap(),
            completed_at: row.try_get(12).unwrap(),
            error_message: row.try_get(13).unwrap(),
            metadata: row.try_get(14).unwrap_or(Value::Object(Default::default())),
            created_at: row.try_get(15).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast recipients
// ---------------------------------------------------------------------------

impl BroadcastRecipient {
    pub async fn list_by_broadcast(pool: &sqlx::PgPool, broadcast_id: &str, status: Option<&str>, limit: i32) -> Result<Vec<Self>, String> {
        let limit = if limit > 0 { limit } else { 100 };
        let query = if let Some(s) = status {
            format!("SELECT * FROM broadcast_recipients WHERE broadcast_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT {}", limit)
        } else {
            format!("SELECT * FROM broadcast_recipients WHERE broadcast_id = $1 ORDER BY created_at DESC LIMIT {}", limit)
        };

        let rows = if let Some(s) = status {
            sqlx::query(&query)
                .bind(broadcast_id)
                .bind(s)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("List recipients: {}", e))?
        } else {
            sqlx::query(&query)
                .bind(broadcast_id)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("List recipients: {}", e))?
        };

        Ok(rows.iter().map(Self::from_row).collect())
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            broadcast_id: row.try_get(1).unwrap_or_default(),
            contact_id: row.try_get(2).unwrap_or_default(),
            channel_code: row.try_get(3).unwrap_or_default(),
            phone_number: row.try_get(4).unwrap(),
            email_address: row.try_get(5).unwrap(),
            external_id: row.try_get(6).unwrap(),
            status: row.try_get(7).unwrap_or_default(),
            message_id: row.try_get(8).unwrap(),
            sent_at: row.try_get(9).unwrap(),
            delivered_at: row.try_get(10).unwrap(),
            read_at: row.try_get(11).unwrap(),
            clicked_at: row.try_get(12).unwrap(),
            replied_at: row.try_get(13).unwrap(),
            error_message: row.try_get(14).unwrap(),
            retry_count: row.try_get(15).unwrap_or(0),
            metadata: row.try_get(16).unwrap_or(Value::Object(Default::default())),
            created_at: row.try_get(17).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast logs
// ---------------------------------------------------------------------------

impl BroadcastLog {
    pub async fn list_by_broadcast(pool: &sqlx::PgPool, broadcast_id: &str, limit: i32) -> Result<Vec<Self>, String> {
        let limit = if limit > 0 { limit } else { 50 };
        let rows = sqlx::query(&format!(
            "SELECT * FROM broadcast_logs WHERE broadcast_id = $1 ORDER BY created_at DESC LIMIT {}", limit
        ))
        .bind(broadcast_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("List logs: {}", e))?;

        Ok(rows.iter().map(Self::from_row).collect())
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            broadcast_id: row.try_get(1).unwrap_or_default(),
            event_type: row.try_get(2).unwrap_or_default(),
            channel_code: row.try_get(3).unwrap(),
            recipient_id: row.try_get(4).unwrap(),
            details: row.try_get(5).unwrap_or(Value::Object(Default::default())),
            created_at: row.try_get(6).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

impl BroadcastSettings {
    pub async fn get(pool: &sqlx::PgPool, organization_id: &str) -> Result<Self, String> {
        let row = sqlx::query("SELECT * FROM broadcast_settings WHERE organization_id = $1")
            .bind(organization_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Get settings: {}", e))?;

        match row {
            Some(row) => Ok(Self::from_row(&row)),
            None => {
                // Create default settings
                let id = gen_id();
                let row = sqlx::query(
                    "INSERT INTO broadcast_settings (id, organization_id) VALUES ($1, $2) RETURNING *"
                )
                .bind(&id)
                .bind(organization_id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("Create default settings: {}", e))?;
                Ok(Self::from_row(&row))
            }
        }
    }

    pub async fn update(pool: &sqlx::PgPool, organization_id: &str, updates: serde_json::Map<String, Value>) -> Result<Self, String> {
        if updates.is_empty() {
            return Self::get(pool, organization_id).await;
        }

        let cols: Vec<&str> = updates.keys().map(|s| s.as_str()).collect();
        let set_clause = cols.iter().map(|c| format!("{} = $1", c)).collect::<Vec<_>>().join(", ");
        let sql = format!("UPDATE broadcast_settings SET {}, updated_at = NOW() WHERE organization_id = ${} RETURNING *", set_clause, cols.len() + 1);

        let mut query = sqlx::query(&sql);
        for value in updates.values() {
            let json = serde_json::to_value(value).unwrap_or(Value::Null);
            query = query.bind(json);
        }
        query = query.bind(organization_id);

        let row = query
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Update settings: {}", e))?;

        Ok(Self::from_row(&row))
    }

    fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            enabled: row.try_get(2).unwrap_or(true),
            max_recipients_per_broadcast: row.try_get(3).unwrap_or(10000),
            default_send_delay_seconds: row.try_get(4).unwrap_or(2),
            rate_limit_per_minute: row.try_get(5).unwrap_or(60),
            allow_scheduling: row.try_get(6).unwrap_or(true),
            require_approval: row.try_get(7).unwrap_or(false),
            approved_by: row.try_get(8).unwrap(),
            auto_format_channels: row.try_get(9).unwrap_or(true),
            track_opens: row.try_get(10).unwrap_or(true),
            track_clicks: row.try_get(11).unwrap_or(true),
            default_channels: row.try_get(12).unwrap_or_else(|_| vec!["whatsapp".to_string(), "email".to_string()]),
            created_at: row.try_get(13).unwrap_or_default(),
            updated_at: row.try_get(14).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Available channels info
// ---------------------------------------------------------------------------

pub fn get_available_channels() -> Vec<ChannelFormat> {
    vec![
        ChannelFormat {
            channel_code: "whatsapp".to_string(),
            channel_name: "WhatsApp".to_string(),
            formatted_body: "Plain text with markdown (*bold*, _italic_). Max 4096 chars.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 4096,
        },
        ChannelFormat {
            channel_code: "email".to_string(),
            channel_name: "Email".to_string(),
            formatted_body: "Full HTML support with images, links, and rich formatting.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: true,
            supports_images: true,
            supports_video: false,
            supports_links: true,
            max_length: 100000,
        },
        ChannelFormat {
            channel_code: "sms".to_string(),
            channel_name: "SMS".to_string(),
            formatted_body: "Plain text only. Max 160 chars per segment.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: false,
            supports_video: false,
            supports_links: true,
            max_length: 160,
        },
        ChannelFormat {
            channel_code: "instagram".to_string(),
            channel_name: "Instagram".to_string(),
            formatted_body: "Plain text with emoji. Max 500 chars.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 500,
        },
        ChannelFormat {
            channel_code: "facebook".to_string(),
            channel_name: "Facebook".to_string(),
            formatted_body: "Plain text with media support.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 64000,
        },
        ChannelFormat {
            channel_code: "telegram".to_string(),
            channel_name: "Telegram".to_string(),
            formatted_body: "MarkdownV2 or HTML. Max 4096 chars.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: true,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 4096,
        },
        ChannelFormat {
            channel_code: "discord".to_string(),
            channel_name: "Discord".to_string(),
            formatted_body: "Markdown. Max 2000 chars per message.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 2000,
        },
        ChannelFormat {
            channel_code: "slack".to_string(),
            channel_name: "Slack".to_string(),
            formatted_body: "Block Kit or plain text. Max 4000 chars.".to_string(),
            formatted_media_url: None,
            formatted_subject: None,
            supports_html: false,
            supports_images: true,
            supports_video: true,
            supports_links: true,
            max_length: 4000,
        },
    ]
}
