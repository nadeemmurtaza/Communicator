use newax_conversation_search as cs;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.conversation_search.v1");
}

use proto::{
    conversation_search_service_server::{ConversationSearchService, ConversationSearchServiceServer},
    CreateSavedSearchRequest, Empty, HistoryRequest, JsonResponse, ListSavedSearchesRequest,
    PinSavedSearchRequest, SavedSearch as ProtoSavedSearch, SavedSearchIdRequest, SavedSearchesResponse,
    SearchAnalytics as ProtoAnalytics, SearchAnalyticsResponse, SearchAnalyticsSnapshot as ProtoSnapshot,
    SearchHistoryEntry as ProtoHistoryEntry, SearchHistoryResponse, SearchRequest, SearchResponse as ProtoSearchResponse,
    SearchResult as ProtoSearchResult, SearchSettings as ProtoSettings, UpdateSettingsRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn result_to_proto(x: cs::SearchResult) -> ProtoSearchResult {
    ProtoSearchResult {
        search_type: x.search_type,
        id: x.id,
        title: x.title,
        snippet: x.snippet,
        metadata_json: x.metadata.to_string(),
        matched_fields: x.matched_fields,
        score: x.score,
    }
}

fn search_response_to_proto(x: cs::SearchResponse) -> ProtoSearchResponse {
    ProtoSearchResponse {
        query: x.query,
        total_results: x.total_results,
        conversations: x.conversations.into_iter().map(result_to_proto).collect(),
        messages: x.messages.into_iter().map(result_to_proto).collect(),
        contacts: x.contacts.into_iter().map(result_to_proto).collect(),
        duration_ms: x.duration_ms,
    }
}

fn saved_search_to_proto(x: cs::SavedSearch) -> ProtoSavedSearch {
    ProtoSavedSearch {
        id: x.id,
        org_id: x.org_id,
        name: x.name,
        query: x.query,
        search_type: x.search_type,
        filters: x.filters,
        created_by: x.created_by,
        created_at: x.created_at,
        updated_at: x.updated_at,
        is_pinned: x.is_pinned,
        usage_count: x.usage_count,
    }
}

fn history_entry_to_proto(x: cs::SearchHistoryEntry) -> ProtoHistoryEntry {
    ProtoHistoryEntry {
        id: x.id,
        org_id: x.org_id,
        query: x.query,
        search_type: x.search_type,
        result_count: x.result_count,
        filters: x.filters,
        performed_by: x.performed_by,
        performed_at: x.performed_at,
        duration_ms: x.duration_ms,
    }
}

fn analytics_to_proto(x: cs::SearchAnalytics) -> ProtoAnalytics {
    ProtoAnalytics {
        id: x.id,
        org_id: x.org_id,
        window_start: x.window_start,
        window_end: x.window_end,
        total_searches: x.total_searches,
        avg_result_count: x.avg_result_count,
        avg_duration_ms: x.avg_duration_ms,
        zero_result_searches: x.zero_result_searches,
        top_queries: x.top_queries,
        top_search_types: x.top_search_types,
        created_at: x.created_at,
    }
}

fn settings_to_proto(x: cs::SearchSettings) -> ProtoSettings {
    ProtoSettings {
        id: x.id,
        org_id: x.org_id,
        max_results: x.max_results,
        search_message_content: x.search_message_content,
        search_contact_details: x.search_contact_details,
        search_conversation_metadata: x.search_conversation_metadata,
        track_search_history: x.track_search_history,
        auto_suggest: x.auto_suggest,
        min_query_length: x.min_query_length,
        highlight_matches: x.highlight_matches,
        created_at: x.created_at,
        updated_at: x.updated_at,
    }
}

#[tonic::async_trait]
impl ConversationSearchService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let summary = cs::ConversationSearchService::new(self.pool.clone())
            .get_summary()
            .await
            .map_err(to_status)?;
        Ok(Response::new(JsonResponse {
            json: serde_json::to_string(&summary).map_err(to_status)?,
        }))
    }

    async fn search(&self, req: Request<SearchRequest>) -> Result<Response<ProtoSearchResponse>, Status> {
        let r = req.into_inner();
        let search_type = if r.search_type.is_empty() { "all".to_string() } else { r.search_type };
        let limit = if r.limit <= 0 { 50 } else { r.limit };
        let filters = serde_json::json!({});
        let result = cs::ConversationSearchService::new(self.pool.clone())
            .search(&r.q, &search_type, &filters, limit)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(search_response_to_proto(result)))
    }

    async fn list_saved_searches(
        &self,
        req: Request<ListSavedSearchesRequest>,
    ) -> Result<Response<SavedSearchesResponse>, Status> {
        let r = req.into_inner();
        let searches = cs::ConversationSearchService::new(self.pool.clone())
            .list_saved_searches(r.search_type.as_deref())
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(SavedSearchesResponse {
            searches: searches.into_iter().map(saved_search_to_proto).collect(),
        }))
    }

    async fn create_saved_search(
        &self,
        req: Request<CreateSavedSearchRequest>,
    ) -> Result<Response<ProtoSavedSearch>, Status> {
        let r = req.into_inner();
        let filters = r
            .filters_json
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
        let logic_req = cs::CreateSavedSearchRequest {
            name: r.name,
            query: r.query,
            search_type: r.search_type,
            filters,
        };
        let search = cs::ConversationSearchService::new(self.pool.clone())
            .create_saved_search(&logic_req, "system")
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(saved_search_to_proto(search)))
    }

    async fn get_saved_search(&self, req: Request<SavedSearchIdRequest>) -> Result<Response<ProtoSavedSearch>, Status> {
        let id = req.into_inner().id;
        let search = cs::ConversationSearchService::new(self.pool.clone())
            .get_saved_search(&id)
            .await
            .map_err(Status::not_found)?;
        Ok(Response::new(saved_search_to_proto(search)))
    }

    async fn delete_saved_search(&self, req: Request<SavedSearchIdRequest>) -> Result<Response<Empty>, Status> {
        let id = req.into_inner().id;
        cs::ConversationSearchService::new(self.pool.clone())
            .delete_saved_search(&id)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(Empty {}))
    }

    async fn pin_saved_search(&self, req: Request<PinSavedSearchRequest>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        cs::ConversationSearchService::new(self.pool.clone())
            .pin_saved_search(&r.id, r.pinned)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(Empty {}))
    }

    async fn use_saved_search(&self, req: Request<SavedSearchIdRequest>) -> Result<Response<ProtoSearchResponse>, Status> {
        let id = req.into_inner().id;
        let result = cs::ConversationSearchService::new(self.pool.clone())
            .use_saved_search(&id)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(search_response_to_proto(result)))
    }

    async fn get_search_history(&self, req: Request<HistoryRequest>) -> Result<Response<SearchHistoryResponse>, Status> {
        let limit = req.into_inner().limit;
        let history = cs::ConversationSearchService::new(self.pool.clone())
            .get_search_history(limit)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(SearchHistoryResponse {
            history: history.into_iter().map(history_entry_to_proto).collect(),
        }))
    }

    async fn clear_search_history(&self, _req: Request<Empty>) -> Result<Response<Empty>, Status> {
        cs::ConversationSearchService::new(self.pool.clone())
            .clear_search_history()
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_analytics(&self, _req: Request<Empty>) -> Result<Response<SearchAnalyticsResponse>, Status> {
        let analytics = cs::ConversationSearchService::new(self.pool.clone())
            .get_analytics()
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(SearchAnalyticsResponse {
            analytics: analytics.into_iter().map(analytics_to_proto).collect(),
        }))
    }

    async fn generate_analytics_snapshot(&self, _req: Request<Empty>) -> Result<Response<ProtoSnapshot>, Status> {
        let snapshot = cs::ConversationSearchService::new(self.pool.clone())
            .generate_analytics_snapshot()
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(ProtoSnapshot {
            window_start: snapshot.window_start,
            window_end: snapshot.window_end,
            total_searches: snapshot.total_searches,
            avg_result_count: snapshot.avg_result_count,
            avg_duration_ms: snapshot.avg_duration_ms,
            zero_result_searches: snapshot.zero_result_searches,
        }))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<ProtoSettings>, Status> {
        let settings = cs::ConversationSearchService::new(self.pool.clone())
            .get_settings()
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(settings_to_proto(settings)))
    }

    async fn update_settings(&self, req: Request<UpdateSettingsRequest>) -> Result<Response<ProtoSettings>, Status> {
        let r = req.into_inner();
        let service = cs::ConversationSearchService::new(self.pool.clone());
        let current = service.get_settings().await.map_err(Status::internal)?;

        let updated = cs::SearchSettings {
            id: current.id.clone(),
            org_id: current.org_id.clone(),
            max_results: r.max_results.unwrap_or(current.max_results),
            search_message_content: r.search_message_content.unwrap_or(current.search_message_content),
            search_contact_details: r.search_contact_details.unwrap_or(current.search_contact_details),
            search_conversation_metadata: r
                .search_conversation_metadata
                .unwrap_or(current.search_conversation_metadata),
            track_search_history: r.track_search_history.unwrap_or(current.track_search_history),
            auto_suggest: r.auto_suggest.unwrap_or(current.auto_suggest),
            min_query_length: r.min_query_length.unwrap_or(current.min_query_length),
            highlight_matches: r.highlight_matches.unwrap_or(current.highlight_matches),
            created_at: current.created_at,
            updated_at: current.updated_at,
        };

        let result = service.update_settings(&updated).await.map_err(Status::internal)?;
        Ok(Response::new(settings_to_proto(result)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_conversation_search=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("CONVERSATION_SEARCH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50067);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-conversation-search starting (gRPC)");
    Server::builder()
        .add_service(ConversationSearchServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
