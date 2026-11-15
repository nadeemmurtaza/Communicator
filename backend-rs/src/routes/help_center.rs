//! Public Help Center API routes -- thin gRPC client for svc-help-center.
//! See crates/help-center and services/help-center. REST paths and
//! payload shapes are unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use newax_proto::help_center::v1::{
    help_center_service_client::HelpCenterServiceClient, ArticleIdRequest, CreateArticleRequest,
    CreateCategoryRequest, Empty, FeedbackRequest, ListArticlesRequest, ListCategoriesRequest, SearchRequest,
    SlugRequest, UpdateArticleRequest, UpdateSettingsRequest,
};
use serde::Deserialize;
use serde_json::json;
use tonic::transport::Channel;

use crate::db::AppState;

async fn client() -> Result<HelpCenterServiceClient<Channel>, (StatusCode, String)> {
    let url = std::env::var("HELP_CENTER_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50064".to_string());
    HelpCenterServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("help-center service unreachable: {e}")))
}

fn json_value(s: String) -> serde_json::Value {
    serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
}

fn grpc_err(status: tonic::Status) -> (StatusCode, String) {
    let code = match status.code() {
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, status.message().to_string())
}

#[derive(Deserialize)]
pub struct ListCategoriesQuery {
    organization_id: Option<String>,
    visible_only: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListArticlesQuery {
    category_id: Option<String>,
    status: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateCategoryBody {
    organization_id: String,
    name: String,
    slug: String,
    description: Option<String>,
    icon: Option<String>,
    display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateArticleBody {
    organization_id: String,
    category_id: String,
    title: String,
    slug: String,
    content: String,
    author_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateArticleBody {
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
    is_pinned: Option<bool>,
    seo_title: Option<String>,
    seo_description: Option<String>,
}

#[derive(Deserialize)]
pub struct FeedbackBody {
    helpful: bool,
}

#[derive(Deserialize)]
pub struct UpdateSettingsBody {
    name: Option<String>,
    tagline: Option<String>,
    primary_color: Option<String>,
    enable_search: Option<bool>,
    enable_article_feedback: Option<bool>,
    footer_text: Option<String>,
}

pub async fn get_summary(State(_s): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_summary(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_public(State(_s): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_public(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn list_categories(
    State(_s): State<AppState>,
    Query(params): Query<ListCategoriesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = ListCategoriesRequest { organization_id: params.organization_id, visible_only: params.visible_only };
    let r = c.list_categories(req).await.map_err(grpc_err)?;
    Ok(Json(json!({"categories": json_value(r.into_inner().json)})))
}

pub async fn create_category(
    State(_s): State<AppState>,
    Json(body): Json<CreateCategoryBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = CreateCategoryRequest {
        organization_id: body.organization_id,
        name: body.name,
        slug: body.slug,
        description: body.description,
        icon: body.icon,
        display_order: body.display_order,
    };
    let r = c.create_category(req).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn list_articles(
    State(_s): State<AppState>,
    Query(params): Query<ListArticlesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = ListArticlesRequest {
        category_id: params.category_id,
        status: params.status,
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
    };
    let r = c.list_articles(req).await.map_err(grpc_err)?;
    Ok(Json(json!({"articles": json_value(r.into_inner().json)})))
}

pub async fn create_article(
    State(_s): State<AppState>,
    Json(body): Json<CreateArticleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = CreateArticleRequest {
        organization_id: body.organization_id,
        category_id: body.category_id,
        title: body.title,
        slug: body.slug,
        content: body.content,
        author_id: body.author_id,
    };
    let r = c.create_article(req).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn search(
    State(_s): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.search_articles(SearchRequest { q: params.q, limit: params.limit.unwrap_or(20) }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_article_by_slug(
    State(_s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_article_by_slug(SlugRequest { slug }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn update_article(
    State(_s): State<AppState>,
    Path(article_id): Path<String>,
    Json(body): Json<UpdateArticleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = UpdateArticleRequest {
        article_id,
        title: body.title,
        content: body.content,
        status: body.status,
        is_pinned: body.is_pinned,
        seo_title: body.seo_title,
        seo_description: body.seo_description,
    };
    let r = c.update_article(req).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn delete_article(
    State(_s): State<AppState>,
    Path(article_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.delete_article(ArticleIdRequest { article_id }).await.map_err(grpc_err)?;
    Ok(Json(json!({"deleted": r.into_inner().deleted})))
}

pub async fn record_feedback(
    State(_s): State<AppState>,
    Path(article_id): Path<String>,
    Json(body): Json<FeedbackBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.record_feedback(FeedbackRequest { article_id, helpful: body.helpful }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_settings(State(_s): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_settings(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn update_settings(
    State(_s): State<AppState>,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = UpdateSettingsRequest {
        name: body.name,
        tagline: body.tagline,
        primary_color: body.primary_color,
        enable_search: body.enable_search,
        enable_article_feedback: body.enable_article_feedback,
        footer_text: body.footer_text,
    };
    let r = c.update_settings(req).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}
