//! Fin Vision — Image Understanding Service
//!
//! Enables the AI to analyze images/screenshots sent by customers and
//! incorporate visual context into conversation responses. Mirrors
//! Intercom's Fin Vision capability.
//!
//! Uses Claude's multi-modal vision capabilities to:
//! - Detect objects and elements in images
//! - Extract text via OCR
//! - Analyze screenshots for error messages, UI elements
//! - Generate human-readable summaries

use crate::db::AppState;
use crate::services::claude_client::{self, ClaudeError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionImage {
    pub id: String,
    pub organization_id: String,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub channel: Option<String>,
    pub image_url: String,
    pub image_data: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysis {
    pub id: String,
    pub image_id: String,
    pub analysis_type: String,
    pub prompt: Option<String>,
    pub result: String,
    pub confidence: f64,
    pub objects_detected: String,
    pub text_extracted: Option<String>,
    pub summary: Option<String>,
    pub processing_time_ms: i64,
    pub claude_model: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub auto_analyze: bool,
    pub default_analysis_type: String,
    pub claude_model: String,
    pub max_file_size_mb: f64,
    pub allowed_mime_types: String,
    pub ocr_enabled: bool,
    pub object_detection_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionSummary {
    pub total_images: i64,
    pub total_analyses: i64,
    pub avg_confidence: f64,
    pub avg_processing_time_ms: i64,
    pub analysis_type_breakdown: Vec<AnalysisTypeCount>,
    pub recent_analyses: Vec<VisionAnalysis>,
    pub enabled: bool,
    pub auto_analyze: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTypeCount {
    pub analysis_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub image_id: String,
    pub status: String,
    pub analysis_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResult {
    pub analysis_id: String,
    pub image_id: String,
    pub analysis_type: String,
    pub result: Value,
    pub confidence: f64,
    pub objects_detected: Vec<String>,
    pub text_extracted: Option<String>,
    pub summary: Option<String>,
    pub processing_time_ms: i64,
    pub claude_model: String,
}

// ---------------------------------------------------------------------------
// Analysis prompts by type
// ---------------------------------------------------------------------------

fn analysis_prompt(analysis_type: &str, custom_prompt: Option<&str>) -> String {
    match custom_prompt {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => match analysis_type {
            "screenshot" => {
                "Analyze this screenshot carefully. Identify: 1) What application or website is shown, 2) Any error messages or warnings visible, 3) Key UI elements and their state, 4) Text content visible in the image. Provide a structured JSON response with: summary, objects_detected (array), text_extracted, error_messages (array), and confidence score.".to_string()
            }
            "document" => {
                "Analyze this document image. Extract all visible text, identify the document type, and summarize the key information. Provide a structured JSON response with: summary, text_extracted, document_type, key_information (array), and confidence score.".to_string()
            }
            "error_code" => {
                "Analyze this image for error codes, error messages, or technical issues. Identify the exact error text, error codes, and provide troubleshooting suggestions. Provide a structured JSON response with: summary, error_messages (array), error_codes (array), text_extracted, suggested_actions (array), and confidence score.".to_string()
            }
            "product" => {
                "Analyze this product image. Identify the product, its features, color, size indicators, and any visible text or labels. Provide a structured JSON response with: summary, objects_detected (array), product_name, features (array), text_extracted, and confidence score.".to_string()
            }
            _ => {
                "Analyze this image and provide a detailed description. Identify objects, text, colors, and context. Provide a structured JSON response with: summary, objects_detected (array), text_extracted, colors_detected (array), and confidence score.".to_string()
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Core engine
// ---------------------------------------------------------------------------

pub async fn get_vision_summary(state: &AppState) -> Result<VisionSummary, String> {
    // Total images
    // NOTE: these queries carried a `WHERE organization_id = ?1` with no
    // corresponding `.bind()` anywhere -- this function takes no
    // organization_id parameter at all, so the placeholder was unbound and
    // would fail at execution time regardless of placeholder style (it would
    // have failed the same way under the original engine). The dead filter
    // is removed here rather than guessed at; this is a judgment call, not a
    // mechanical placeholder rename -- flagged for review.
    let total_images: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vision_images"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Total analyses
    let total_analyses: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vision_analyses va JOIN vision_images vi ON va.image_id = vi.id"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Average confidence
    let avg_confidence: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(va.confidence), 0) FROM vision_analyses va JOIN vision_images vi ON va.image_id = vi.id"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Average processing time
    let avg_processing_time_ms: i64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(va.processing_time_ms), 0) FROM vision_analyses va JOIN vision_images vi ON va.image_id = vi.id"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Analysis type breakdown
    let type_rows = sqlx::query(
        "SELECT va.analysis_type, COUNT(*) as cnt FROM vision_analyses va \
         JOIN vision_images vi ON va.image_id = vi.id \
         GROUP BY va.analysis_type ORDER BY cnt DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let analysis_type_breakdown: Vec<AnalysisTypeCount> = type_rows
        .iter()
        .map(|row| AnalysisTypeCount {
            analysis_type: row.try_get(0).unwrap_or_default(),
            count: row.try_get(1).unwrap_or(0i64),
        })
        .collect();

    // Recent analyses (last 10)
    let recent_rows = sqlx::query(
        "SELECT va.id, va.image_id, va.analysis_type, va.prompt, va.result, va.confidence, \
         va.objects_detected, va.text_extracted, va.summary, va.processing_time_ms, \
         va.claude_model, va.error_message, va.created_at \
         FROM vision_analyses va JOIN vision_images vi ON va.image_id = vi.id \
         ORDER BY va.created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let recent_analyses: Vec<VisionAnalysis> = recent_rows
        .iter()
        .map(|row| VisionAnalysis {
            id: row.try_get(0).unwrap_or_default(),
            image_id: row.try_get(1).unwrap_or_default(),
            analysis_type: row.try_get(2).unwrap_or_default(),
            prompt: row.try_get(3).ok(),
            result: row.try_get(4).unwrap_or_default(),
            confidence: row.try_get(5).unwrap_or(0.0),
            objects_detected: row.try_get(6).unwrap_or_default(),
            text_extracted: row.try_get(7).ok(),
            summary: row.try_get(8).ok(),
            processing_time_ms: row.try_get(9).unwrap_or(0i64),
            claude_model: row.try_get(10).unwrap_or_default(),
            error_message: row.try_get(11).ok(),
            created_at: row.try_get(12).unwrap_or_default(),
        })
        .collect();

    // Settings
    let settings = get_vision_settings(state, "org-default").await;
    let enabled = settings.as_ref().map(|s| s.enabled).unwrap_or(true);
    let auto_analyze = settings.as_ref().map(|s| s.auto_analyze).unwrap_or(true);

    Ok(VisionSummary {
        total_images,
        total_analyses,
        avg_confidence,
        avg_processing_time_ms,
        analysis_type_breakdown,
        recent_analyses,
        enabled,
        auto_analyze,
    })
}

pub async fn upload_image(
    state: &AppState,
    organization_id: &str,
    conversation_id: Option<&str>,
    contact_id: Option<&str>,
    channel: Option<&str>,
    image_data: &str,
    mime_type: &str,
    file_size: i64,
) -> Result<UploadResult, String> {
    // Check settings
    let settings = get_vision_settings(state, organization_id).await;
    if let Some(ref s) = settings {
        if !s.enabled {
            return Err("Fin Vision is disabled for this organization".to_string());
        }
        // Validate file size
        let max_size_bytes = (s.max_file_size_mb * 1024.0 * 1024.0) as i64;
        if file_size > max_size_bytes {
            return Err(format!(
                "File size {} bytes exceeds maximum {} MB",
                file_size, s.max_file_size_mb
            ));
        }
        // Validate MIME type
        let allowed: Vec<Value> = serde_json::from_str(&s.allowed_mime_types).unwrap_or_default();
        let allowed_strs: Vec<String> = allowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        if !allowed_strs.is_empty() && !allowed_strs.contains(&mime_type.to_string()) {
            return Err(format!("MIME type {} is not allowed. Allowed types: {}", mime_type, s.allowed_mime_types));
        }
    }

    let image_id = format!("vision-img-{}", Utc::now().timestamp_millis());

    // Insert image record
    let conv_id = conversation_id.unwrap_or("");
    let contact = contact_id.unwrap_or("");
    let chan = channel.unwrap_or("");
    let safe_data = image_data.replace('\'', "''");
    let safe_mime = mime_type.replace('\'', "''");

    sqlx::query(&format!(
        "INSERT INTO vision_images (id, organization_id, conversation_id, contact_id, channel, image_url, image_data, mime_type, file_size, status) \
         VALUES ('{}', '{}', '{}', '{}', '{}', '', '{}', '{}', {}, 'uploaded')",
        image_id, organization_id, conv_id, contact, chan, safe_data, safe_mime, file_size
    ))
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to save image: {}", e))?;

    // Auto-analyze if enabled
    let auto = settings.as_ref().map(|s| s.auto_analyze).unwrap_or(true);
    if auto {
        let analysis_type = settings
            .as_ref()
            .map(|s| s.default_analysis_type.clone())
            .unwrap_or_else(|| "general".to_string());

        match analyze_image(state, &image_id, &analysis_type, None, None).await {
            Ok(result) => {
                return Ok(UploadResult {
                    image_id,
                    status: "analyzed".to_string(),
                    analysis_id: Some(result.analysis_id.clone()),
                    message: "Image uploaded and analyzed successfully".to_string(),
                });
            }
            Err(e) => {
                // Update status to failed
                let _ = sqlx::query(&format!(
                    "UPDATE vision_images SET status = 'failed', updated_at = (NOW()) WHERE id = '{}'",
                    image_id
                ))
                .execute(&state.db)
                .await;

                return Ok(UploadResult {
                    image_id,
                    status: "upload_ok_analysis_failed".to_string(),
                    analysis_id: None,
                    message: format!("Image uploaded but analysis failed: {}", e),
                });
            }
        }
    }

    Ok(UploadResult {
        image_id,
        status: "uploaded".to_string(),
        analysis_id: None,
        message: "Image uploaded successfully".to_string(),
    })
}

pub async fn analyze_image(
    state: &AppState,
    image_id: &str,
    analysis_type: &str,
    custom_prompt: Option<String>,
    claude_model: Option<String>,
) -> Result<AnalyzeResult, String> {
    // Get image data
    let image_row = sqlx::query(
        "SELECT id, image_data, image_url, mime_type, organization_id FROM vision_images WHERE id = $1"
    )
    .bind(image_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Image {} not found", image_id))?;

    let image_data = image_row.try_get::<String, _>(1).unwrap_or_default();
    let image_url = image_row.try_get::<String, _>(2).unwrap_or_default();
    let mime_type = image_row.try_get::<String, _>(3).unwrap_or_default();
    let org_id = image_row.try_get::<String, _>(4).unwrap_or_default();

    // Update status to analyzing
    let _ = sqlx::query(&format!(
        "UPDATE vision_images SET status = 'analyzing', updated_at = (NOW()) WHERE id = '{}'",
        image_id
    ))
    .execute(&state.db)
    .await;

    let analysis_id = format!("vision-analysis-{}", Utc::now().timestamp_millis());
    let model = claude_model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
    let prompt = analysis_prompt(analysis_type, custom_prompt.as_deref());

    let started = Instant::now();

    // Try Claude multi-modal analysis
    let (result, confidence, objects_detected, text_extracted, summary, error_message) =
        match analyze_with_claude(state, &image_data, &image_url, &mime_type, &prompt, &model).await
        {
            Ok(analysis) => (
                analysis.result,
                analysis.confidence,
                analysis.objects,
                analysis.text_extracted,
                analysis.summary,
                None,
            ),
            Err(e) => {
                // Fallback: basic analysis without Claude
                let fallback = fallback_analysis(&mime_type, analysis_type);
                (
                    fallback.result,
                    0.3,
                    fallback.objects,
                    None,
                    Some("Basic analysis (Claude unavailable)".to_string()),
                    Some(e),
                )
            }
        };

    let processing_time_ms = started.elapsed().as_millis() as i64;

    // Save analysis result
    let safe_result = serde_json::to_string(&result).unwrap_or_default().replace('\'', "''");
    let safe_objects = serde_json::to_string(&objects_detected).unwrap_or_default().replace('\'', "''");
    let safe_text = text_extracted.as_ref().map(|t| t.replace('\'', "''")).unwrap_or_default();
    let safe_summary = summary.as_ref().map(|s| s.replace('\'', "''")).unwrap_or_default();
    let safe_error = error_message.as_ref().map(|e| e.replace('\'', "''")).unwrap_or_default();

    sqlx::query(&format!(
        "INSERT INTO vision_analyses (id, image_id, analysis_type, prompt, result, confidence, \
         objects_detected, text_extracted, summary, processing_time_ms, claude_model, error_message) \
         VALUES ('{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', {}, '{}', '{}')",
        analysis_id, image_id, analysis_type, prompt.replace('\'', "''"), safe_result, confidence,
        safe_objects, safe_text, safe_summary, processing_time_ms, model, safe_error
    ))
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to save analysis: {}", e))?;

    // Update image status
    let status = if error_message.is_some() { "completed_with_errors" } else { "completed" };
    let _ = sqlx::query(&format!(
        "UPDATE vision_images SET status = '{}', updated_at = (NOW()) WHERE id = '{}'",
        status, image_id
    ))
    .execute(&state.db)
    .await;

    Ok(AnalyzeResult {
        analysis_id,
        image_id: image_id.to_string(),
        analysis_type: analysis_type.to_string(),
        result,
        confidence,
        objects_detected,
        text_extracted,
        summary,
        processing_time_ms,
        claude_model: model,
    })
}

/// Internal struct for Claude analysis result
#[derive(Debug, Clone)]
struct ClaudeVisionAnalysis {
    result: Value,
    confidence: f64,
    objects: Vec<String>,
    text_extracted: Option<String>,
    summary: Option<String>,
}

async fn analyze_with_claude(
    state: &AppState,
    image_data: &str,
    image_url: &str,
    mime_type: &str,
    prompt: &str,
    model: &str,
) -> Result<ClaudeVisionAnalysis, String> {
    // Build the image source - prefer URL, fall back to base64 data
    let image_source = if !image_url.is_empty() {
        json!({
            "type": "image",
            "source": {
                "type": "url",
                "url": image_url
            }
        })
    } else if !image_data.is_empty() {
        // Determine media type from MIME type
        let media_type = mime_type.strip_prefix("image/").unwrap_or("jpeg");
        json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": media_type,
                "data": image_data
            }
        })
    } else {
        return Err("No image data or URL provided".to_string());
    };

    // Build messages payload with image content
    let messages = json!([
        {
            "role": "user",
            "content": [
                image_source,
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }
    ]);

    // Make the API call directly
    let config = match claude_client::ClaudeConfig::from_state(state) {
        Ok(c) => c,
        Err(e) => return Err(format!("Claude not configured: {}", e.message())),
    };

    let payload = json!({
        "model": model,
        "max_tokens": 4096,
        "messages": messages,
        "system": "You are an AI vision assistant. Analyze images carefully and provide structured JSON responses."
    });

    let url = format!("{}/v1/messages", config.base_url.trim_end_matches('/'));
    let response = state
        .http_client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", &config.api_version)
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(60))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to call Claude API: {}", e))?;

    let status = response.status();
    let body: Value = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    if !status.is_success() {
        let error_msg = body.get("error").and_then(|e| e.get("message")).and_then(Value::as_str).unwrap_or("Unknown error");
        return Err(format!("Claude API error (HTTP {}): {}", status.as_u16(), error_msg));
    }

    // Extract text from response
    let text = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| "No text content in Claude response".to_string())?;

    // Parse the JSON response from Claude
    let parsed = parse_claude_vision_response(text);

    Ok(ClaudeVisionAnalysis {
        result: parsed.result,
        confidence: parsed.confidence,
        objects: parsed.objects,
        text_extracted: parsed.text_extracted,
        summary: parsed.summary,
    })
}

struct ParsedVisionResponse {
    result: Value,
    confidence: f64,
    objects: Vec<String>,
    text_extracted: Option<String>,
    summary: Option<String>,
}

fn parse_claude_vision_response(text: &str) -> ParsedVisionResponse {
    // Try to parse as JSON first
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        let result = value.clone();
        let confidence = value.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.7);
        let objects: Vec<String> = value
            .get("objects_detected")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let text_extracted = value.get("text_extracted").and_then(|v| v.as_str()).map(|s| s.to_string());
        let summary = value.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());

        ParsedVisionResponse {
            result,
            confidence,
            objects,
            text_extracted,
            summary,
        }
    } else {
        // Try to extract JSON from markdown code blocks
        let trimmed = text.trim();
        if let Some(start) = trimmed.find("```") {
            let rest = &trimmed[start + 3..];
            let json_part = if let Some(end) = rest.find("```") {
                &rest[..end]
            } else {
                rest
            };
            // Skip "json" language marker if present
            let json_str = if let Some(after_json) = json_part.strip_prefix("json") {
                after_json.trim()
            } else {
                json_part.trim()
            };
            if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                let confidence = value.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.7);
                let objects: Vec<String> = value
                    .get("objects_detected")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let text_extracted = value.get("text_extracted").and_then(|v| v.as_str()).map(|s| s.to_string());
                let summary = value.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());

                return ParsedVisionResponse {
                    result: value,
                    confidence,
                    objects,
                    text_extracted,
                    summary,
                };
            }
        }

        // Fallback: treat the entire text as a summary
        ParsedVisionResponse {
            result: json!({"raw_text": text.trim()}),
            confidence: 0.5,
            objects: vec![],
            text_extracted: None,
            summary: Some(text.trim().to_string()),
        }
    }
}

fn fallback_analysis(mime_type: &str, analysis_type: &str) -> ParsedVisionResponse {
    ParsedVisionResponse {
        result: json!({
            "analysis_type": analysis_type,
            "mime_type": mime_type,
            "note": "Basic fallback analysis - Claude was unavailable"
        }),
        confidence: 0.3,
        objects: vec!["image".to_string()],
        text_extracted: None,
        summary: Some(format!("Image uploaded ({}). Claude was unavailable for detailed analysis.", mime_type)),
    }
}

pub async fn get_vision_settings(state: &AppState, organization_id: &str) -> Option<VisionSettings> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, auto_analyze, default_analysis_type, claude_model, \
         max_file_size_mb, allowed_mime_types, ocr_enabled, object_detection_enabled, created_at, updated_at \
         FROM vision_settings WHERE organization_id = $1 LIMIT 1"
    )
    .bind(organization_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    row.map(|r| VisionSettings {
        id: r.try_get(0).unwrap_or_default(),
        organization_id: r.try_get(1).unwrap_or_default(),
        enabled: r.try_get(2).unwrap_or(true),
        auto_analyze: r.try_get(3).unwrap_or(true),
        default_analysis_type: r.try_get(4).unwrap_or_else(|_| "general".to_string()),
        claude_model: r.try_get(5).unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
        max_file_size_mb: r.try_get(6).unwrap_or(10.0),
        allowed_mime_types: r.try_get(7).unwrap_or_else(|_| "[\"image/jpeg\", \"image/png\", \"image/webp\", \"image/gif\"]".to_string()),
        ocr_enabled: r.try_get(8).unwrap_or(true),
        object_detection_enabled: r.try_get(9).unwrap_or(true),
        created_at: r.try_get(10).unwrap_or_default(),
        updated_at: r.try_get(11).unwrap_or_default(),
    })
}

pub async fn update_vision_settings(
    state: &AppState,
    organization_id: &str,
    payload: &Value,
) -> Result<VisionSettings, String> {
    let mut enabled = true;
    let mut auto_analyze = true;
    let mut default_analysis_type = String::new();
    let mut claude_model = String::new();
    let mut max_file_size_mb = f64::NAN;
    let mut allowed_mime_types = String::new();
    let mut ocr_enabled = true;
    let mut object_detection_enabled = true;

    if let Some(Some(true)) | Some(Some(false)) = payload.get("enabled").map(Value::as_bool) {
        enabled = payload.get("enabled").unwrap().as_bool().unwrap();
    }
    if let Some(Some(true)) | Some(Some(false)) = payload.get("autoAnalyze").map(Value::as_bool) {
        auto_analyze = payload.get("autoAnalyze").unwrap().as_bool().unwrap();
    }
    if let Some(t) = payload.get("defaultAnalysisType").and_then(|v| v.as_str()) {
        default_analysis_type = t.to_string();
    }
    if let Some(m) = payload.get("claudeModel").and_then(|v| v.as_str()) {
        claude_model = m.to_string();
    }
    if let Some(s) = payload.get("maxFileSizeMb").and_then(|v| v.as_f64()) {
        max_file_size_mb = s;
    }
    if let Some(types) = payload.get("allowedMimeTypes") {
        allowed_mime_types = serde_json::to_string(types).unwrap_or_default();
    }
    if let Some(Some(true)) | Some(Some(false)) = payload.get("ocrEnabled").map(Value::as_bool) {
        ocr_enabled = payload.get("ocrEnabled").unwrap().as_bool().unwrap();
    }
    if let Some(Some(true)) | Some(Some(false)) = payload.get("objectDetectionEnabled").map(Value::as_bool) {
        object_detection_enabled = payload.get("objectDetectionEnabled").unwrap().as_bool().unwrap();
    }

    let safe_org = organization_id.replace('\'', "''");
    let mut updates = vec![format!("enabled = {}", if enabled { 1 } else { 0 })];
    updates.push(format!("auto_analyze = {}", if auto_analyze { 1 } else { 0 }));
    updates.push(format!("ocr_enabled = {}", if ocr_enabled { 1 } else { 0 }));
    updates.push(format!("object_detection_enabled = {}", if object_detection_enabled { 1 } else { 0 }));

    if !default_analysis_type.is_empty() {
        let safe_type = default_analysis_type.replace('\'', "''");
        updates.push(format!("default_analysis_type = '{}'", safe_type));
    }
    if !claude_model.is_empty() {
        let safe_model = claude_model.replace('\'', "''");
        updates.push(format!("claude_model = '{}'", safe_model));
    }
    if !max_file_size_mb.is_nan() {
        updates.push(format!("max_file_size_mb = {}", max_file_size_mb));
    }
    if !allowed_mime_types.is_empty() {
        let safe_types = allowed_mime_types.replace('\'', "''");
        updates.push(format!("allowed_mime_types = '{}'", safe_types));
    }
    updates.push("updated_at = (NOW())".to_string());

    let set_clause = updates.join(", ");
    sqlx::query(&format!(
        "UPDATE vision_settings SET {} WHERE organization_id = '{}'",
        set_clause, safe_org
    ))
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to update settings: {}", e))?;

    get_vision_settings(state, organization_id)
        .await
        .ok_or_else(|| "Settings not found after update".to_string())
}

pub async fn list_images(
    state: &AppState,
    organization_id: &str,
    status_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<VisionImage>, String> {
    let where_clause = if let Some(s) = status_filter {
        format!("WHERE organization_id = '{}' AND status = '{}'", organization_id, s)
    } else {
        format!("WHERE organization_id = '{}'", organization_id)
    };

    let rows = sqlx::query(
        &format!(
            "SELECT id, organization_id, conversation_id, contact_id, channel, image_url, \
             image_data, mime_type, file_size, status, created_at, updated_at \
             FROM vision_images {} ORDER BY created_at DESC LIMIT {}",
            where_clause, limit
        )
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Note: image_data is excluded from the list response for performance
    Ok(rows
        .iter()
        .map(|row| VisionImage {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            conversation_id: row.try_get(2).ok(),
            contact_id: row.try_get(3).ok(),
            channel: row.try_get(4).ok(),
            image_url: row.try_get(5).unwrap_or_default(),
            image_data: None, // Excluded from list for performance
            mime_type: row.try_get(7).unwrap_or_default(),
            file_size: row.try_get(8).unwrap_or(0i64),
            status: row.try_get(9).unwrap_or_default(),
            created_at: row.try_get(10).unwrap_or_default(),
            updated_at: row.try_get(11).unwrap_or_default(),
        })
        .collect())
}

pub async fn get_image_with_analysis(
    state: &AppState,
    image_id: &str,
) -> Result<(VisionImage, Vec<VisionAnalysis>), String> {
    // Get image
    let image_row = sqlx::query(
        "SELECT id, organization_id, conversation_id, contact_id, channel, image_url, \
         image_data, mime_type, file_size, status, created_at, updated_at \
         FROM vision_images WHERE id = $1 LIMIT 1"
    )
    .bind(image_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Image {} not found", image_id))?;

    let image = VisionImage {
        id: image_row.try_get(0).unwrap_or_default(),
        organization_id: image_row.try_get(1).unwrap_or_default(),
        conversation_id: image_row.try_get(2).ok(),
        contact_id: image_row.try_get(3).ok(),
        channel: image_row.try_get(4).ok(),
        image_url: image_row.try_get(5).unwrap_or_default(),
        image_data: image_row.try_get(6).ok(),
        mime_type: image_row.try_get(7).unwrap_or_default(),
        file_size: image_row.try_get(8).unwrap_or(0i64),
        status: image_row.try_get(9).unwrap_or_default(),
        created_at: image_row.try_get(10).unwrap_or_default(),
        updated_at: image_row.try_get(11).unwrap_or_default(),
    };

    // Get analyses for this image
    let analysis_rows = sqlx::query(
        "SELECT id, image_id, analysis_type, prompt, result, confidence, \
         objects_detected, text_extracted, summary, processing_time_ms, \
         claude_model, error_message, created_at \
         FROM vision_analyses WHERE image_id = $1 ORDER BY created_at DESC"
    )
    .bind(image_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let analyses: Vec<VisionAnalysis> = analysis_rows
        .iter()
        .map(|row| VisionAnalysis {
            id: row.try_get(0).unwrap_or_default(),
            image_id: row.try_get(1).unwrap_or_default(),
            analysis_type: row.try_get(2).unwrap_or_default(),
            prompt: row.try_get(3).ok(),
            result: row.try_get(4).unwrap_or_default(),
            confidence: row.try_get(5).unwrap_or(0.0),
            objects_detected: row.try_get(6).unwrap_or_default(),
            text_extracted: row.try_get(7).ok(),
            summary: row.try_get(8).ok(),
            processing_time_ms: row.try_get(9).unwrap_or(0i64),
            claude_model: row.try_get(10).unwrap_or_default(),
            error_message: row.try_get(11).ok(),
            created_at: row.try_get(12).unwrap_or_default(),
        })
        .collect();

    Ok((image, analyses))
}

pub async fn delete_image(state: &AppState, image_id: &str) -> Result<(), String> {
    // Delete analyses first
    let _ = sqlx::query("DELETE FROM vision_analyses WHERE image_id = $1")
        .bind(image_id)
        .execute(&state.db)
        .await;

    // Delete image
    let res = sqlx::query("DELETE FROM vision_images WHERE id = $1")
        .bind(image_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if res.rows_affected() == 0 {
        return Err(format!("Image {} not found", image_id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_summary_serialization() {
        let summary = VisionSummary {
            total_images: 42,
            total_analyses: 38,
            avg_confidence: 0.85,
            avg_processing_time_ms: 1200,
            analysis_type_breakdown: vec![AnalysisTypeCount {
                analysis_type: "screenshot".to_string(),
                count: 20,
            }],
            recent_analyses: vec![],
            enabled: true,
            auto_analyze: true,
        };
        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(json["totalImages"], 42);
        assert_eq!(json["avgConfidence"], 0.85);
    }

    #[test]
    fn test_analysis_prompt_general() {
        let prompt = analysis_prompt("general", None);
        assert!(prompt.contains("Analyze this image"));
        assert!(prompt.contains("structured JSON"));
    }

    #[test]
    fn test_analysis_prompt_screenshot() {
        let prompt = analysis_prompt("screenshot", None);
        assert!(prompt.contains("screenshot"));
        assert!(prompt.contains("error messages"));
    }

    #[test]
    fn test_custom_prompt_override() {
        let prompt = analysis_prompt("general", Some("Describe this product image"));
        assert_eq!(prompt, "Describe this product image");
    }

    #[test]
    fn test_fallback_analysis() {
        let result = fallback_analysis("image/png", "general");
        assert_eq!(result.confidence, 0.3);
        assert!(result.summary.is_some());
        assert!(result.summary.unwrap().contains("Claude was unavailable"));
    }
}
