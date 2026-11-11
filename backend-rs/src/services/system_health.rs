use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnboardingStep {
    pub id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub route: String,
    pub check_endpoint: Option<String>,
    pub display_order: i32,
    pub is_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnboardingProgress {
    pub id: String,
    pub organization_id: String,
    pub step_id: String,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub completed_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheckResult {
    pub id: String,
    pub organization_id: String,
    pub component: String,
    pub status: String,
    pub message: Option<String>,
    pub details: Option<String>,
    pub last_checked_at: String,
    pub check_duration_ms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnboardingSettings {
    pub id: String,
    pub organization_id: String,
    pub show_onboarding: bool,
    pub auto_check: bool,
    pub health_check_interval_minutes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingSummary {
    pub total_steps: i32,
    pub completed_steps: i32,
    pub required_steps: i32,
    pub required_completed: i32,
    pub completion_percentage: f64,
    pub required_completion_percentage: f64,
    pub categories: Vec<CategoryProgress>,
    pub next_steps: Vec<OnboardingStep>,
    pub settings: OnboardingSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryProgress {
    pub category: String,
    pub total: i32,
    pub completed: i32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckSummary {
    pub total_components: i32,
    pub healthy: i32,
    pub degraded: i32,
    pub unhealthy: i32,
    pub unknown: i32,
    pub overall_status: String,
    pub results: Vec<HealthCheckResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteStepResult {
    pub success: bool,
    pub progress: OnboardingProgress,
    pub summary: OnboardingSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunHealthCheckResult {
    pub success: bool,
    pub summary: HealthCheckSummary,
}

pub struct SystemHealthService {
    pub pool: PgPool,
}

impl SystemHealthService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get_summary(&self, org_id: &str) -> Result<OnboardingSummary, String> {
        // Get settings
        let settings = sqlx::query(
            "SELECT id, organization_id, show_onboarding, auto_check, health_check_interval_minutes \
             FROM onboarding_settings WHERE organization_id = $1 LIMIT 1"
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch settings: {}", e))?
        .map(|row| OnboardingSettings {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            show_onboarding: row.try_get(2).unwrap_or(true),
            auto_check: row.try_get(3).unwrap_or(true),
            health_check_interval_minutes: row.try_get(4).unwrap_or(60),
        })
        .unwrap_or(OnboardingSettings {
            id: format!("onboard-settings-{}", org_id),
            organization_id: org_id.to_string(),
            show_onboarding: true,
            auto_check: true,
            health_check_interval_minutes: 60,
        });

        // Get all steps
        let steps_rows = sqlx::query(
            "SELECT id, category, title, description, route, check_endpoint, display_order, is_required \
             FROM onboarding_steps ORDER BY display_order"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch steps: {}", e))?;

        let steps: Vec<OnboardingStep> = steps_rows
            .iter()
            .map(|row| OnboardingStep {
                id: row.try_get(0).unwrap_or_default(),
                category: row.try_get(1).unwrap_or_default(),
                title: row.try_get(2).unwrap_or_default(),
                description: row.try_get(3).unwrap_or_default(),
                route: row.try_get(4).unwrap_or_default(),
                check_endpoint: row.try_get(5).ok().flatten(),
                display_order: row.try_get(6).unwrap_or(0),
                is_required: row.try_get(7).unwrap_or(false),
            })
            .collect();

        let total_steps = steps.len() as i32;
        let required_steps = steps.iter().filter(|s| s.is_required).count() as i32;

        // Get progress for this org
        let progress_rows = sqlx::query(
            "SELECT step_id, is_completed, completed_at, completed_by, notes \
             FROM onboarding_progress WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch progress: {}", e))?;

        let completed_steps: std::collections::HashSet<String> = progress_rows
            .iter()
            .filter(|row| row.try_get::<bool, _>(1).unwrap_or(false))
            .filter_map(|row| row.try_get::<String, _>(0).ok())
            .collect();

        let completed_count = completed_steps.len() as i32;
        let required_completed = steps
            .iter()
            .filter(|s| s.is_required && completed_steps.contains(&s.id))
            .count() as i32;

        // Category progress
        let mut categories: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();
        for step in &steps {
            let entry = categories.entry(step.category.clone()).or_insert((0, 0));
            entry.0 += 1;
            if completed_steps.contains(&step.id) {
                entry.1 += 1;
            }
        }

        let category_progress: Vec<CategoryProgress> = categories
            .into_iter()
            .map(|(category, (total, completed))| CategoryProgress {
                category,
                total,
                completed,
                percentage: if total > 0 { (completed as f64 / total as f64) * 100.0 } else { 0.0 },
            })
            .collect();

        // Next steps (first 3 incomplete)
        let next_steps: Vec<OnboardingStep> = steps
            .into_iter()
            .filter(|s| !completed_steps.contains(&s.id))
            .take(3)
            .collect();

        Ok(OnboardingSummary {
            total_steps,
            completed_steps: completed_count,
            required_steps,
            required_completed,
            completion_percentage: if total_steps > 0 { (completed_count as f64 / total_steps as f64) * 100.0 } else { 0.0 },
            required_completion_percentage: if required_steps > 0 { (required_completed as f64 / required_steps as f64) * 100.0 } else { 0.0 },
            categories: category_progress,
            next_steps,
            settings,
        })
    }

    pub async fn get_steps(&self) -> Result<Vec<OnboardingStep>, String> {
        let rows = sqlx::query(
            "SELECT id, category, title, description, route, check_endpoint, display_order, is_required \
             FROM onboarding_steps ORDER BY display_order"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch steps: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| OnboardingStep {
                id: row.try_get(0).unwrap_or_default(),
                category: row.try_get(1).unwrap_or_default(),
                title: row.try_get(2).unwrap_or_default(),
                description: row.try_get(3).unwrap_or_default(),
                route: row.try_get(4).unwrap_or_default(),
                check_endpoint: row.try_get(5).ok().flatten(),
                display_order: row.try_get(6).unwrap_or(0),
                is_required: row.try_get(7).unwrap_or(false),
            })
            .collect())
    }

    pub async fn complete_step(
        &self,
        org_id: &str,
        step_id: &str,
        completed_by: Option<&str>,
    ) -> Result<CompleteStepResult, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Upsert progress
        sqlx::query(
            "INSERT INTO onboarding_progress (id, organization_id, step_id, is_completed, completed_at, completed_by, notes, updated_at) \
             VALUES ($1, $2, $3, true, $4, $5, NULL, $4) \
             ON CONFLICT (organization_id, step_id) \
             DO UPDATE SET is_completed = true, completed_at = $4, completed_by = $5, updated_at = $4"
        )
        .bind(&id)
        .bind(org_id)
        .bind(step_id)
        .bind(&now)
        .bind(completed_by.map(String::from))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update progress: {}", e))?;

        let progress = OnboardingProgress {
            id: id.clone(),
            organization_id: org_id.to_string(),
            step_id: step_id.to_string(),
            is_completed: true,
            completed_at: Some(now),
            completed_by: completed_by.map(String::from),
            notes: None,
        };

        let summary = self.get_summary(org_id).await?;

        Ok(CompleteStepResult {
            success: true,
            progress,
            summary,
        })
    }

    pub async fn get_progress(&self, org_id: &str) -> Result<Vec<OnboardingProgress>, String> {
        let rows = sqlx::query(
            "SELECT op.id, op.organization_id, op.step_id, op.is_completed, \
                    op.completed_at, op.completed_by, op.notes \
             FROM onboarding_progress op \
             JOIN onboarding_steps os ON op.step_id = os.id \
             WHERE op.organization_id = $1 \
             ORDER BY os.display_order"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch progress: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| OnboardingProgress {
                id: row.try_get(0).unwrap_or_default(),
                organization_id: row.try_get(1).unwrap_or_default(),
                step_id: row.try_get(2).unwrap_or_default(),
                is_completed: row.try_get(3).unwrap_or(false),
                completed_at: row.try_get(4).ok().flatten(),
                completed_by: row.try_get(5).ok().flatten(),
                notes: row.try_get(6).ok().flatten(),
            })
            .collect())
    }

    pub async fn run_health_check(&self, org_id: &str) -> Result<RunHealthCheckResult, String> {
        let mut results = Vec::new();

        // Check database
        let db_start = std::time::Instant::now();
        let db_status = if sqlx::query("SELECT 1")
            .fetch_optional(&self.pool)
            .await
            .is_ok()
        {
            "healthy"
        } else {
            "unhealthy"
        };
        let db_duration = db_start.elapsed().as_millis() as i32;

        results.push(HealthCheckResult {
            id: Uuid::new_v4().to_string(),
            organization_id: org_id.to_string(),
            component: "database".to_string(),
            status: db_status.to_string(),
            message: if db_status == "healthy" {
                Some("Database connection is healthy".to_string())
            } else {
                Some("Database connection failed".to_string())
            },
            details: None,
            last_checked_at: chrono::Utc::now().to_rfc3339(),
            check_duration_ms: Some(db_duration),
        });

        // Check Claude API
        let claude_start = std::time::Instant::now();
        let claude_status = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            "healthy"
        } else {
            "unhealthy"
        };
        let claude_duration = claude_start.elapsed().as_millis() as i32;

        results.push(HealthCheckResult {
            id: Uuid::new_v4().to_string(),
            organization_id: org_id.to_string(),
            component: "claude_api".to_string(),
            status: claude_status.to_string(),
            message: if claude_status == "healthy" {
                Some("Claude API key is configured".to_string())
            } else {
                Some("Claude API key is not configured".to_string())
            },
            details: None,
            last_checked_at: chrono::Utc::now().to_rfc3339(),
            check_duration_ms: Some(claude_duration),
        });

        // Check inboxes
        let inbox_rows = sqlx::query(
            "SELECT channel_type, is_active FROM inboxes WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for row in &inbox_rows {
            let channel: String = row.try_get(0).unwrap_or_default();
            let is_active: bool = row.try_get(1).unwrap_or(true);
            results.push(HealthCheckResult {
                id: Uuid::new_v4().to_string(),
                organization_id: org_id.to_string(),
                component: format!("inbox_{}", channel),
                status: if is_active { "healthy" } else { "degraded" }.to_string(),
                message: Some(format!("{} inbox is {}", channel, if is_active { "active" } else { "inactive" })),
                details: None,
                last_checked_at: chrono::Utc::now().to_rfc3339(),
                check_duration_ms: None,
            });
        }

        // Store results
        for result in &results {
            let _ = sqlx::query(
                "INSERT INTO health_check_results (id, organization_id, component, status, message, details, last_checked_at, check_duration_ms) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&result.id)
            .bind(org_id)
            .bind(&result.component)
            .bind(&result.status)
            .bind(&result.message)
            .bind(&result.details)
            .bind(&result.last_checked_at)
            .bind(result.check_duration_ms)
            .execute(&self.pool)
            .await;
        }

        let healthy = results.iter().filter(|r| r.status == "healthy").count() as i32;
        let degraded = results.iter().filter(|r| r.status == "degraded").count() as i32;
        let unhealthy = results.iter().filter(|r| r.status == "unhealthy").count() as i32;
        let unknown = results.iter().filter(|r| r.status == "unknown").count() as i32;

        let overall_status = if unhealthy > 0 {
            "unhealthy"
        } else if degraded > 0 {
            "degraded"
        } else if healthy > 0 {
            "healthy"
        } else {
            "unknown"
        }.to_string();

        Ok(RunHealthCheckResult {
            success: true,
            summary: HealthCheckSummary {
                total_components: results.len() as i32,
                healthy,
                degraded,
                unhealthy,
                unknown,
                overall_status,
                results,
            },
        })
    }

    pub async fn get_health_summary(&self, org_id: &str) -> Result<HealthCheckSummary, String> {
        let rows = sqlx::query(
            "SELECT id, organization_id, component, status, message, details, last_checked_at, check_duration_ms \
             FROM health_check_results \
             WHERE organization_id = $1 \
             ORDER BY last_checked_at DESC"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch health results: {}", e))?;

        let results: Vec<HealthCheckResult> = rows
            .iter()
            .map(|row| HealthCheckResult {
                id: row.try_get(0).unwrap_or_default(),
                organization_id: row.try_get(1).unwrap_or_default(),
                component: row.try_get(2).unwrap_or_default(),
                status: row.try_get(3).unwrap_or_default(),
                message: row.try_get(4).ok().flatten(),
                details: row.try_get(5).ok().flatten(),
                last_checked_at: row.try_get(6).unwrap_or_default(),
                check_duration_ms: row.try_get(7).ok().flatten(),
            })
            .collect();

        let healthy = results.iter().filter(|r| r.status == "healthy").count() as i32;
        let degraded = results.iter().filter(|r| r.status == "degraded").count() as i32;
        let unhealthy = results.iter().filter(|r| r.status == "unhealthy").count() as i32;
        let unknown = results.iter().filter(|r| r.status == "unknown").count() as i32;

        let overall_status = if unhealthy > 0 {
            "unhealthy"
        } else if degraded > 0 {
            "degraded"
        } else if healthy > 0 {
            "healthy"
        } else {
            "unknown"
        }.to_string();

        Ok(HealthCheckSummary {
            total_components: results.len() as i32,
            healthy,
            degraded,
            unhealthy,
            unknown,
            overall_status,
            results,
        })
    }

    pub async fn get_settings(&self, org_id: &str) -> Result<OnboardingSettings, String> {
        let row = sqlx::query(
            "SELECT id, organization_id, show_onboarding, auto_check, health_check_interval_minutes \
             FROM onboarding_settings WHERE organization_id = $1 LIMIT 1"
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch settings: {}", e))?;

        match row {
            Some(r) => Ok(OnboardingSettings {
                id: r.try_get(0).unwrap_or_default(),
                organization_id: r.try_get(1).unwrap_or_default(),
                show_onboarding: r.try_get(2).unwrap_or(true),
                auto_check: r.try_get(3).unwrap_or(true),
                health_check_interval_minutes: r.try_get(4).unwrap_or(60),
            }),
            None => Ok(OnboardingSettings {
                id: format!("onboard-settings-{}", org_id),
                organization_id: org_id.to_string(),
                show_onboarding: true,
                auto_check: true,
                health_check_interval_minutes: 60,
            }),
        }
    }

    pub async fn update_settings(
        &self,
        org_id: &str,
        show_onboarding: Option<bool>,
        auto_check: Option<bool>,
        health_check_interval: Option<i32>,
    ) -> Result<OnboardingSettings, String> {
        let mut updates = Vec::new();
        if let Some(v) = show_onboarding {
            updates.push(format!("show_onboarding = {}", if v { 1 } else { 0 }));
        }
        if let Some(v) = auto_check {
            updates.push(format!("auto_check = {}", if v { 1 } else { 0 }));
        }
        if let Some(v) = health_check_interval {
            updates.push(format!("health_check_interval_minutes = {}", v));
        }

        if !updates.is_empty() {
            let set_clause = updates.join(", ");
            let _ = sqlx::query(&format!(
                "INSERT INTO onboarding_settings (id, organization_id, show_onboarding, auto_check, health_check_interval_minutes) \
                 VALUES ('onboard-settings-{}', '{}', COALESCE((SELECT show_onboarding FROM onboarding_settings WHERE organization_id = '{}'), true), \
                 COALESCE((SELECT auto_check FROM onboarding_settings WHERE organization_id = '{}'), true), \
                 COALESCE((SELECT health_check_interval_minutes FROM onboarding_settings WHERE organization_id = '{}'), 60)) \
                 ON CONFLICT (id) DO UPDATE SET {}, updated_at = NOW()",
                org_id, org_id, org_id, org_id, org_id, set_clause
            ))
            .execute(&self.pool)
            .await;
        }

        self.get_settings(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_summary_serialization() {
        let summary = OnboardingSummary {
            total_steps: 16,
            completed_steps: 5,
            required_steps: 4,
            required_completed: 3,
            completion_percentage: 31.25,
            required_completion_percentage: 75.0,
            categories: vec![],
            next_steps: vec![],
            settings: OnboardingSettings {
                id: "test".to_string(),
                organization_id: "org-default".to_string(),
                show_onboarding: true,
                auto_check: true,
                health_check_interval_minutes: 60,
            },
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalSteps\":16"));
        assert!(json.contains("\"completionPercentage\":31.25"));
    }

    #[test]
    fn test_health_check_summary_serialization() {
        let summary = HealthCheckSummary {
            total_components: 3,
            healthy: 2,
            degraded: 1,
            unhealthy: 0,
            unknown: 0,
            overall_status: "degraded".to_string(),
            results: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalComponents\":3"));
        assert!(json.contains("\"overallStatus\":\"degraded\""));
    }

    #[test]
    fn test_complete_step_result_serialization() {
        let result = CompleteStepResult {
            success: true,
            progress: OnboardingProgress {
                id: "test".to_string(),
                organization_id: "org-default".to_string(),
                step_id: "onboard-welcome".to_string(),
                is_completed: true,
                completed_at: Some("2024-01-01T00:00:00Z".to_string()),
                completed_by: Some("user-1".to_string()),
                notes: None,
            },
            summary: OnboardingSummary {
                total_steps: 16,
                completed_steps: 1,
                required_steps: 4,
                required_completed: 1,
                completion_percentage: 6.25,
                required_completion_percentage: 25.0,
                categories: vec![],
                next_steps: vec![],
                settings: OnboardingSettings {
                    id: "test".to_string(),
                    organization_id: "org-default".to_string(),
                    show_onboarding: true,
                    auto_check: true,
                    health_check_interval_minutes: 60,
                },
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"isCompleted\":true"));
    }
}
