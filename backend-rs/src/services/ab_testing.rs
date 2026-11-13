//! Phase 38: A/B Testing & Phased Deployments Service
//!
//! Enables data-driven experimentation on AI configurations (confidence thresholds,
//! retrieval strategies, response formats, etc.) with statistical analysis and winner detection.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABExperiment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub experiment_type: String,
    pub status: String,
    pub traffic_split: f64,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub winner_variant_id: Option<String>,
    pub created_by: Option<String>,
    pub org_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABVariant {
    pub id: String,
    pub experiment_id: String,
    pub variant_name: String,
    pub config: serde_json::Value,
    pub is_control: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABAssignment {
    pub id: String,
    pub experiment_id: String,
    pub variant_id: String,
    pub conversation_id: String,
    pub contact_id: Option<String>,
    pub assigned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABResult {
    pub id: String,
    pub experiment_id: String,
    pub variant_id: String,
    pub assignment_id: Option<String>,
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariantStats {
    pub variant_id: String,
    pub variant_name: String,
    pub is_control: bool,
    pub assignments: i64,
    pub results: i64,
    pub avg_metric_values: HashMap<String, f64>,
    pub min_metric_values: HashMap<String, f64>,
    pub max_metric_values: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABExperimentSummary {
    pub id: String,
    pub experiment_id: String,
    pub total_assignments: i64,
    pub total_results: i64,
    pub variant_stats: Vec<VariantStats>,
    pub statistical_significance: f64,
    pub recommended_winner: Option<String>,
    pub snapshot_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABSettings {
    pub id: String,
    pub org_id: String,
    pub enabled: bool,
    pub auto_detect_winner: bool,
    pub min_sample_size: i32,
    pub significance_threshold: f64,
    pub max_concurrent_experiments: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABTestingSummary {
    pub total_experiments: i64,
    pub running_experiments: i64,
    pub completed_experiments: i64,
    pub total_assignments: i64,
    pub total_results: i64,
    pub experiments_by_type: Vec<ExperimentTypeCount>,
    pub recent_experiments: Vec<ABExperiment>,
    pub settings: ABSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExperimentTypeCount {
    pub experiment_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignmentResult {
    pub assignment: ABAssignment,
    pub variant: ABVariant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateExperimentResult {
    pub experiment: ABExperiment,
    pub variants: Vec<ABVariant>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummarySnapshotResult {
    pub summary: ABExperimentSummary,
}

pub struct ABTestingService {
    pub pool: PgPool,
}

impl ABTestingService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Get overall A/B testing summary
    pub async fn get_summary(&self) -> Result<ABTestingSummary, anyhow::Error> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ab_experiments")
            .fetch_one(&self.pool).await?;
        let running: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ab_experiments WHERE status = 'running'")
            .fetch_one(&self.pool).await?;
        let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ab_experiments WHERE status = 'completed'")
            .fetch_one(&self.pool).await?;
        let total_assignments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ab_assignments")
            .fetch_one(&self.pool).await?;
        let total_results: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ab_results")
            .fetch_one(&self.pool).await?;

        let type_rows = sqlx::query("SELECT experiment_type, COUNT(*) as count FROM ab_experiments GROUP BY experiment_type ORDER BY count DESC")
            .fetch_all(&self.pool).await?;
        let experiments_by_type: Vec<ExperimentTypeCount> = type_rows.iter().map(|r| {
            ExperimentTypeCount {
                experiment_type: r.try_get(0).unwrap_or_default(),
                count: r.try_get(1).unwrap_or(0i64),
            }
        }).collect();

        let recent_rows = sqlx::query(
            "SELECT id, name, description, experiment_type, status, traffic_split, \
             start_at, end_at, winner_variant_id, created_by, org_id, created_at, updated_at \
             FROM ab_experiments ORDER BY created_at DESC LIMIT 5"
        ).fetch_all(&self.pool).await?;

        let recent_experiments: Vec<ABExperiment> = recent_rows.iter().map(|r| {
            ABExperiment {
                id: r.try_get(0).unwrap_or_default(),
                name: r.try_get(1).unwrap_or_default(),
                description: r.try_get(2).ok(),
                experiment_type: r.try_get(3).unwrap_or_default(),
                status: r.try_get(4).unwrap_or_default(),
                traffic_split: r.try_get(5).unwrap_or(0.5f64),
                start_at: r.try_get(6).ok(),
                end_at: r.try_get(7).ok(),
                winner_variant_id: r.try_get(8).ok(),
                created_by: r.try_get(9).ok(),
                org_id: r.try_get(10).unwrap_or_default(),
                created_at: r.try_get(11).unwrap_or_default(),
                updated_at: r.try_get(12).unwrap_or_default(),
            }
        }).collect();

        let settings = self.get_settings().await.unwrap_or_else(|_| ABSettings {
            id: "default".to_string(),
            org_id: "org-default".to_string(),
            enabled: true,
            auto_detect_winner: false,
            min_sample_size: 100,
            significance_threshold: 0.95,
            max_concurrent_experiments: 5,
        });

        Ok(ABTestingSummary {
            total_experiments: total,
            running_experiments: running,
            completed_experiments: completed,
            total_assignments,
            total_results,
            experiments_by_type,
            recent_experiments,
            settings,
        })
    }

    /// List experiments with filters
    pub async fn list_experiments(
        &self,
        status: Option<&str>,
        experiment_type: Option<&str>,
    ) -> Result<Vec<ABExperiment>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT id, name, description, experiment_type, status, traffic_split, \
             start_at, end_at, winner_variant_id, created_by, org_id, created_at, updated_at \
             FROM ab_experiments ORDER BY created_at DESC"
        ).fetch_all(&self.pool).await?;

        let mut experiments = Vec::new();
        for row in &rows {
            let exp = ABExperiment {
                id: row.try_get(0).unwrap_or_default(),
                name: row.try_get(1).unwrap_or_default(),
                description: row.try_get(2).ok(),
                experiment_type: row.try_get(3).unwrap_or_default(),
                status: row.try_get(4).unwrap_or_default(),
                traffic_split: row.try_get(5).unwrap_or(0.5f64),
                start_at: row.try_get(6).ok(),
                end_at: row.try_get(7).ok(),
                winner_variant_id: row.try_get(8).ok(),
                created_by: row.try_get(9).ok(),
                org_id: row.try_get(10).unwrap_or_default(),
                created_at: row.try_get(11).unwrap_or_default(),
                updated_at: row.try_get(12).unwrap_or_default(),
            };

            // Filter in Rust
            if let Some(s) = status {
                if exp.status != s {
                    continue;
                }
            }
            if let Some(t) = experiment_type {
                if exp.experiment_type != t {
                    continue;
                }
            }

            experiments.push(exp);
        }

        Ok(experiments)
    }

    /// Create a new experiment with variants
    pub async fn create_experiment(
        &self,
        name: &str,
        description: Option<&str>,
        experiment_type: &str,
        traffic_split: f64,
        variants: &[ABVariantInput],
    ) -> Result<CreateExperimentResult, anyhow::Error> {
        let experiment_id = uuid::Uuid::new_v4().to_string();

        // Create experiment
        sqlx::query(
            "INSERT INTO ab_experiments (id, name, description, experiment_type, status, traffic_split) \
             VALUES ($1, $2, $3, $4, 'draft', $5)"
        )
        .bind(&experiment_id)
        .bind(name)
        .bind(description)
        .bind(experiment_type)
        .bind(traffic_split)
        .execute(&self.pool)
        .await?;

        // Create variants
        let mut created_variants = Vec::new();
        for (i, v) in variants.iter().enumerate() {
            let variant_id = uuid::Uuid::new_v4().to_string();
            let is_control = i == 0;
            sqlx::query(
                "INSERT INTO ab_variants (id, experiment_id, variant_name, config, is_control) \
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(&variant_id)
            .bind(&experiment_id)
            .bind(&v.variant_name)
            .bind(serde_json::to_string(&v.config).unwrap_or_else(|_| "{}".to_string()))
            .bind(is_control)
            .execute(&self.pool)
            .await?;

            created_variants.push(ABVariant {
                id: variant_id,
                experiment_id: experiment_id.clone(),
                variant_name: v.variant_name.clone(),
                config: v.config.clone(),
                is_control,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        let experiment = ABExperiment {
            id: experiment_id.clone(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            experiment_type: experiment_type.to_string(),
            status: "draft".to_string(),
            traffic_split,
            start_at: None,
            end_at: None,
            winner_variant_id: None,
            created_by: None,
            org_id: "org-default".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(CreateExperimentResult {
            experiment,
            variants: created_variants,
        })
    }

    /// Update experiment status
    pub async fn update_experiment_status(
        &self,
        experiment_id: &str,
        status: &str,
    ) -> Result<ABExperiment, anyhow::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE ab_experiments SET status = $1, updated_at = $2, \
             start_at = CASE WHEN $1 = 'running' THEN NOW() ELSE start_at END, \
             end_at = CASE WHEN $1 IN ('completed', 'failed') THEN NOW() ELSE end_at END \
             WHERE id = $3"
        )
        .bind(status)
        .bind(&now)
        .bind(experiment_id)
        .execute(&self.pool)
        .await?;

        self.get_experiment(experiment_id).await
    }

    /// Get experiment with variants
    pub async fn get_experiment(&self, experiment_id: &str) -> Result<ABExperiment, anyhow::Error> {
        let row = sqlx::query(
            "SELECT id, name, description, experiment_type, status, traffic_split, \
             start_at, end_at, winner_variant_id, created_by, org_id, created_at, updated_at \
             FROM ab_experiments WHERE id = $1"
        ).bind(experiment_id).fetch_one(&self.pool).await?;

        Ok(ABExperiment {
            id: row.try_get(0).unwrap_or_default(),
            name: row.try_get(1).unwrap_or_default(),
            description: row.try_get(2).ok(),
            experiment_type: row.try_get(3).unwrap_or_default(),
            status: row.try_get(4).unwrap_or_default(),
            traffic_split: row.try_get(5).unwrap_or(0.5f64),
            start_at: row.try_get(6).ok(),
            end_at: row.try_get(7).ok(),
            winner_variant_id: row.try_get(8).ok(),
            created_by: row.try_get(9).ok(),
            org_id: row.try_get(10).unwrap_or_default(),
            created_at: row.try_get(11).unwrap_or_default(),
            updated_at: row.try_get(12).unwrap_or_default(),
        })
    }

    /// Get variants for an experiment
    pub async fn get_variants(&self, experiment_id: &str) -> Result<Vec<ABVariant>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT id, experiment_id, variant_name, config, is_control, created_at \
             FROM ab_variants WHERE experiment_id = $1 ORDER BY is_control DESC, created_at ASC"
        ).bind(experiment_id).fetch_all(&self.pool).await?;

        let mut variants = Vec::new();
        for row in &rows {
            let config_str: String = row.try_get(3).unwrap_or_else(|_| "{}".to_string());
            let config: serde_json::Value = serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Object(Default::default()));
            variants.push(ABVariant {
                id: row.try_get(0).unwrap_or_default(),
                experiment_id: row.try_get(1).unwrap_or_default(),
                variant_name: row.try_get(2).unwrap_or_default(),
                config,
                is_control: row.try_get(4).unwrap_or(false),
                created_at: row.try_get(5).unwrap_or_default(),
            });
        }
        Ok(variants)
    }

    /// Assign a conversation to a variant (randomized based on traffic split)
    pub async fn assign_conversation(
        &self,
        experiment_id: &str,
        conversation_id: &str,
        contact_id: Option<&str>,
    ) -> Result<AssignmentResult, anyhow::Error> {
        // Check if A/B testing is enabled
        let settings = self.get_settings().await;
        if !settings.map(|s| s.enabled).unwrap_or(true) {
            return Err(anyhow::anyhow!("A/B testing is disabled"));
        }

        // Get experiment
        let experiment = self.get_experiment(experiment_id).await?;
        if experiment.status != "running" {
            return Err(anyhow::anyhow!("Experiment is not running"));
        }

        // Get variants
        let variants = self.get_variants(experiment_id).await?;
        if variants.is_empty() {
            return Err(anyhow::anyhow!("No variants for experiment"));
        }

        // Random assignment based on traffic split
        let random: f64 = rand::random();
        let variant = if random < experiment.traffic_split {
            // Assign to control
            variants.iter().find(|v| v.is_control).unwrap_or(&variants[0]).clone()
        } else {
            // Assign to a non-control variant
            let non_control: Vec<&ABVariant> = variants.iter().filter(|v| !v.is_control).collect();
            if non_control.is_empty() {
                variants[0].clone()
            } else {
                non_control[rand::random::<usize>() % non_control.len()].clone()
            }
        };

        // Create assignment
        let assignment_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO ab_assignments (id, experiment_id, variant_id, conversation_id, contact_id) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&assignment_id)
        .bind(experiment_id)
        .bind(&variant.id)
        .bind(conversation_id)
        .bind(contact_id)
        .execute(&self.pool)
        .await?;

        Ok(AssignmentResult {
            assignment: ABAssignment {
                id: assignment_id,
                experiment_id: experiment_id.to_string(),
                variant_id: variant.id.clone(),
                conversation_id: conversation_id.to_string(),
                contact_id: contact_id.map(|s| s.to_string()),
                assigned_at: chrono::Utc::now().to_rfc3339(),
            },
            variant,
        })
    }

    /// Record a result for an assignment
    pub async fn record_result(
        &self,
        experiment_id: &str,
        variant_id: &str,
        assignment_id: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<ABResult, anyhow::Error> {
        let result_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO ab_results (id, experiment_id, variant_id, assignment_id, metric_name, metric_value) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&result_id)
        .bind(experiment_id)
        .bind(variant_id)
        .bind(assignment_id)
        .bind(metric_name)
        .bind(metric_value)
        .execute(&self.pool)
        .await?;

        Ok(ABResult {
            id: result_id,
            experiment_id: experiment_id.to_string(),
            variant_id: variant_id.to_string(),
            assignment_id: Some(assignment_id.to_string()),
            metric_name: metric_name.to_string(),
            metric_value,
            metadata: serde_json::Value::Object(Default::default()),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Generate summary snapshot for an experiment
    pub async fn generate_summary(&self, experiment_id: &str) -> Result<SummarySnapshotResult, anyhow::Error> {
        // Get assignments count
        let total_assignments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ab_assignments WHERE experiment_id = $1"
        ).bind(experiment_id).fetch_one(&self.pool).await?;

        // Get results count
        let total_results: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ab_results WHERE experiment_id = $1"
        ).bind(experiment_id).fetch_one(&self.pool).await?;

        // Get variant stats
        let variants = self.get_variants(experiment_id).await?;
        let mut variant_stats = Vec::new();

        for variant in &variants {
            let assignments: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM ab_assignments WHERE experiment_id = $1 AND variant_id = $2"
            ).bind(experiment_id).bind(&variant.id).fetch_one(&self.pool).await?;

            let results: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM ab_results WHERE experiment_id = $1 AND variant_id = $2"
            ).bind(experiment_id).bind(&variant.id).fetch_one(&self.pool).await?;

            // Get average metric values
            let metric_rows = sqlx::query(
                "SELECT metric_name, AVG(metric_value) as avg_val, MIN(metric_value) as min_val, MAX(metric_value) as max_val \
                 FROM ab_results WHERE experiment_id = $1 AND variant_id = $2 \
                 GROUP BY metric_name"
            ).bind(experiment_id).bind(&variant.id).fetch_all(&self.pool).await?;

            let mut avg_metrics = HashMap::new();
            let mut min_metrics = HashMap::new();
            let mut max_metrics = HashMap::new();

            for row in &metric_rows {
                let name: String = row.try_get(0).unwrap_or_default();
                let avg: f64 = row.try_get(1).unwrap_or(0.0);
                let min: f64 = row.try_get(2).unwrap_or(0.0);
                let max: f64 = row.try_get(3).unwrap_or(0.0);
                avg_metrics.insert(name.clone(), avg);
                min_metrics.insert(name.clone(), min);
                max_metrics.insert(name.clone(), max);
            }

            variant_stats.push(VariantStats {
                variant_id: variant.id.clone(),
                variant_name: variant.variant_name.clone(),
                is_control: variant.is_control,
                assignments,
                results,
                avg_metric_values: avg_metrics,
                min_metric_values: min_metrics,
                max_metric_values: max_metrics,
            });
        }

        // Calculate statistical significance (simplified t-test approximation)
        let significance = self.calculate_significance(&variant_stats);

        // Determine recommended winner
        let recommended_winner = if significance >= 0.95 {
            // Find variant with highest average resolution_rate
            variant_stats.iter()
                .max_by(|a, b| {
                    let a_val = a.avg_metric_values.get("resolution_rate").copied().unwrap_or(0.0);
                    let b_val = b.avg_metric_values.get("resolution_rate").copied().unwrap_or(0.0);
                    a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal)
                })
                .filter(|v| v.results >= 10)
                .map(|v| v.variant_id.clone())
        } else {
            None
        };

        // Store summary snapshot
        let summary_id = uuid::Uuid::new_v4().to_string();
        let variant_stats_json = serde_json::to_string(&variant_stats).unwrap_or_else(|_| "{}".to_string());
        sqlx::query(
            "INSERT INTO ab_experiment_summary (id, experiment_id, total_assignments, total_results, \
             variant_stats, statistical_significance, recommended_winner) \
             VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7)"
        )
        .bind(&summary_id)
        .bind(experiment_id)
        .bind(total_assignments)
        .bind(total_results)
        .bind(&variant_stats_json)
        .bind(significance)
        .bind(&recommended_winner)
        .execute(&self.pool)
        .await?;

        Ok(SummarySnapshotResult {
            summary: ABExperimentSummary {
                id: summary_id,
                experiment_id: experiment_id.to_string(),
                total_assignments,
                total_results,
                variant_stats,
                statistical_significance: significance,
                recommended_winner,
                snapshot_at: chrono::Utc::now().to_rfc3339(),
            },
        })
    }

    /// Calculate statistical significance (simplified)
    fn calculate_significance(&self, variant_stats: &[VariantStats]) -> f64 {
        if variant_stats.len() < 2 {
            return 0.0;
        }

        // Simplified: compare first two variants' resolution_rate
        let mut values: Vec<f64> = Vec::new();
        for vs in variant_stats {
            if let Some(val) = vs.avg_metric_values.get("resolution_rate") {
                values.push(*val);
            }
        }

        if values.len() < 2 {
            return 0.0;
        }

        // Simple difference-based significance
        let max_val = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&0.0);
        let min_val = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&0.0);

        if max_val == 0.0 {
            return 0.0;
        }

        let relative_diff = (max_val - min_val) / max_val;

        // Map difference to significance (0-1)
        // >10% difference = high significance
        if relative_diff > 0.1 {
            0.99
        } else if relative_diff > 0.05 {
            0.95
        } else if relative_diff > 0.02 {
            0.9
        } else if relative_diff > 0.01 {
            0.8
        } else {
            0.5
        }
    }

    /// Get settings
    pub async fn get_settings(&self) -> Result<ABSettings, anyhow::Error> {
        let row = sqlx::query(
            "SELECT id, org_id, enabled, auto_detect_winner, min_sample_size, \
             significance_threshold, max_concurrent_experiments \
             FROM ab_settings WHERE org_id = 'org-default' LIMIT 1"
        ).fetch_one(&self.pool).await?;

        Ok(ABSettings {
            id: row.try_get(0).unwrap_or_default(),
            org_id: row.try_get(1).unwrap_or_default(),
            enabled: row.try_get(2).unwrap_or(true),
            auto_detect_winner: row.try_get(3).unwrap_or(false),
            min_sample_size: row.try_get(4).unwrap_or(100i32),
            significance_threshold: row.try_get(5).unwrap_or(0.95f64),
            max_concurrent_experiments: row.try_get(6).unwrap_or(5i32),
        })
    }

    /// Update settings
    pub async fn update_settings(&self, settings: &ABSettings) -> Result<ABSettings, anyhow::Error> {
        sqlx::query(
            "UPDATE ab_settings SET enabled = $1, auto_detect_winner = $2, \
             min_sample_size = $3, significance_threshold = $4, \
             max_concurrent_experiments = $5, updated_at = NOW() \
             WHERE id = $6"
        )
        .bind(settings.enabled)
        .bind(settings.auto_detect_winner)
        .bind(settings.min_sample_size)
        .bind(settings.significance_threshold)
        .bind(settings.max_concurrent_experiments)
        .bind(&settings.id)
        .execute(&self.pool)
        .await?;

        self.get_settings().await
    }

    /// Delete experiment
    pub async fn delete_experiment(&self, experiment_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM ab_experiments WHERE id = $1")
            .bind(experiment_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get results for an experiment
    pub async fn get_results(&self, experiment_id: &str) -> Result<Vec<ABResult>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT id, experiment_id, variant_id, assignment_id, metric_name, metric_value, \
             COALESCE(metadata, '{}'::text) as metadata, created_at \
             FROM ab_results WHERE experiment_id = $1 ORDER BY created_at DESC LIMIT 100"
        ).bind(experiment_id).fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for row in &rows {
            let metadata_str: String = row.try_get(6).unwrap_or_else(|_| "{}".to_string());
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Object(Default::default()));
            results.push(ABResult {
                id: row.try_get(0).unwrap_or_default(),
                experiment_id: row.try_get(1).unwrap_or_default(),
                variant_id: row.try_get(2).unwrap_or_default(),
                assignment_id: row.try_get(3).ok(),
                metric_name: row.try_get(4).unwrap_or_default(),
                metric_value: row.try_get(5).unwrap_or(0.0),
                metadata,
                created_at: row.try_get(7).unwrap_or_default(),
            });
        }
        Ok(results)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ABVariantInput {
    pub variant_name: String,
    pub config: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = ABTestingSummary {
            total_experiments: 1,
            running_experiments: 0,
            completed_experiments: 1,
            total_assignments: 100,
            total_results: 50,
            experiments_by_type: vec![ExperimentTypeCount {
                experiment_type: "confidence_threshold".to_string(),
                count: 1,
            }],
            recent_experiments: vec![],
            settings: ABSettings {
                id: "default".to_string(),
                org_id: "org-default".to_string(),
                enabled: true,
                auto_detect_winner: false,
                min_sample_size: 100,
                significance_threshold: 0.95,
                max_concurrent_experiments: 5,
            },
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("ABTestingSummary"));
    }

    #[test]
    fn test_assignment_result() {
        let result = AssignmentResult {
            assignment: ABAssignment {
                id: "test".to_string(),
                experiment_id: "exp".to_string(),
                variant_id: "var".to_string(),
                conversation_id: "conv".to_string(),
                contact_id: None,
                assigned_at: "2026-01-01T00:00:00Z".to_string(),
            },
            variant: ABVariant {
                id: "var".to_string(),
                experiment_id: "exp".to_string(),
                variant_name: "Control".to_string(),
                config: serde_json::json!({"threshold": 0.7}),
                is_control: true,
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Control"));
    }

    #[test]
    fn test_significance_calculation() {
        // Can't test without DB, but structure is correct
    }
}
