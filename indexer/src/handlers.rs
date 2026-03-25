use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Response},
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    AuditQuery, DiscoveryQuery, Event, EventQuery, GlobalSearchQuery, GlobalSearchResponse,
    HistoryQuery, NewAuditLog, PagedResponse, ReplayRequest, RetentionRequest, RetentionResponse,
    SuggestionQuery, TradeSearchQuery, WebSocketMessage,
};
use crate::websocket::WebSocketManager;
use crate::database::Database;

/// Default page size — kept small for mobile clients.
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

/// GET / — API discovery / navigation index.
pub async fn api_index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "StellarEscrow Indexer API",
        "version": "1.0.0",
        "endpoints": {
            "health":          "GET  /health",
            "events":          "GET  /events?limit=20&offset=0&event_type=&trade_id=&from_ledger=&to_ledger=",
            "event_by_id":     "GET  /events/:id",
            "events_by_trade": "GET  /events/trade/:trade_id",
            "events_by_type":  "GET  /events/type/:event_type",
            "replay":          "POST /events/replay  {from_ledger, to_ledger?}",
            "websocket":       "GET  /ws",
            "help":            "GET  /help",
            "audit_ingest":    "POST /audit  {actor, category, action, outcome, ...}",
            "audit_query":     "GET  /audit?actor=&category=&action=&outcome=&severity=&from=&to=&limit=&offset=",
            "audit_stats":     "GET  /audit/stats",
            "audit_purge":     "DELETE /audit/purge  {older_than_days?}"
        }
    }))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: events,
        total,
        limit,
        offset,
    }))
}

pub async fn get_event_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Event>, AppError> {
    let event = state.database.get_event_by_id(id).await?;
    Ok(Json(event))
}

pub async fn get_events_by_trade_id(
    Path(trade_id): Path<u64>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { trade_id: Some(trade_id), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { event_type: Some(event_type), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state.database.get_events_in_range(request.from_ledger, to_ledger, "contract_id").await?;

    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    Ok(Json(json!({ "replayed": events.len(), "from_ledger": request.from_ledger, "to_ledger": request.to_ledger })))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
}

pub async fn global_search(
    Query(params): Query<GlobalSearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<GlobalSearchResponse>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 50);
    let trade_query = TradeSearchQuery {
        q: Some(params.q.clone()),
        status: None,
        seller: None,
        buyer: None,
        min_amount: None,
        max_amount: None,
        limit: Some(limit),
        offset: Some(0),
    };

    let user_query = DiscoveryQuery {
        q: Some(params.q.clone()),
        role: Some("user".to_string()),
        limit: Some(limit),
    };
    let arb_query = DiscoveryQuery {
        q: Some(params.q.clone()),
        role: Some("arbitrator".to_string()),
        limit: Some(limit),
    };

    let trades = state.database.search_trades(&trade_query).await?;
    let users = state.database.discover_entities(&user_query).await?;
    let arbitrators = state.database.discover_entities(&arb_query).await?;
    let suggestions = state.database.get_search_suggestions(&params.q, 10).await?;

    state.database.record_search(&params.q, "global").await?;

    Ok(Json(GlobalSearchResponse {
        trades,
        users,
        arbitrators,
        suggestions,
    }))
}

pub async fn search_trades(
    Query(params): Query<TradeSearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::TradeSearchResult>>, AppError> {
    let rows = state.database.search_trades(&params).await?;
    if let Some(q) = params.q {
        if !q.is_empty() {
            state.database.record_search(&q, "trades").await?;
        }
    }
    Ok(Json(rows))
}

pub async fn discover_entities(
    Query(params): Query<DiscoveryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::DiscoveryResult>>, AppError> {
    let rows = state.database.discover_entities(&params).await?;
    if let Some(q) = params.q {
        if !q.is_empty() {
            state.database.record_search(&q, "discovery").await?;
        }
    }
    Ok(Json(rows))
}

pub async fn search_suggestions(
    Query(params): Query<SuggestionQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::SearchSuggestion>>, AppError> {
    let rows = state
        .database
        .get_search_suggestions(&params.q, params.limit.unwrap_or(10))
        .await?;
    Ok(Json(rows))
}

pub async fn search_history(
    Query(params): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::SearchHistoryEntry>>, AppError> {
    let rows = state
        .database
        .get_search_history(params.limit.unwrap_or(20))
        .await?;
    Ok(Json(rows))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub ws_manager: Arc<WebSocketManager>,
}

// =============================================================================
// Audit Log Handlers
// =============================================================================

/// POST /audit — ingest a new audit log entry.
pub async fn create_audit_log(
    State(state): State<AppState>,
    Json(body): Json<NewAuditLog>,
) -> Result<Json<crate::models::AuditLog>, AppError> {
    let log = state.database.insert_audit_log(&body).await?;
    Ok(Json(log))
}

/// GET /audit — query audit logs with optional filters.
pub async fn query_audit_logs(
    Query(params): Query<AuditQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<crate::models::AuditLog>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let offset = params.offset.unwrap_or(0).max(0);
    let q = AuditQuery { limit: Some(limit), offset: Some(offset), ..params };

    let (logs, total) = tokio::try_join!(
        state.database.query_audit_logs(&q),
        state.database.count_audit_logs(&q),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: logs,
        total,
        limit,
        offset,
    }))
}

/// GET /audit/stats — aggregated analysis (counts by category, outcome, severity, top actors/actions).
pub async fn audit_stats(
    State(state): State<AppState>,
) -> Result<Json<crate::models::AuditStats>, AppError> {
    let stats = state.database.audit_stats().await?;
    Ok(Json(stats))
}

/// DELETE /audit/purge — apply retention policy, deleting logs older than N days.
pub async fn purge_audit_logs(
    State(state): State<AppState>,
    Json(body): Json<RetentionRequest>,
) -> Result<Json<RetentionResponse>, AppError> {
    let days = body.older_than_days.unwrap_or(90).clamp(1, 365);
    let deleted = state.database.purge_old_audit_logs(days).await?;
    Ok(Json(RetentionResponse { deleted, older_than_days: days }))
}
