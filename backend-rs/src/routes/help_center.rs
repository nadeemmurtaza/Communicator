//! Public Help Center API routes (Phase 21).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::AppState;
use crate::services::help_center;

// ——— Query Params ———

#[derive(Deserialize)]
struct ListCategoriesQuery {
    organization_id: Option<String>,
    visible_only: Option<bool>,
}

#[derive(Deserialize)]
struct ListArticlesQuery {
    category_id: Option<String>,
    status: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i32>,
}

// ——— Request Bodies ———

#[derive(Deserialize)]
struct CreateCategoryBody {
    organization_id: String,
    name: String,
    slug: String,
    description: Option<String>,
    icon: Option<String>,
    display_order: Option<i32>,
}

#[derive(Deserialize)]
struct CreateArticleBody {
    organization_id: String,
    category_id: String,
    title: String,
    slug: String,
    content: String,
    author_id: Option<String>,
}

#[derive(Deserialize)]
struct UpdateArticleBody {
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
    is_pinned: Option<bool>,
    seo_title: Option<String>,
    seo_description: Option<String>,
}

#[derive(Deserialize)]
struct FeedbackBody {
    helpful: bool,
}

#[derive(Deserialize)]
struct UpdateSettingsBody {
    name: Option<String>,
    tagline: Option<String>,
    primary_color: Option<String>,
    enable_search: Option<bool>,
    enable_article_feedback: Option<bool>,
    footer_text: Option<String>,
}

// ——— Endpoints ———

/// GET /help-center/summary — Dashboard summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let summary = help_center::get_help_center_summary(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// GET /help-center/public — Public help center overview (for embedding)
pub async fn get_public(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let data = help_center::get_public_help_center(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(data))
}

/// GET /help-center/categories — List categories
pub async fn list_categories(
    State(state): State<AppState>,
    Query(params): Query<ListCategoriesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let categories = help_center::list_categories(
        &state,
        params.organization_id.as_deref(),
        params.visible_only.unwrap_or(true),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"categories": categories})))
}

/// POST /help-center/categories — Create category
pub async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategoryBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let category = help_center::create_category(
        &state,
        &body.organization_id,
        &body.name,
        &body.slug,
        body.description.as_deref(),
        body.icon.as_deref(),
        body.display_order.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(category).unwrap_or_default()))
}

/// GET /help-center/articles — List articles
pub async fn list_articles(
    State(state): State<AppState>,
    Query(params): Query<ListArticlesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let articles = help_center::list_articles(
        &state,
        params.category_id.as_deref(),
        params.status.as_deref(),
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"articles": articles})))
}

/// POST /help-center/articles — Create article
pub async fn create_article(
    State(state): State<AppState>,
    Json(body): Json<CreateArticleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = help_center::create_article(
        &state,
        &body.organization_id,
        &body.category_id,
        &body.title,
        &body.slug,
        &body.content,
        body.author_id.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(article).unwrap_or_default()))
}

/// GET /help-center/articles/search — Search articles
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = help_center::search_articles(&state, &params.q, params.limit.unwrap_or(20))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /help-center/articles/slug/:slug — Get article by slug (public)
pub async fn get_article_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = help_center::get_article_by_slug(&state, &slug)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match article {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err((StatusCode::NOT_FOUND, "Article not found".to_string())),
    }
}

/// PUT /help-center/articles/:id — Update article
pub async fn update_article(
    State(state): State<AppState>,
    Path(article_id): Path<String>,
    Json(body): Json<UpdateArticleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = help_center::update_article(
        &state,
        &article_id,
        body.title.as_deref(),
        body.content.as_deref(),
        body.status.as_deref(),
        body.is_pinned,
        body.seo_title.as_deref(),
        body.seo_description.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match article {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err((StatusCode::NOT_FOUND, "Article not found".to_string())),
    }
}

/// DELETE /help-center/articles/:id — Delete article
pub async fn delete_article(
    State(state): State<AppState>,
    Path(article_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let deleted = help_center::delete_article(&state, &article_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(Json(json!({"deleted": true})))
    } else {
        Err((StatusCode::NOT_FOUND, "Article not found".to_string()))
    }
}

/// POST /help-center/articles/:id/feedback — Record article feedback
pub async fn record_feedback(
    State(state): State<AppState>,
    Path(article_id): Path<String>,
    Json(body): Json<FeedbackBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = help_center::record_article_feedback(&state, &article_id, body.helpful)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /help-center/settings — Get help center settings
pub async fn get_settings(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = help_center::get_settings(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}

/// PUT /help-center/settings — Update help center settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = help_center::update_settings(
        &state,
        body.name.as_deref(),
        body.tagline.as_deref(),
        body.primary_color.as_deref(),
        body.enable_search,
        body.enable_article_feedback,
        body.footer_text.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}
