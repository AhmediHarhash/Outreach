//! Lead routes

use axum::{
    extract::{Extension, Path, Query},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    models::{CreateLeadRequest, Lead, LeadListQuery, LeadListResponse, UpdateLeadRequest},
    AppState,
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_leads).post(create_lead))
        .route("/:id", get(get_lead).put(update_lead).delete(delete_lead))
}

/// List leads with filtering and pagination
async fn list_leads(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<LeadListQuery>,
) -> ApiResult<Json<LeadListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");

    // Build dynamic query
    let leads = sqlx::query_as!(
        Lead,
        r#"
        SELECT * FROM leads
        WHERE user_id = $1
          AND ($2::text IS NULL OR status = $2)
          AND ($3::int IS NULL OR priority >= $3)
          AND ($4::text IS NULL OR
               company_name ILIKE '%' || $4 || '%' OR
               contact_name ILIKE '%' || $4 || '%' OR
               contact_email ILIKE '%' || $4 || '%')
        ORDER BY
            CASE WHEN $5 = 'created_at' AND $6 = 'desc' THEN created_at END DESC,
            CASE WHEN $5 = 'created_at' AND $6 = 'asc' THEN created_at END ASC,
            CASE WHEN $5 = 'priority' AND $6 = 'desc' THEN priority END DESC,
            CASE WHEN $5 = 'priority' AND $6 = 'asc' THEN priority END ASC,
            CASE WHEN $5 = 'company_name' AND $6 = 'desc' THEN company_name END DESC,
            CASE WHEN $5 = 'company_name' AND $6 = 'asc' THEN company_name END ASC,
            created_at DESC
        LIMIT $7 OFFSET $8
        "#,
        auth.id,
        query.status,
        query.priority,
        query.search,
        sort_by,
        sort_order,
        per_page as i64,
        offset as i64
    )
    .fetch_all(state.db.pool())
    .await?;

    // Get total count
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM leads
        WHERE user_id = $1
          AND ($2::text IS NULL OR status = $2)
          AND ($3::int IS NULL OR priority >= $3)
          AND ($4::text IS NULL OR
               company_name ILIKE '%' || $4 || '%' OR
               contact_name ILIKE '%' || $4 || '%' OR
               contact_email ILIKE '%' || $4 || '%')
        "#,
        auth.id,
        query.status,
        query.priority,
        query.search
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(LeadListResponse {
        leads,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// Create a new lead
async fn create_lead(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateLeadRequest>,
) -> ApiResult<Json<Lead>> {
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    let estimated_value = req.estimated_value.map(|v| {
        sqlx::types::BigDecimal::try_from(v).unwrap_or_default()
    });

    let lead = sqlx::query_as!(
        Lead,
        r#"
        INSERT INTO leads (
            user_id, company_name, company_domain, company_linkedin,
            company_size, industry, location,
            contact_name, contact_title, contact_email, contact_phone, contact_linkedin,
            status, priority, estimated_value, source, tags, notes, next_followup_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        RETURNING *
        "#,
        auth.id,
        req.company_name,
        req.company_domain,
        req.company_linkedin,
        req.company_size,
        req.industry,
        req.location,
        req.contact_name,
        req.contact_title,
        req.contact_email,
        req.contact_phone,
        req.contact_linkedin,
        req.status.as_deref().unwrap_or("new"),
        req.priority.unwrap_or(0),
        estimated_value,
        req.source,
        req.tags.as_deref(),
        req.notes,
        req.next_followup_at
    )
    .fetch_one(state.db.pool())
    .await?;

    // Create sync event
    sqlx::query!(
        r#"
        INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
        VALUES ($1, 'lead', $2, 'created', $3, 1)
        "#,
        auth.id,
        lead.id,
        serde_json::to_value(&lead).unwrap_or_default()
    )
    .execute(state.db.pool())
    .await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id)
        VALUES ($1, 'lead_created', 'lead', $2)
        "#,
        auth.id,
        lead.id
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(lead))
}

/// Get a single lead
async fn get_lead(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Lead>> {
    let lead = sqlx::query_as!(
        Lead,
        r#"SELECT * FROM leads WHERE id = $1 AND user_id = $2"#,
        id,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::NotFound("Lead not found".to_string()))?;

    Ok(Json(lead))
}

/// Update a lead
async fn update_lead(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLeadRequest>,
) -> ApiResult<Json<Lead>> {
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check ownership
    let existing = sqlx::query!(
        r#"SELECT id FROM leads WHERE id = $1 AND user_id = $2"#,
        id,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::NotFound("Lead not found".to_string()))?;

    let estimated_value = req.estimated_value.map(|v| {
        sqlx::types::BigDecimal::try_from(v).unwrap_or_default()
    });

    let lead = sqlx::query_as!(
        Lead,
        r#"
        UPDATE leads SET
            company_name = COALESCE($3, company_name),
            company_domain = COALESCE($4, company_domain),
            company_linkedin = COALESCE($5, company_linkedin),
            company_size = COALESCE($6, company_size),
            industry = COALESCE($7, industry),
            location = COALESCE($8, location),
            contact_name = COALESCE($9, contact_name),
            contact_title = COALESCE($10, contact_title),
            contact_email = COALESCE($11, contact_email),
            contact_phone = COALESCE($12, contact_phone),
            contact_linkedin = COALESCE($13, contact_linkedin),
            status = COALESCE($14, status),
            priority = COALESCE($15, priority),
            estimated_value = COALESCE($16, estimated_value),
            tags = COALESCE($17, tags),
            notes = COALESCE($18, notes),
            last_contacted_at = COALESCE($19, last_contacted_at),
            next_followup_at = COALESCE($20, next_followup_at),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING *
        "#,
        id,
        auth.id,
        req.company_name,
        req.company_domain,
        req.company_linkedin,
        req.company_size,
        req.industry,
        req.location,
        req.contact_name,
        req.contact_title,
        req.contact_email,
        req.contact_phone,
        req.contact_linkedin,
        req.status,
        req.priority,
        estimated_value,
        req.tags.as_deref(),
        req.notes,
        req.last_contacted_at,
        req.next_followup_at
    )
    .fetch_one(state.db.pool())
    .await?;

    // Get current version
    let version: i64 = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(MAX(version), 0) + 1
        FROM sync_events
        WHERE entity_type = 'lead' AND entity_id = $1
        "#,
        id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(1);

    // Create sync event
    sqlx::query!(
        r#"
        INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
        VALUES ($1, 'lead', $2, 'updated', $3, $4)
        "#,
        auth.id,
        lead.id,
        serde_json::to_value(&lead).unwrap_or_default(),
        version
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(lead))
}

/// Delete a lead
async fn delete_lead(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let result = sqlx::query!(
        r#"DELETE FROM leads WHERE id = $1 AND user_id = $2"#,
        id,
        auth.id
    )
    .execute(state.db.pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Lead not found".to_string()));
    }

    // Create sync event for deletion
    sqlx::query!(
        r#"
        INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
        VALUES ($1, 'lead', $2, 'deleted', '{}',
            COALESCE((SELECT MAX(version) FROM sync_events WHERE entity_type = 'lead' AND entity_id = $2), 0) + 1)
        "#,
        auth.id,
        id
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
