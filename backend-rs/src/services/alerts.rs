//! Port of `services/alerts.py`.
//!
//! Python posted to a Slack-compatible webhook and discarded every failure. The
//! outcome is returned here instead, so a caller can say out loud that nobody
//! was paged rather than letting a hot lead look notified when it was not.

use crate::db::AppState;
use std::time::Duration;

pub struct HotLeadAlert<'a> {
    /// `None` means the contact has no linked lead row; the message says that
    /// instead of naming someone.
    pub lead_name: Option<&'a str>,
    pub institution_type: Option<&'a str>,
    pub city: Option<&'a str>,
    pub channel: &'a str,
    pub intent: &'a str,
    pub sentiment: &'a str,
    pub conversation_id: &'a str,
    pub message_body: &'a str,
}

fn or_unrecorded(value: Option<&str>) -> &str {
    match value {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => "(not recorded)",
    }
}

pub enum AlertOutcome {
    /// The webhook answered 2xx.
    Notified(u16),
    /// `SALES_ALERT_WEBHOOK_URL` is unset, so there is nowhere to post.
    NotConfigured,
    /// A URL is configured but the post failed; carries the reason.
    Failed(String),
}

impl AlertOutcome {
    pub fn describe(&self) -> String {
        match self {
            AlertOutcome::Notified(status) => {
                format!("hot-lead alert posted to the sales webhook (HTTP {status})")
            }
            AlertOutcome::NotConfigured => "no hot-lead alert was sent: SALES_ALERT_WEBHOOK_URL \
             is not configured, so there is no paging target"
                .to_string(),
            AlertOutcome::Failed(reason) => {
                format!("hot-lead alert was NOT delivered: {reason}")
            }
        }
    }
    pub fn delivered(&self) -> bool {
        matches!(self, AlertOutcome::Notified(_))
    }
}

pub async fn send_hot_lead_alert(state: &AppState, alert: &HotLeadAlert<'_>) -> AlertOutcome {
    let webhook = state.config.sales_alert_webhook_url.trim();
    if webhook.is_empty() {
        return AlertOutcome::NotConfigured;
    }

    let who = match alert.lead_name {
        Some(name) if !name.trim().is_empty() => format!("*{name}*"),
        _ => "(no lead row linked to this contact)".to_string(),
    };
    let text = format!(
        ":rotating_light: Hot lead: {who} (institution type: {}, city: {})\nIntent: `{}` | \
         Sentiment: `{}` | Channel: `{}`\n> {}\nConversation: {}",
        or_unrecorded(alert.institution_type),
        or_unrecorded(alert.city),
        alert.intent,
        alert.sentiment,
        alert.channel,
        alert.message_body,
        alert.conversation_id
    );

    let result = state
        .http_client
        .post(webhook)
        .json(&serde_json::json!({ "text": text }))
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                AlertOutcome::Notified(status.as_u16())
            } else {
                AlertOutcome::Failed(format!(
                    "the sales webhook answered HTTP {} with: {}",
                    status.as_u16(),
                    response
                        .text()
                        .await
                        .unwrap_or_else(|err| format!("<body unreadable: {err}>"))
                ))
            }
        }
        Err(err) => AlertOutcome::Failed(format!("the sales webhook request failed: {err}")),
    }
}
