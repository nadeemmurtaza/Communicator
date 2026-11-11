use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Models - directly mapped from nID PostgreSQL tables
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidAccount {
    pub id: String,
    pub phone: String,
    pub email: Option<String>,
    pub phone_verified_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidProfile {
    pub id: String,
    pub account_id: Option<String>,
    pub organization_id: Option<String>,
    pub slug: String,
    pub display_name: String,
    pub headline: Option<String>,
    pub category: String,
    pub bio: Option<String>,
    pub photo_key: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub public_email: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub timezone: String,
    pub is_published: bool,
    pub is_personal: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidOrganization {
    pub id: String,
    pub tenant_id: String,
    pub subject_id: String,
    pub legal_name: String,
    pub registration_number: Option<String>,
    pub country_code: Option<String>,
    pub is_verified: bool,
    pub verified_level: Option<String>,
    pub verified_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidMembership {
    pub id: String,
    pub account_id: Option<String>,
    pub organization_id: String,
    pub role: Option<String>,
    pub is_admin: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidInvoice {
    pub id: String,
    pub invoice_number: String,
    pub status: String,
    pub currency: String,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub amount_paid_minor: i64,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub due_at: Option<String>,
    pub issued_at: Option<String>,
    pub paid_at: Option<String>,
    pub tenant_id: String,
    pub tenant_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidPayment {
    pub id: String,
    pub provider: String,
    pub provider_transaction_id: String,
    pub amount_minor: i64,
    pub currency: String,
    pub status: String,
    pub method_type: Option<String>,
    pub paid_at: Option<String>,
    pub tenant_id: String,
    pub tenant_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidSubscription {
    pub id: String,
    pub status: String,
    pub plan_key: String,
    pub plan_name: String,
    pub plan_mode: String,
    pub plan_currency: String,
    pub plan_amount: i64,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub tenant_id: String,
    pub service_key: Option<String>,
    pub service_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NidTenant {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrandSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub total_orders: i64,
    pub total_sales: f64,
    pub total_customers: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalOrder {
    pub id: String,
    pub order_number: String,
    pub status: String,
    pub total_amount: f64,
    pub currency: String,
    pub platform_name: String,
    pub platform_type: String,
    pub brand_name: String,
    pub shop_name: Option<String>,
    pub platform_logo_url: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaunchpadOverview {
    pub personal_profile: Option<NidProfile>,
    pub work_profiles: Vec<NidProfile>,
    pub organizations: Vec<NidOrganization>,
    pub memberships: Vec<NidMembership>,
    pub tenants: Vec<NidTenant>,
    pub total_orders: i64,
    pub total_personal_orders: i64,
    pub total_invoices: i64,
    pub total_payments: i64,
    pub active_subscriptions: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkProfileOverview {
    pub organization: NidOrganization,
    pub brands: Vec<BrandSummary>,
    pub total_customers: i64,
    pub total_orders: i64,
    pub total_sales: f64,
    pub total_contacts: i64,
    pub active_subscriptions: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalProfileOverview {
    pub profile: NidProfile,
    pub account: Option<NidAccount>,
    pub orders: Vec<PersonalOrder>,
    pub invoices: Vec<NidInvoice>,
    pub payments: Vec<NidPayment>,
    pub subscriptions: Vec<NidSubscription>,
    pub total_spent: f64,
}

// ============================================================================
// NidSyncService - queries nID tables directly from shared PostgreSQL
// ============================================================================

pub struct NidSyncService {
    pool: PgPool,
}

impl NidSyncService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---- Account Lookup ----

    pub async fn get_account_by_phone(&self, phone: &str) -> Result<Option<NidAccount>> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String)>(
            "SELECT id, phone, email, phone_verified_at, created_at FROM account WHERE phone = $1"
        )
        .bind(phone)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NidAccount {
            id: r.0,
            phone: r.1,
            email: r.2,
            phone_verified_at: r.3,
            created_at: r.4,
        }))
    }

    // ---- Profile Queries ----

    pub async fn get_personal_profile(&self, account_id: &str) -> Result<Option<NidProfile>> {
        let row = sqlx::query_as::<_, (String, Option<String>, String, String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, bool)>(
            "SELECT id, account_id, slug, display_name, headline, category, bio, photo_key, phone, public_email, city, timezone, is_published FROM profile WHERE account_id = $1 AND organization_id IS NULL"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NidProfile {
            id: r.0,
            account_id: Some(account_id.to_string()),
            organization_id: None,
            slug: r.2,
            display_name: r.3,
            headline: r.4,
            category: r.5,
            bio: r.6,
            photo_key: r.7,
            phone: r.8,
            whatsapp: None,
            public_email: r.9,
            city: r.10,
            country_code: None,
            timezone: r.11,
            is_published: r.12,
            is_personal: true,
        }))
    }

    pub async fn get_work_profiles(&self, account_id: &str) -> Result<Vec<NidProfile>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, String, String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, bool)>(
            r#"SELECT p.id, p.organization_id, p.slug, p.display_name, p.headline, p.category, p.bio, p.photo_key, p.phone, p.public_email, p.city, p.timezone, p.is_published
               FROM profile p
               JOIN organization_membership om ON p.organization_id = om.organization_id
               WHERE om.account_id = $1 AND om.status = 'ACTIVE'"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidProfile {
            id: r.0,
            account_id: Some(account_id.to_string()),
            organization_id: r.1,
            slug: r.2,
            display_name: r.3,
            headline: r.4,
            category: r.5,
            bio: r.6,
            photo_key: r.7,
            phone: r.8,
            whatsapp: None,
            public_email: r.9,
            city: r.10,
            country_code: None,
            timezone: r.11,
            is_published: r.12,
            is_personal: false,
        }).collect())
    }

    // ---- Organization Queries ----

    pub async fn get_user_organizations(&self, account_id: &str) -> Result<Vec<NidOrganization>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>)>(
            r#"SELECT o.id, o.tenant_id, o.subject_id, o.legal_name, o.registration_number, o.country_code
               FROM organization o
               JOIN organization_membership om ON o.id = om.organization_id
               WHERE om.account_id = $1 AND om.status = 'ACTIVE'"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidOrganization {
            id: r.0,
            tenant_id: r.1,
            subject_id: r.2,
            legal_name: r.3,
            registration_number: r.4,
            country_code: r.5,
            is_verified: false,
            verified_level: None,
            verified_at: None,
        }).collect())
    }

    pub async fn get_organization(&self, org_id: &str) -> Result<Option<NidOrganization>> {
        let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>)>(
            "SELECT id, tenant_id, subject_id, legal_name, registration_number, country_code FROM organization WHERE id = $1"
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NidOrganization {
            id: r.0,
            tenant_id: r.1,
            subject_id: r.2,
            legal_name: r.3,
            registration_number: r.4,
            country_code: r.5,
            is_verified: false,
            verified_level: None,
            verified_at: None,
        }))
    }

    // ---- Membership Queries ----

    pub async fn get_user_memberships(&self, account_id: &str) -> Result<Vec<NidMembership>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, bool, String)>(
            "SELECT id, organization_id, role, is_admin, status FROM organization_membership WHERE account_id = $1"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidMembership {
            id: r.0,
            account_id: Some(account_id.to_string()),
            organization_id: r.1,
            role: r.2,
            is_admin: r.3,
            status: r.4,
        }).collect())
    }

    // ---- Tenant Queries ----

    pub async fn get_tenants(&self) -> Result<Vec<NidTenant>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, name, slug FROM tenant"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidTenant {
            id: r.0,
            name: r.1,
            slug: r.2,
        }).collect())
    }

    // ---- Invoice Queries ----

    pub async fn get_invoices_by_tenant(&self, tenant_id: &str) -> Result<Vec<NidInvoice>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, i64, i64, i64, i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String)>(
            r#"SELECT i.id, i.number, i.status, i.currency, i.subtotal_minor, i.tax_minor, i.total_minor, i.amount_paid_minor,
                      i.period_start, i.period_end, i.due_at, i.issued_at, i.paid_at, t.name
               FROM invoice i
               JOIN tenant t ON i.tenant_id = t.id
               WHERE i.tenant_id = $1
               ORDER BY i.issued_at DESC"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidInvoice {
            id: r.0,
            invoice_number: r.1,
            status: r.2,
            currency: r.3,
            subtotal_minor: r.4,
            tax_minor: r.5,
            total_minor: r.6,
            amount_paid_minor: r.7,
            period_start: r.8,
            period_end: r.9,
            due_at: r.10,
            issued_at: r.11,
            paid_at: r.12,
            tenant_id: tenant_id.to_string(),
            tenant_name: r.13,
        }).collect())
    }

    pub async fn get_invoices_by_account(&self, account_id: &str) -> Result<Vec<NidInvoice>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, i64, i64, i64, i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String)>(
            r#"SELECT i.id, i.number, i.status, i.currency, i.subtotal_minor, i.tax_minor, i.total_minor, i.amount_paid_minor,
                      i.period_start, i.period_end, i.due_at, i.issued_at, i.paid_at, t.name
               FROM invoice i
               JOIN billing_account ba ON i.billing_account_id = ba.id
               JOIN tenant t ON i.tenant_id = t.id
               WHERE ba.tenant_id IN (SELECT tenant_id FROM tenant_staff_membership WHERE account_id = $1)
               ORDER BY i.issued_at DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidInvoice {
            id: r.0,
            invoice_number: r.1,
            status: r.2,
            currency: r.3,
            subtotal_minor: r.4,
            tax_minor: r.5,
            total_minor: r.6,
            amount_paid_minor: r.7,
            period_start: r.8,
            period_end: r.9,
            due_at: r.10,
            issued_at: r.11,
            paid_at: r.12,
            tenant_id: r.13.clone(),
            tenant_name: Some(r.13),
        }).collect())
    }

    // ---- Payment Queries ----

    pub async fn get_payments_by_tenant(&self, tenant_id: &str) -> Result<Vec<NidPayment>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, String, String, Option<String>, Option<String>, String)>(
            r#"SELECT p.id, p.provider, p.provider_transaction_id, p.amount_minor, p.currency, p.status, p.method_type, p.paid_at, t.name
               FROM payment p
               JOIN tenant t ON p.tenant_id = t.id
               WHERE p.tenant_id = $1
               ORDER BY p.paid_at DESC"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidPayment {
            id: r.0,
            provider: r.1,
            provider_transaction_id: r.2,
            amount_minor: r.3,
            currency: r.4,
            status: r.5,
            method_type: r.6,
            paid_at: r.7,
            tenant_id: tenant_id.to_string(),
            tenant_name: r.8,
        }).collect())
    }

    // ---- Subscription Queries ----

    pub async fn get_subscriptions_by_tenant(&self, tenant_id: &str) -> Result<Vec<NidSubscription>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, Option<String>, Option<String>, bool, Option<String>, Option<String>)>(
            r#"SELECT s.id, s.status, pl.key, pl.name, pl.mode, pl.currency, pl.unit_amount_minor,
                      s.current_period_start, s.current_period_end, s.cancel_at_period_end,
                      bs.key, bs.name
               FROM subscription s
               JOIN plan pl ON s.plan_id = pl.id
               JOIN billing_service bs ON pl.service_id = bs.id
               WHERE s.tenant_id = $1"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NidSubscription {
            id: r.0,
            status: r.1,
            plan_key: r.2,
            plan_name: r.3,
            plan_mode: r.4,
            plan_currency: r.5,
            plan_amount: r.6,
            current_period_start: r.7,
            current_period_end: r.8,
            cancel_at_period_end: r.9,
            tenant_id: tenant_id.to_string(),
            service_key: r.10,
            service_name: r.11,
        }).collect())
    }

    // ---- Launchpad Overview ----

    pub async fn get_launchpad_overview(&self, account_id: &str) -> Result<LaunchpadOverview> {
        let personal_profile = self.get_personal_profile(account_id).await?;
        let work_profiles = self.get_work_profiles(account_id).await?;
        let organizations = self.get_user_organizations(account_id).await?;
        let memberships = self.get_user_memberships(account_id).await?;
        let tenants = self.get_tenants().await?;

        // Count totals from Communicator tables
        let total_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_invoices: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoice WHERE tenant_id IN (SELECT tenant_id FROM tenant_staff_membership WHERE account_id = $1)"
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_payments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payment WHERE tenant_id IN (SELECT tenant_id FROM tenant_staff_membership WHERE account_id = $1)"
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let active_subscriptions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM subscription WHERE status = 'ACTIVE' AND tenant_id IN (SELECT tenant_id FROM tenant_staff_membership WHERE account_id = $1)"
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(LaunchpadOverview {
            personal_profile,
            work_profiles,
            organizations,
            memberships,
            tenants,
            total_orders,
            total_personal_orders: 0,
            total_invoices,
            total_payments,
            active_subscriptions,
        })
    }

    // ---- Work Profile Overview ----

    pub async fn get_work_profile_overview(&self, org_id: &str) -> Result<WorkProfileOverview> {
        let org = self.get_organization(org_id).await?.context("Organization not found")?;

        // Get brands for this organization
        let brands = sqlx::query_as::<_, (String, String, String, i64, f64, i64)>(
            r#"SELECT b.id, b.name, b.slug,
                      (SELECT COUNT(*) FROM commerce_orders WHERE brand_id = b.id) as total_orders,
                      (SELECT COALESCE(SUM(total_amount), 0) FROM commerce_orders WHERE brand_id = b.id) as total_sales,
                      (SELECT COUNT(DISTINCT customer_id) FROM commerce_orders WHERE brand_id = b.id) as total_customers
               FROM brands b
               WHERE organization_id = $1"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let brand_summaries: Vec<BrandSummary> = brands.into_iter().map(|r| BrandSummary {
            id: r.0,
            name: r.1,
            slug: r.2,
            total_orders: r.3,
            total_sales: r.4,
            total_customers: r.5,
        }).collect();

        let total_customers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM customers WHERE brand_id IN (SELECT id FROM brands WHERE organization_id = $1)"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders WHERE brand_id IN (SELECT id FROM brands WHERE organization_id = $1)"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_sales: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM commerce_orders WHERE brand_id IN (SELECT id FROM brands WHERE organization_id = $1)"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0.0);

        let total_contacts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM contacts WHERE brand_id IN (SELECT id FROM brands WHERE organization_id = $1)"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let active_subscriptions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM subscription WHERE status = 'ACTIVE' AND tenant_id = $1"
        )
        .bind(&org.tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(WorkProfileOverview {
            organization: org,
            brands: brand_summaries,
            total_customers,
            total_orders,
            total_sales,
            total_contacts,
            active_subscriptions,
        })
    }

    // ---- Personal Profile Overview ----

    pub async fn get_personal_profile_overview(&self, account_id: &str) -> Result<PersonalProfileOverview> {
        let profile = self.get_personal_profile(account_id).await?.context("Profile not found")?;
        let account = self.get_account_by_phone(&profile.phone.clone().unwrap_or_default()).await?;

        let invoices = self.get_invoices_by_account(account_id).await?;
        let payments = self.get_payments_by_tenant(&account_id).await?;
        let subscriptions = self.get_subscriptions_by_tenant(&account_id).await?;

        // Personal orders from commerce_orders
        let orders = sqlx::query_as::<_, (String, String, String, f64, String, String, String, String, Option<String>, Option<String>, String)>(
            r#"SELECT co.id, co.order_number, co.status, co.total_amount, co.currency,
                      opi.platform_name, opi.platform_type, opi.brand_name, opi.shop_name, opi.platform_logo_url, co.created_at
               FROM commerce_orders co
               LEFT JOIN order_platform_info opi ON co.id = opi.order_id
               WHERE co.customer_id = $1
               ORDER BY co.created_at DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let personal_orders: Vec<PersonalOrder> = orders.into_iter().map(|r| PersonalOrder {
            id: r.0,
            order_number: r.1,
            status: r.2,
            total_amount: r.3,
            currency: r.4,
            platform_name: r.5,
            platform_type: r.6,
            brand_name: r.7,
            shop_name: r.8,
            platform_logo_url: r.9,
            created_at: r.10,
        }).collect();

        let total_spent: f64 = payments.iter()
            .filter(|p| p.status == "SUCCEEDED")
            .map(|p| p.amount_minor as f64 / 100.0)
            .sum();

        Ok(PersonalProfileOverview {
            profile,
            account,
            orders: personal_orders,
            invoices,
            payments,
            subscriptions,
            total_spent,
        })
    }

    // ---- Order Platform Info ----

    pub async fn set_order_platform_info(&self, order_id: &str, platform_name: &str, platform_type: &str, brand_name: &str, shop_name: Option<&str>) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO order_platform_info (id, order_id, platform_name, platform_type, brand_name, shop_name, synced_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (order_id) DO UPDATE SET platform_name = $3, platform_type = $4, brand_name = $5, shop_name = $6, synced_at = $7"#
        )
        .bind(&id)
        .bind(order_id)
        .bind(platform_name)
        .bind(platform_type)
        .bind(brand_name)
        .bind(shop_name.map(|s| s.to_string()))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ---- Profile Cache ----

    pub async fn cache_profile(&self, profile: &NidProfile) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO nid_profile_cache (id, nid_profile_id, nid_account_id, nid_organization_id, slug, display_name, headline, category, bio, photo_key, phone, whatsapp, public_email, city, country_code, timezone, is_published, is_personal, synced_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
               ON CONFLICT (nid_profile_id) DO UPDATE SET slug = $5, display_name = $6, synced_at = $19"#
        )
        .bind(&id)
        .bind(&profile.id)
        .bind(&profile.account_id)
        .bind(&profile.organization_id)
        .bind(&profile.slug)
        .bind(&profile.display_name)
        .bind(&profile.headline)
        .bind(&profile.category)
        .bind(&profile.bio)
        .bind(&profile.photo_key)
        .bind(&profile.phone)
        .bind(&profile.whatsapp)
        .bind(&profile.public_email)
        .bind(&profile.city)
        .bind(&profile.country_code)
        .bind(&profile.timezone)
        .bind(profile.is_published)
        .bind(profile.is_personal)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
