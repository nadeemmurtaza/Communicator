//! Help Center Service (Phase 21) — public, customer-facing self-service portal.
//!
//! Backed by `help_center_categories`, `help_center_articles`,
//! `help_center_searches` and `help_center_settings` (migration 0029).

use crate::db::AppState;
use serde::Serialize;
use sqlx::Row;

const DEFAULT_ORG: &str = "org-default";

const ARTICLE_COLUMNS: &str = "id, organization_id, category_id, title, slug, content, author_id, status, \
    is_pinned, view_count, helpful_count, not_helpful_count, seo_title, seo_description, \
    created_at, updated_at, published_at";

#[derive(Debug, Clone, Serialize)]
pub struct HelpCategory {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub display_order: i32,
    pub is_visible: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpArticle {
    pub id: String,
    pub organization_id: String,
    pub category_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub author_id: Option<String>,
    pub status: String,
    pub is_pinned: bool,
    pub view_count: i32,
    pub helpful_count: i32,
    pub not_helpful_count: i32,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpCenterSettings {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub tagline: Option<String>,
    pub primary_color: Option<String>,
    pub enable_search: bool,
    pub enable_article_feedback: bool,
    pub footer_text: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArticleSearchResult {
    pub query: String,
    pub result_count: i64,
    pub results: Vec<HelpArticle>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackResult {
    pub article_id: String,
    pub helpful: bool,
    pub helpful_count: i32,
    pub not_helpful_count: i32,
}

fn datetime_string(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<chrono::DateTime<chrono::Utc>, _>(column)
        .map(|d| d.to_rfc3339())
        .unwrap_or_default()
}

fn optional_datetime_string(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column)
        .ok()
        .flatten()
        .map(|d| d.to_rfc3339())
}

fn row_to_category(row: &sqlx::postgres::PgRow) -> HelpCategory {
    HelpCategory {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        slug: row.try_get("slug").unwrap_or_default(),
        description: row.try_get("description").ok(),
        icon: row.try_get("icon").ok(),
        display_order: row.try_get("display_order").unwrap_or(0),
        is_visible: row.try_get("is_visible").unwrap_or(true),
        created_at: datetime_string(row, "created_at"),
        updated_at: datetime_string(row, "updated_at"),
    }
}

fn row_to_article(row: &sqlx::postgres::PgRow) -> HelpArticle {
    HelpArticle {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        category_id: row.try_get("category_id").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        slug: row.try_get("slug").unwrap_or_default(),
        content: row.try_get("content").unwrap_or_default(),
        author_id: row.try_get("author_id").ok(),
        status: row.try_get("status").unwrap_or_default(),
        is_pinned: row.try_get("is_pinned").unwrap_or(false),
        view_count: row.try_get("view_count").unwrap_or(0),
        helpful_count: row.try_get("helpful_count").unwrap_or(0),
        not_helpful_count: row.try_get("not_helpful_count").unwrap_or(0),
        seo_title: row.try_get("seo_title").ok(),
        seo_description: row.try_get("seo_description").ok(),
        created_at: datetime_string(row, "created_at"),
        updated_at: datetime_string(row, "updated_at"),
        published_at: optional_datetime_string(row, "published_at"),
    }
}

fn row_to_settings(row: &sqlx::postgres::PgRow) -> HelpCenterSettings {
    HelpCenterSettings {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        tagline: row.try_get("tagline").ok(),
        primary_color: row.try_get("primary_color").ok(),
        enable_search: row.try_get("enable_search").unwrap_or(true),
        enable_article_feedback: row.try_get("enable_article_feedback").unwrap_or(true),
        footer_text: row.try_get("footer_text").ok(),
        created_at: datetime_string(row, "created_at"),
        updated_at: datetime_string(row, "updated_at"),
    }
}

async fn get_article_by_id(state: &AppState, id: &str) -> Result<Option<HelpArticle>, String> {
    let row = sqlx::query(&format!("SELECT {} FROM help_center_articles WHERE id = $1", ARTICLE_COLUMNS))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|r| row_to_article(&r)))
}

/// Dashboard summary: counts and aggregate feedback across the help center.
pub async fn get_help_center_summary(state: &AppState) -> Result<serde_json::Value, String> {
    let category_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM help_center_categories")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let article_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM help_center_articles")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let published_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM help_center_articles WHERE status = 'published'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let total_views: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(view_count), 0) FROM help_center_articles")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_helpful: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(helpful_count), 0) FROM help_center_articles")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_not_helpful: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(not_helpful_count), 0) FROM help_center_articles")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    let search_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM help_center_searches")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "category_count": category_count,
        "article_count": article_count,
        "published_article_count": published_count,
        "total_views": total_views,
        "total_helpful_votes": total_helpful,
        "total_not_helpful_votes": total_not_helpful,
        "search_count": search_count,
    }))
}

/// Public overview used to embed the help center widget: settings, visible
/// categories and the currently pinned published articles.
pub async fn get_public_help_center(state: &AppState) -> Result<serde_json::Value, String> {
    let settings = get_settings(state).await?;
    let categories = list_categories(state, None, true).await?;

    let rows = sqlx::query(&format!(
        "SELECT {} FROM help_center_articles WHERE status = 'published' AND is_pinned = true ORDER BY updated_at DESC LIMIT 10",
        ARTICLE_COLUMNS
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let pinned: Vec<HelpArticle> = rows.iter().map(row_to_article).collect();

    Ok(serde_json::json!({
        "settings": settings,
        "categories": categories,
        "pinned_articles": pinned,
    }))
}

/// List categories for an organization, optionally restricted to visible ones.
pub async fn list_categories(
    state: &AppState,
    org_id: Option<&str>,
    visible_only: bool,
) -> Result<Vec<HelpCategory>, String> {
    let org = org_id.unwrap_or(DEFAULT_ORG);
    let query = if visible_only {
        "SELECT id, organization_id, name, slug, description, icon, display_order, is_visible, created_at, updated_at \
         FROM help_center_categories WHERE organization_id = $1 AND is_visible = true \
         ORDER BY display_order ASC, name ASC"
    } else {
        "SELECT id, organization_id, name, slug, description, icon, display_order, is_visible, created_at, updated_at \
         FROM help_center_categories WHERE organization_id = $1 \
         ORDER BY display_order ASC, name ASC"
    };

    let rows = sqlx::query(query)
        .bind(org)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(row_to_category).collect())
}

/// Create a new help center category.
pub async fn create_category(
    state: &AppState,
    org_id: &str,
    name: &str,
    slug: &str,
    description: Option<&str>,
    icon: Option<&str>,
    display_order: i32,
) -> Result<HelpCategory, String> {
    let id = format!("help-cat-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO help_center_categories (id, organization_id, name, slug, description, icon, display_order) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&id)
    .bind(org_id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(icon)
    .bind(display_order)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create category: {}", e))?;

    let row = sqlx::query(
        "SELECT id, organization_id, name, slug, description, icon, display_order, is_visible, created_at, updated_at \
         FROM help_center_categories WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row_to_category(&row))
}

/// List articles, optionally filtered by category and/or status.
pub async fn list_articles(
    state: &AppState,
    category_id: Option<&str>,
    status: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<HelpArticle>, String> {
    let mut query = format!("SELECT {} FROM help_center_articles WHERE 1=1", ARTICLE_COLUMNS);
    let mut string_binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(c) = category_id {
        query.push_str(&format!(" AND category_id = ${}", idx));
        string_binds.push(c.to_string());
        idx += 1;
    }
    if let Some(s) = status {
        query.push_str(&format!(" AND status = ${}", idx));
        string_binds.push(s.to_string());
        idx += 1;
    }
    query.push_str(&format!(
        " ORDER BY is_pinned DESC, created_at DESC LIMIT ${} OFFSET ${}",
        idx,
        idx + 1
    ));

    let mut q = sqlx::query(&query);
    for b in &string_binds {
        q = q.bind(b);
    }
    q = q.bind(limit as i64).bind(offset as i64);

    let rows = q.fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_article).collect())
}

/// Create a new help article (starts as a draft).
pub async fn create_article(
    state: &AppState,
    org_id: &str,
    category_id: &str,
    title: &str,
    slug: &str,
    content: &str,
    author_id: Option<&str>,
) -> Result<HelpArticle, String> {
    let id = format!("help-art-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO help_center_articles (id, organization_id, category_id, title, slug, content, author_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&id)
    .bind(org_id)
    .bind(category_id)
    .bind(title)
    .bind(slug)
    .bind(content)
    .bind(author_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create article: {}", e))?;

    get_article_by_id(state, &id)
        .await?
        .ok_or_else(|| "Article not found after insert".to_string())
}

/// Full-text-ish search (title/content ILIKE) over published articles, with
/// the query logged to `help_center_searches` for analytics.
pub async fn search_articles(state: &AppState, query: &str, limit: i32) -> Result<ArticleSearchResult, String> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(&format!(
        "SELECT {} FROM help_center_articles \
         WHERE status = 'published' AND (title ILIKE $1 OR content ILIKE $1) \
         ORDER BY is_pinned DESC, view_count DESC LIMIT $2",
        ARTICLE_COLUMNS
    ))
    .bind(&pattern)
    .bind(limit as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let results: Vec<HelpArticle> = rows.iter().map(row_to_article).collect();
    let result_count = results.len() as i64;

    let search_id = format!("help-search-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO help_center_searches (id, organization_id, query, result_count) VALUES ($1, $2, $3, $4)",
    )
    .bind(&search_id)
    .bind(DEFAULT_ORG)
    .bind(query)
    .bind(result_count)
    .execute(&state.db)
    .await
    .ok();

    Ok(ArticleSearchResult {
        query: query.to_string(),
        result_count,
        results,
    })
}

/// Fetch a published-or-not article by its public slug, incrementing its view count.
pub async fn get_article_by_slug(state: &AppState, slug: &str) -> Result<Option<HelpArticle>, String> {
    let row = sqlx::query(&format!("SELECT {} FROM help_center_articles WHERE slug = $1", ARTICLE_COLUMNS))
        .bind(slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let Some(r) = row else { return Ok(None) };
    let mut article = row_to_article(&r);

    sqlx::query("UPDATE help_center_articles SET view_count = view_count + 1 WHERE id = $1")
        .bind(&article.id)
        .execute(&state.db)
        .await
        .ok();
    article.view_count += 1;

    Ok(Some(article))
}

/// Update an article's editable fields. Every field is optional and only
/// overwrites the column when supplied. Transitioning `status` to
/// `'published'` for the first time stamps `published_at`.
pub async fn update_article(
    state: &AppState,
    article_id: &str,
    title: Option<&str>,
    content: Option<&str>,
    status: Option<&str>,
    is_pinned: Option<bool>,
    seo_title: Option<&str>,
    seo_description: Option<&str>,
) -> Result<Option<HelpArticle>, String> {
    let row = sqlx::query(&format!(
        "UPDATE help_center_articles SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            status = COALESCE($3, status),
            is_pinned = COALESCE($4, is_pinned),
            seo_title = COALESCE($5, seo_title),
            seo_description = COALESCE($6, seo_description),
            published_at = CASE WHEN $3 = 'published' AND published_at IS NULL THEN NOW() ELSE published_at END,
            updated_at = NOW()
         WHERE id = $7
         RETURNING {}",
        ARTICLE_COLUMNS
    ))
    .bind(title)
    .bind(content)
    .bind(status)
    .bind(is_pinned)
    .bind(seo_title)
    .bind(seo_description)
    .bind(article_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Failed to update article: {}", e))?;

    Ok(row.map(|r| row_to_article(&r)))
}

/// Delete an article. Returns whether a row was actually removed.
pub async fn delete_article(state: &AppState, article_id: &str) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM help_center_articles WHERE id = $1")
        .bind(article_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete article: {}", e))?;
    Ok(result.rows_affected() > 0)
}

/// Record a helpful/not-helpful vote on an article and return the new tallies.
pub async fn record_article_feedback(
    state: &AppState,
    article_id: &str,
    helpful: bool,
) -> Result<FeedbackResult, String> {
    let column = if helpful { "helpful_count" } else { "not_helpful_count" };
    let query = format!(
        "UPDATE help_center_articles SET {column} = {column} + 1, updated_at = NOW() \
         WHERE id = $1 RETURNING helpful_count, not_helpful_count",
        column = column
    );
    let row = sqlx::query(&query)
        .bind(article_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("Failed to record feedback: {}", e))?
        .ok_or_else(|| "Article not found".to_string())?;

    Ok(FeedbackResult {
        article_id: article_id.to_string(),
        helpful,
        helpful_count: row.try_get("helpful_count").unwrap_or(0),
        not_helpful_count: row.try_get("not_helpful_count").unwrap_or(0),
    })
}

/// Get this organization's help center settings, seeding a default row the
/// first time it's requested.
pub async fn get_settings(state: &AppState) -> Result<HelpCenterSettings, String> {
    let row = sqlx::query(
        "SELECT id, organization_id, name, tagline, primary_color, enable_search, enable_article_feedback, \
         footer_text, created_at, updated_at FROM help_center_settings WHERE organization_id = $1",
    )
    .bind(DEFAULT_ORG)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        return Ok(row_to_settings(&r));
    }

    let id = format!("help-settings-{}", uuid::Uuid::new_v4());
    let row = sqlx::query(
        "INSERT INTO help_center_settings (id, organization_id, name) VALUES ($1, $2, $3) \
         ON CONFLICT (organization_id) DO UPDATE SET organization_id = EXCLUDED.organization_id \
         RETURNING id, organization_id, name, tagline, primary_color, enable_search, enable_article_feedback, \
                   footer_text, created_at, updated_at",
    )
    .bind(&id)
    .bind(DEFAULT_ORG)
    .bind("Help Center")
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to seed help center settings: {}", e))?;

    Ok(row_to_settings(&row))
}

/// Update help center settings. Every field is optional and only overwrites
/// the column when supplied.
pub async fn update_settings(
    state: &AppState,
    name: Option<&str>,
    tagline: Option<&str>,
    primary_color: Option<&str>,
    enable_search: Option<bool>,
    enable_article_feedback: Option<bool>,
    footer_text: Option<&str>,
) -> Result<HelpCenterSettings, String> {
    // Make sure a row exists before trying to update it.
    get_settings(state).await?;

    let row = sqlx::query(
        "UPDATE help_center_settings SET
            name = COALESCE($1, name),
            tagline = COALESCE($2, tagline),
            primary_color = COALESCE($3, primary_color),
            enable_search = COALESCE($4, enable_search),
            enable_article_feedback = COALESCE($5, enable_article_feedback),
            footer_text = COALESCE($6, footer_text),
            updated_at = NOW()
         WHERE organization_id = $7
         RETURNING id, organization_id, name, tagline, primary_color, enable_search, enable_article_feedback, \
                   footer_text, created_at, updated_at",
    )
    .bind(name)
    .bind(tagline)
    .bind(primary_color)
    .bind(enable_search)
    .bind(enable_article_feedback)
    .bind(footer_text)
    .bind(DEFAULT_ORG)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to update settings: {}", e))?;

    Ok(row_to_settings(&row))
}
