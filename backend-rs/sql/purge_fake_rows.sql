-- ===========================================================================
-- purge_fake_rows.sql
-- ===========================================================================
-- PURPOSE
--   Delete the fabricated demo/placeholder rows that the (now-deleted) seeders
--   already wrote into real product tables, so the API returns only genuine
--   data instead of Chatwoot-style example content.
--
-- TARGET
--   The canonical Rust/SQLite database: backend-rs/communicator.db
--   (Python/Postgres rows are covered by the predicates in PART D, which are
--   the same literal values -- the two seeders were line-for-line mirrors.)
--
--   See README_purge.md for the exact apply command. NOT run automatically.
--
-- HOW EVERY PREDICATE WAS DERIVED
--   From the literal values in the seeders themselves (read from the deleted
--   Python code via `git show HEAD:<path>` and from the still-present Rust
--   `seed_default_* / ensure_default_* / *_if_empty` helpers), never from
--   guesses. Each DELETE names its source in the comment above it.
--
-- ORDERING
--   Children before parents (broadcast_recipients, ticket_activities,
--   webhook_delivery_logs, sequence_steps, simulation_runs). The SQLite schema
--   in src/db/mod.rs declares no FK constraints, so the order is defensive and
--   also correct if this script is retargeted at Postgres.
--
-- DELIBERATELY NOT TOUCHED
--   * role_definitions system roles (admin/manager/agent/restricted_agent/
--     compliance_auditor) -- permission model required by authz, not demo data.
--   * stripe_configs row '00000000-0000-0000-0000-000000000002' (default, empty
--     keys, created lazily by get_or_create_stripe_config_internal) -- PART B.
--   * tenants rows (create_organization writes owner_account_id='user_admin'
--     from a real POST body) -- PART B.
--   * any row whose values came from a user POST/PUT (gateway/webhook/
--     automation/tracker config endpoints all upsert from payload).
-- ===========================================================================

PRAGMA foreign_keys = OFF;
BEGIN;

-- ---------------------------------------------------------------------------
-- PART A1 -- CHILD ROWS OF PROVABLY SEEDED PARENTS
-- ---------------------------------------------------------------------------

-- Predicate: recipients belonging to the 3 example campaigns.
-- Seeder: Rust routes/broadcasts.rs::seed_default_campaigns (INSERT OR REPLACE
--         INTO broadcasts + broadcast_recipients, 'Spring sale early access',
--         'New store opening', 'Loyalty points reminder') and the deleted Python
--         api/routes/broadcasts.py POST /broadcasts/seed-defaults.
DELETE FROM broadcast_recipients
 WHERE broadcast_id IN (
     SELECT id FROM broadcasts
      WHERE name IN ('Spring sale early access', 'New store opening', 'Loyalty points reminder')
         OR template_name IN ('spring_sale_offer', 'store_launch_announcement', 'loyalty_rewards_update')
 );

-- Predicate: the 3 hard-coded placeholder contact ids the Rust seeder inserted.
-- Seeder: routes/broadcasts.rs::seed_default_campaigns ->
--         VALUES (..., 'contact-1', 'read', ...), (..., 'contact-2', ...), (..., 'contact-3', ...).
DELETE FROM broadcast_recipients
 WHERE contact_id IN ('contact-1', 'contact-2', 'contact-3');

-- Predicate: activity log rows of the 6 demo tickets.
-- Seeder: deleted Python services/ticket_service.py::seed_demo_tickets_if_empty
--         (4 TicketActivity rows: 'Captain AI Incident Sentinel', 'Sarah (Support)',
--         'David (Finance)', 'Marcus (Warehouse Lead)').
DELETE FROM ticket_activities
 WHERE ticket_id IN (
     SELECT id FROM tickets
      WHERE ticket_number IN ('TCK-1001','TCK-1002','TCK-1003','TCK-1004','TCK-1005','TCK-1006')
 );

-- Belt-and-braces: the same activities matched by their fabricated author names
-- (covers tickets whose number was later edited by hand).
DELETE FROM ticket_activities
 WHERE author_name IN ('Captain AI Incident Sentinel', 'Sarah (Support)',
                       'David (Finance)', 'Marcus (Warehouse Lead)');

-- Predicate: linked issues hanging off the 6 demo tickets.
-- Seeder: same ticket_demo seeder (tracker links are only ever created for those
--         tickets; no real ticket used TCK-1001..1006).
DELETE FROM linked_issues
 WHERE ticket_id IN (
     SELECT id FROM tickets
      WHERE ticket_number IN ('TCK-1001','TCK-1002','TCK-1003','TCK-1004','TCK-1005','TCK-1006')
 );

-- Predicate: delivery logs of the two fabricated webhook endpoints.
-- Seeder: Rust gateways_webhooks_stripe.rs::seed_default_webhooks_if_empty +
--         deleted Python services/webhook_service.py::seed_default_webhooks_if_empty.
DELETE FROM webhook_delivery_logs
 WHERE endpoint_id IN (
     SELECT id FROM webhook_endpoints
      WHERE secret IN ('whsec_prod_991823746152889201928374',
                       'whsec_slack_alert_token_551829',
                       'whsec_ops_771826451029384756192837')
         OR url IN ('https://api.internal.bayworks.co/webhooks/crm-sync',
                    'https://hooks.slack.com/services/T00/B00/mock123',
                    'https://hooks.slack.com/services/T000/B000/XXXXXmock')
 );

-- Predicate: steps of the two example follow-up sequences.
-- Seeder: Rust routes/services_sequences.rs::list_services/list_sequences
--         (if seqs.is_empty() -> INSERT 'seq-1','seq-2' + sequence_steps).
DELETE FROM sequence_steps
 WHERE sequence_id IN ('seq-1', 'seq-2');

-- Predicate: simulation runs of the 4 example scenarios.
-- Seeder: deleted Python services/simulation_service.py::seed_default_policies_and_scenarios.
DELETE FROM simulation_runs
 WHERE scenario_id IN (
     SELECT id FROM simulation_scenarios
      WHERE name IN ('Damaged Goods: $42 Perfume Bottle Autonomous Resolution',
                     'High-Value Return: $280 Chronograph Watch (Requires Approval)',
                     'Expedited Shipping Delay: Carrier Weather Exception',
                     'VIP Exchange: Running Shoes Size Swap')
 );

-- ---------------------------------------------------------------------------
-- PART A2 -- SEEDED PARENT ROWS
-- ---------------------------------------------------------------------------

-- Predicate: campaign name / template name literals.
-- Seeder: Rust routes/broadcasts.rs::seed_default_campaigns + deleted Python
--         api/routes/broadcasts.py POST /broadcasts/seed-defaults
--         (fake counters 248/240/218/164/12/8, body 'Hi Maya, our spring sale starts...').
DELETE FROM broadcasts
 WHERE name IN ('Spring sale early access', 'New store opening', 'Loyalty points reminder')
    OR template_name IN ('spring_sale_offer', 'store_launch_announcement', 'loyalty_rewards_update');

-- Predicate: demo ticket numbers + fabricated titles.
-- Seeder: deleted Python services/ticket_service.py::seed_demo_tickets_if_empty.
DELETE FROM tickets
 WHERE ticket_number IN ('TCK-1001','TCK-1002','TCK-1003','TCK-1004','TCK-1005','TCK-1006')
    OR title IN ('Widespread Incident: US-East Stripe Payment 504 Gateway Timeouts',
                 'Card charged $89.00 but order showed checkout error',
                 'Cart emptied on pay button press without confirmation',
                 'Finance Back-Office: Audit duplicated authorizations for incident TCK-1001',
                 'Logistics & Warehouse: Batch #412 Glass Packaging Reinforcement',
                 'Carrier Incident: Winter Blizzard Delay - Midwest Regional Hubs');

-- Predicate: the 6 auto-installed canned-response short codes (all addressed to
--            the placeholder customer "Bay Works").
-- Seeder: Rust canned_macros_sla.rs::ensure_default_canned + deleted Python
--         api/routes/canned_responses.py::_ensure_default_canned.
DELETE FROM canned_responses
 WHERE short_code IN ('greet', 'pricing', 'hours', 'refund', 'followup', 'closing');

-- Predicate: the 4 auto-installed macro names.
-- Seeder: Rust canned_macros_sla.rs::ensure_default_macros + deleted Python
--         services/macro_executor.py::seed_default_macros (DEFAULT_SYSTEM_MACROS).
DELETE FROM macros
 WHERE name IN ('Resolve Ticket + Send CSAT + Tag ''Resolved''',
               'Escalate to Tier 2 + Mark Urgent + Assign Support Team',
               'Send Password Reset Link + Snooze 24h',
               'VIP Refund Approved + Assign Billing');

-- Predicate: the 3 auto-installed SLA policy names. (Their fabricated thresholds
--            were 15/120 urgent, 30/360 high, 60/1440 medium+low -- used only as
--            a human cross-check, not as a predicate, because those numbers are
--            legitimate values a real operator could choose.)
-- Seeder: Rust canned_macros_sla.rs::ensure_default_sla_policies + deleted Python
--         services/sla_service.py (DEFAULT_SLA_POLICIES).
DELETE FROM sla_policies
 WHERE name IN ('Critical VIP Escalation SLA', 'High Priority SLA', 'Standard Customer Support SLA');

-- Predicate: the auto-installed WhatsApp automation rule set, matched by the
--            exact trigger_keyword literals the seeders inserted.
-- Seeder: Rust routes/automation.rs::seed_default_automations (11 rules, incl.
--         'stop' / 'Opt-Out Compliance (STOP)') + deleted Python
--         services/whatsapp_automations.py section 8 (10 rules) and
--         api/routes/automation.py POST /automation-rules/seed-defaults.
DELETE FROM automation_rules
 WHERE trigger_keyword IN ('menu', 'agent', 'human', 'services', 'pricing', 'demo',
                           'location', 'csat', 'quote', 'enterprise', 'stop')
   AND name IN ('Main Interactive Menu', 'Live Human Support Handoff',
                'Human Agent Alias (Operator)', 'Services Catalog Quick View',
                'Pricing & Rates Inquiry', 'Book a Meeting / Demo CTA',
                'Office Location & Timings', 'Post-Resolution Feedback / CSAT Survey',
                'High-Intent Lead Auto-Tagging', 'VIP Customer Auto-Tagging',
                'Opt-Out Compliance (STOP)');

-- Predicate: the two placeholder service catalogue rows (fixed ids srv-1/srv-2).
-- Seeder: Rust routes/services_sequences.rs::list_services (if services.is_empty()).
DELETE FROM services
 WHERE id IN ('srv-1', 'srv-2')
    OR name IN ('Enterprise Omnichannel Support', 'AI Copilot & Autonomous Routing');

-- Predicate: the two placeholder sequences (fixed ids seq-1/seq-2).
-- Seeder: Rust routes/services_sequences.rs::list_sequences (if seqs.is_empty()).
DELETE FROM follow_up_sequences
 WHERE id IN ('seq-1', 'seq-2')
    OR name IN ('New Inbound Lead Onboarding', 'Enterprise Re-Engagement Sequence');

-- Predicate: gateway rows created by the if_empty seeding, by name.
-- Seeder: Rust gateways_webhooks_stripe.rs::seed_default_gateways_if_empty
--         (9 rows) + deleted Python services/gateway_service.py::seed_default_gateways_if_empty
--         (11 rows). Union of both literal name sets.
DELETE FROM channel_gateway_connections
 WHERE name IN ('Twilio SMS Global', 'Vonage SMS (Nexmo)', 'MessageBird Enterprise SMS',
                'MessageBird SMS Europe', 'SendGrid Production Mailer', 'SendGrid Transactional API',
                'Postmark Transactional', 'Custom Corporate SMTP', 'Corporate SMTP Server (TLS)',
                'Custom HTTP REST SMS Gateway', 'AWS SES Production Mailer', 'Resend Modern Developer Email',
                'Twilio Voice & WebRTC', 'Twilio Voice & WebRTC Softphone', 'Telnyx SIP & Voice',
                'Telnyx Global SIP & WebRTC', 'Standard SIP Trunk (Asterisk/FreeSWITCH)',
                'Generic SIP Trunk / FreeSWITCH PBX');

-- Predicate: the same seeded gateways caught by their fake credential literals,
--            in case an operator renamed them (name-independent safety net).
-- Seeder: same two *_gateways_if_empty helpers (credentials JSON literals).
DELETE FROM channel_gateway_connections
 WHERE credentials LIKE '%AC_mock_twilio_sms_sid%'
    OR credentials LIKE '%mock_token_twilio_sms%'
    OR credentials LIKE '%mock_vonage_key_8492%'
    OR credentials LIKE '%mock_vonage_secret_7721%'
    OR credentials LIKE '%live_mock_mb_key_29910%'
    OR credentials LIKE '%live_mock_bird_key_9931%'
    OR credentials LIKE '%gw_sec_custom_token_9918%'
    OR credentials LIKE '%SG.mock_sendgrid_key_991823%'
    OR credentials LIKE '%SG.mock_sendgrid_key_098172635%'
    OR credentials LIKE '%mock_postmark_server_token%'
    OR credentials LIKE '%secret_smtp_password_123%'
    OR credentials LIKE '%AKIA_MOCK_SES_ACCESS_KEY%'
    OR credentials LIKE '%re_mock_resend_api_key_77182%'
    OR credentials LIKE '%AC_mock_twilio_voice_sid%'
    OR credentials LIKE '%mock_token_twilio_voice%'
    OR credentials LIKE '%KEY_mock_telnyx_sip_key%'
    OR credentials LIKE '%KEY_telnyx_mock_credentials_99182%'
    OR credentials LIKE '%secret_sip_trunk_pass%'
    OR credentials LIKE '%sip_pbx_secret_password%'
    OR config LIKE '%MG_mock_service%'
    OR config LIKE '%AP_mock_twiml_sid%'
    OR config LIKE '%conn_mock_telnyx_1122%';

-- Predicate: webhook endpoints by their fabricated secrets/URLs/names.
-- Seeder: Rust gateways_webhooks_stripe.rs::seed_default_webhooks_if_empty
--         ('Enterprise CRM & Data Warehouse Sync', 'Slack Incident Alert Dispatcher')
--         + deleted Python services/webhook_service.py::seed_default_webhooks_if_empty
--         ('Enterprise CRM & Data Warehouse Sync', 'Incident & Outage Alerts Slack Connector').
DELETE FROM webhook_endpoints
 WHERE secret IN ('whsec_prod_991823746152889201928374',
                  'whsec_slack_alert_token_551829',
                  'whsec_ops_771826451029384756192837')
    OR url IN ('https://api.internal.bayworks.co/webhooks/crm-sync',
               'https://hooks.slack.com/services/T00/B00/mock123',
               'https://hooks.slack.com/services/T000/B000/XXXXXmock')
    OR name IN ('Enterprise CRM & Data Warehouse Sync',
                'Slack Incident Alert Dispatcher',
                'Incident & Outage Alerts Slack Connector');

-- Predicate: AI provider rows with the hard-coded placeholder API keys.
-- Seeder: deleted Python services/ai_connection_service.py::seed_default_connections_and_policies
--         ('sk-ant-api03-default-configured', 'sk-proj-default-configured',
--          test_status='connected' latency 165/180).
DELETE FROM ai_connections
 WHERE api_key IN ('sk-ant-api03-default-configured', 'sk-proj-default-configured')
    OR name IN ('Anthropic Claude 3.5 Sonnet', 'OpenAI GPT-4o', 'Google Gemini 1.5 Pro',
                'DeepSeek R1 (Reasoning)', 'Local Ollama (Llama 3.3 / Qwen)');

-- Predicate: the 4 AI job-policy rows installed alongside those connections.
-- Seeder: same deleted seeder (AICustomPolicy rows for job_type
--         self_improving / autonomous_agent / copilot_drafts / simulations).
-- RISK: job_type is UNIQUE -- if an operator later saved their own policy for one
--       of these jobs, this DELETE removes their work. Run the ai_custom_policies
--       SELECT in the VERIFY section first and keep only the untouched defaults.
DELETE FROM ai_custom_policies
 WHERE job_type IN ('self_improving', 'autonomous_agent', 'copilot_drafts', 'simulations');

-- Predicate: the 3 demo SOP policy documents.
-- Seeder: deleted Python services/simulation_service.py::seed_default_policies_and_scenarios.
DELETE FROM agent_policies
 WHERE name IN ('Return & Refund Policy (SOP-REF-201)',
                'Shipping Delays & Carrier Exceptions (SOP-SHP-104)',
                'VIP Customer Protocol (SOP-VIP-301)');

-- Predicate: the 4 demo simulation scenarios.
-- Seeder: same deleted simulation_service.py seeder.
DELETE FROM simulation_scenarios
 WHERE name IN ('Damaged Goods: $42 Perfume Bottle Autonomous Resolution',
                'High-Value Return: $280 Chronograph Watch (Requires Approval)',
                'Expedited Shipping Delay: Carrier Weather Exception',
                'VIP Exchange: Running Shoes Size Swap');

-- Predicate: the 4 fabricated insight titles (and their invented CSAT lifts).
-- Seeder: deleted Python services/self_improving_service.py::seed_default_insights_and_retraining.
DELETE FROM agent_insights
 WHERE title IN ('Exemplar from Sarah (Support Lead): 5-Star Shipping De-escalation Phrasing',
                 'Exemplar from Marcus (Senior Tier): High-Value Return Courtesy Handoff',
                 'Emerging Trend: Surge in Eco-Friendly & Recyclable Packaging Inquiries',
                 'Team Coaching: Providing Direct Carrier Drop-off Links Boosts CSAT by 24%');

-- Predicate: the fabricated baseline retraining event v2.1 -> v2.2.
-- Seeder: same deleted self_improving_service.py seeder.
DELETE FROM agent_retraining_events
 WHERE title = 'Retraining Cycle v2.2 - Carrier Delays & De-escalation Protocol';

-- Predicate: the 9 auto-installed custom attribute definitions.
-- Seeder: deleted Python services/custom_attribute_service.py::initialize_default_attributes
--         (DEFAULT_ATTRIBUTES).
DELETE FROM custom_attribute_definitions
 WHERE name IN ('account_tier', 'company_size', 'annual_revenue', 'is_vip', 'renewal_date',
                'issue_category', 'escalation_level', 'billing_dispute_amount', 'jira_key');

-- Predicate: issue-tracker configs carrying the demo tokens only (Python seeder).
-- Seeder: deleted Python services/issue_tracker_service.py::seed_default_tracker_configs_if_empty
--         (Linear 'Engineering Core Workspace', GitHub 'Newax Organization',
--          Jira 'Enterprise Atlassian Cloud', Slack 'Primary Team Slack').
-- NOTE: provider is UNIQUE, so the credential literal -- not the provider name --
--       is the safe discriminator. Rust never seeded this table.
DELETE FROM issue_tracker_configs
 WHERE credentials LIKE '%lin_api_demo_secret_token_123%'
    OR credentials LIKE '%ghp_demo_secret_token_456%'
    OR credentials LIKE '%jira_api_token_789%'
    OR credentials LIKE '%xoxb-demo-slack-bot-token-999%'
    OR account_name IN ('Engineering Core Workspace', 'Newax Organization',
                        'Enterprise Atlassian Cloud', 'Primary Team Slack');

-- NOTE: no seeder ever fabricated Contact / Conversation / Inbox rows, so those
--       tables are deliberately untouched here (the seeded campaigns' audience
--       counts were fake numbers on the broadcast row, not real recipient rows).
--       Contact ids 'contact-1..3' are removed only from broadcast_recipients,
--       above, never from the contacts table itself.
-- ---------------------------------------------------------------------------
-- PART B -- LEFT FOR HUMAN REVIEW (intentionally NOT deleted by this script)
-- ---------------------------------------------------------------------------
-- These rows are fabricated too, but they are produced by code paths that are
-- still live (dispatch simulation, automation nudges, gateway test sends), not
-- by the deleted seeders. Deleting them silently could remove rows a customer
-- considers real. Inspect with the SELECTs below, then uncomment.
--
--   -- routes/broadcasts.rs simulated dispatch ("provider_message_id = meta_sim_<uuid>",
--   -- sender_type='system', sender_name='Campaign Broadcast'):
--   -- SELECT id, conversation_id, body FROM messages WHERE provider_message_id LIKE 'meta_sim_%';
--   -- DELETE FROM messages WHERE sender_type = 'system' AND sender_name = 'Campaign Broadcast'
--   --   AND provider_message_id LIKE 'meta_sim_%';
--   -- DELETE FROM broadcast_recipients WHERE provider_message_id LIKE 'meta_sim_%';
--
--   -- routes/automation.rs inactivity nudge bot messages (body 'Hi there, just checking in ...'):
--   -- SELECT id, conversation_id FROM messages WHERE provider_message_id = 'inactivity_nudge'
--   --   OR intent = 'inactivity_nudge';
--   -- DELETE FROM messages WHERE provider_message_id = 'inactivity_nudge' OR intent = 'inactivity_nudge';
--
--   -- routes/inboxes.rs::record_channel_message synthetic conversations
--   -- (id LIKE 'conv_%', e.g. 'conv_whatsapp') and their 'Communicator Gateway' messages:
--   -- SELECT id, channel, contact_phone FROM conversations WHERE id LIKE 'conv_%';
--   -- DELETE FROM messages WHERE conversation_id IN (SELECT id FROM conversations WHERE id LIKE 'conv_%');
--   -- DELETE FROM conversations WHERE id LIKE 'conv_%';
--
--   -- Fake display name written into audit/assignment columns
--   -- (routes/canned_macros_sla.rs `unwrap_or_else(|| "Riley Brooks")`,
--   --  routes/admin_system.rs "assignedAgent": "Riley Brooks"):
--   -- SELECT id, sender_name FROM messages WHERE sender_name = 'Riley Brooks';
--
--   -- 'AC_TWILIO_PROD_001' and '+923089270737' in routes/inboxes.rs are response
--   -- fallbacks for unconfigured integrations, not stored rows: nothing to purge.
--   -- Verify they are absent:
--   -- SELECT id, name, credentials FROM channel_gateway_connections WHERE credentials LIKE '%AC_TWILIO_PROD_001%';
--   -- SELECT id, name, phone_number FROM inboxes WHERE phone_number = '+923089270737';
--
--   -- stripe_configs default row (empty keys, get_or_create_stripe_config_internal)
--   -- and tenants.owner_account_id='user_admin' (create_organization from a real POST):
--   -- both are lazy defaults, not demo data. NOT purged.
--   -- SELECT id, account_name, api_key FROM stripe_configs;
--   -- SELECT id, tenant_key, owner_account_id FROM tenants;

-- ---------------------------------------------------------------------------
-- PART C -- TABLES THAT DO NOT EXIST IN THE RUST SQLite SCHEMA (commented out)
-- ---------------------------------------------------------------------------
-- src/db/mod.rs never creates these. If `sqlite3 backend-rs/communicator.db ".tables"`
-- (or the Postgres database, PART D) does contain them, uncomment the matching block
-- and run it in the same order (memberships/gaps before parents).
--
--   -- Python services/routing_service.py DEFAULT_TEAM_MEMBERS (auto-inserted in list_agents)
--   -- and Python api/routes/teams.py / Rust team_routing.rs::seed_default_teams
--   -- ('Sales & Growth', 'Customer Support', 'VIP Accounts').
--   -- DELETE FROM team_memberships WHERE team_id IN (SELECT id FROM teams WHERE name IN ('Sales & Growth','Customer Support','VIP Accounts'))
--   --    OR team_member_id IN (SELECT id FROM team_members WHERE email IN ('riley.brooks@bayworks.com','jamie.cole@bayworks.com','alex.rivera@bayworks.com','morgan.vance@bayworks.com'));
--   -- DELETE FROM team_members WHERE email IN ('riley.brooks@bayworks.com','jamie.cole@bayworks.com','alex.rivera@bayworks.com','morgan.vance@bayworks.com');
--   -- DELETE FROM teams WHERE name IN ('Sales & Growth', 'Customer Support', 'VIP Accounts');
--
--   -- Python services/knowledge_hub_service.py::ensure_default_hub_content (7 articles)
--   -- and ::ensure_default_recommendations (3 suggestions).
--   -- DELETE FROM knowledge_base_entries WHERE title IN (
--   --   'Refund Processing & Timeline Guidelines','Invoice & Receipt Download Instructions',
--   --   'Domestic & Express Courier Delivery Timelines','Damaged or Lost Shipment Claims Process',
--   --   'Modifying Shipping Address Post-Order','Pure French Linen Washing & Care Instructions',
--   --   'VIP Platinum Tier Benefits & Expedited Shipping');
--   -- DELETE FROM knowledge_recommendations WHERE title IN (
--   --   'Missing International Customs Duties & VAT Exemption Policy',
--   --   'Carrier Tracking Live API Connection Needed for Parcel Status',
--   --   'Automated 1-Click Subscription Cancellation Workflow');
--
--   -- Python services/knowledge_gaps.py `starter_gaps` (fabricated frequency counts).
--   -- DELETE FROM knowledge_gaps WHERE question IN (
--   --   'What is the standard turnaround time for refunds?',
--   --   'Do you provide enterprise SLA guarantees with 99.99% uptime?',
--   --   'Can I connect custom webhook endpoints for incoming WhatsApp messages?');
--
--   -- Python services/segment_service.py::seed_default_segments (audience segments).
--   -- DELETE FROM segments WHERE name IN ('All Omnichannel Contacts','WhatsApp VIP Audience',
--   --   'Email & SMS Subscribers','Social & TikTok Community');

-- ---------------------------------------------------------------------------
-- PART D -- PYTHON / POSTGRES MIRROR
-- ---------------------------------------------------------------------------
-- The deleted Python seeders wrote to Postgres
-- (postgresql+asyncpg://.../communicator), NOT to this SQLite file, so PART A
-- alone does not clean the legacy database. The same PART A predicates apply, with
-- these differences:
--   * table names are the SQLAlchemy snake_case names above plus: teams,
--     team_members, team_memberships, knowledge_base_entries, knowledge_recommendations,
--     knowledge_gaps, segments, contact_notes, drafts (PART C blocks).
--   * enum columns store lowercase values ('medium', 'urgent', 'sms', 'email',
--     'voice_sip', 'twilio', 'smtp', ...) -- the name/keyword/secret predicates used
--     here are case-exact literals from the seeders and need no change.
--   * run with:  PGPASSWORD=development-only psql -h localhost -U postgres -d communicator \
--                  -v ON_ERROR_STOP=1 -f purge_fake_rows.sql
--     (documented only -- do not run without approval and a pg_dump backup).

COMMIT;
PRAGMA foreign_keys = ON;

-- ===========================================================================
-- VERIFY -- re-run after the purge; every count should be 0 for seeded shapes.
-- ===========================================================================
-- SELECT 'broadcasts',            COUNT(*) FROM broadcasts
--   WHERE name IN ('Spring sale early access','New store opening','Loyalty points reminder')
--      OR template_name IN ('spring_sale_offer','store_launch_announcement','loyalty_rewards_update')
-- UNION ALL SELECT 'broadcast_recipients', COUNT(*) FROM broadcast_recipients
--   WHERE contact_id IN ('contact-1','contact-2','contact-3')
-- UNION ALL SELECT 'tickets',       COUNT(*) FROM tickets
--   WHERE ticket_number IN ('TCK-1001','TCK-1002','TCK-1003','TCK-1004','TCK-1005','TCK-1006')
-- UNION ALL SELECT 'ticket_activities', COUNT(*) FROM ticket_activities
--   WHERE author_name IN ('Captain AI Incident Sentinel','Sarah (Support)','David (Finance)','Marcus (Warehouse Lead)')
-- UNION ALL SELECT 'canned_responses',  COUNT(*) FROM canned_responses
--   WHERE short_code IN ('greet','pricing','hours','refund','followup','closing')
-- UNION ALL SELECT 'macros',        COUNT(*) FROM macros
--   WHERE name IN ('Resolve Ticket + Send CSAT + Tag ''Resolved''','Escalate to Tier 2 + Mark Urgent + Assign Support Team',
--                  'Send Password Reset Link + Snooze 24h','VIP Refund Approved + Assign Billing')
-- UNION ALL SELECT 'sla_policies',  COUNT(*) FROM sla_policies
--   WHERE name IN ('Critical VIP Escalation SLA','High Priority SLA','Standard Customer Support SLA')
-- UNION ALL SELECT 'automation_rules', COUNT(*) FROM automation_rules
--   WHERE name IN ('Main Interactive Menu','Live Human Support Handoff','Human Agent Alias (Operator)',
--                  'Services Catalog Quick View','Pricing & Rates Inquiry','Book a Meeting / Demo CTA',
--                  'Office Location & Timings','Post-Resolution Feedback / CSAT Survey',
--                  'High-Intent Lead Auto-Tagging','VIP Customer Auto-Tagging','Opt-Out Compliance (STOP)')
-- UNION ALL SELECT 'channel_gateway_connections', COUNT(*) FROM channel_gateway_connections
--   WHERE credentials LIKE '%mock%' OR credentials LIKE '%demo%' OR credentials LIKE '%secret_smtp_password%'
-- UNION ALL SELECT 'webhook_endpoints', COUNT(*) FROM webhook_endpoints
--   WHERE url LIKE '%bayworks%' OR url LIKE '%hooks.slack.com/services/T0%'
-- UNION ALL SELECT 'webhook_delivery_logs', COUNT(*) FROM webhook_delivery_logs
-- UNION ALL SELECT 'services',      COUNT(*) FROM services WHERE id IN ('srv-1','srv-2')
-- UNION ALL SELECT 'follow_up_sequences', COUNT(*) FROM follow_up_sequences WHERE id IN ('seq-1','seq-2')
-- UNION ALL SELECT 'sequence_steps',COUNT(*) FROM sequence_steps WHERE sequence_id IN ('seq-1','seq-2')
-- UNION ALL SELECT 'ai_connections',COUNT(*) FROM ai_connections
--   WHERE api_key IN ('sk-ant-api03-default-configured','sk-proj-default-configured')
-- UNION ALL SELECT 'ai_custom_policies', COUNT(*) FROM ai_custom_policies
--   WHERE job_type IN ('self_improving','autonomous_agent','copilot_drafts','simulations')
-- UNION ALL SELECT 'agent_policies',COUNT(*) FROM agent_policies
--   WHERE name IN ('Return & Refund Policy (SOP-REF-201)','Shipping Delays & Carrier Exceptions (SOP-SHP-104)','VIP Customer Protocol (SOP-VIP-301)')
-- UNION ALL SELECT 'simulation_scenarios', COUNT(*) FROM simulation_scenarios
--   WHERE name LIKE 'Damaged Goods: $42%' OR name LIKE 'High-Value Return: $280%'
--      OR name = 'Expedited Shipping Delay: Carrier Weather Exception' OR name = 'VIP Exchange: Running Shoes Size Swap'
-- UNION ALL SELECT 'simulation_runs', COUNT(*) FROM simulation_runs
-- UNION ALL SELECT 'agent_insights',COUNT(*) FROM agent_insights
--   WHERE title LIKE 'Exemplar from %' OR title LIKE 'Emerging Trend: Surge in Eco-Friendly%'
--      OR title LIKE 'Team Coaching: Providing Direct Carrier%'
-- UNION ALL SELECT 'agent_retraining_events', COUNT(*) FROM agent_retraining_events
--   WHERE title = 'Retraining Cycle v2.2 - Carrier Delays & De-escalation Protocol'
-- UNION ALL SELECT 'custom_attribute_definitions', COUNT(*) FROM custom_attribute_definitions
--   WHERE name IN ('account_tier','company_size','annual_revenue','is_vip','renewal_date',
--                  'issue_category','escalation_level','billing_dispute_amount','jira_key')
-- UNION ALL SELECT 'issue_tracker_configs', COUNT(*) FROM issue_tracker_configs
--   WHERE credentials LIKE '%demo%' OR account_name IN ('Engineering Core Workspace','Newax Organization',
--                  'Enterprise Atlassian Cloud','Primary Team Slack')
-- UNION ALL SELECT 'linked_issues', COUNT(*) FROM linked_issues
--   WHERE ticket_id IN (SELECT id FROM tickets WHERE ticket_number LIKE 'TCK-10%');
--
-- Total rows still living in each purged table (expected: whatever a human created):
-- SELECT 'broadcasts' AS t, COUNT(*) AS n FROM broadcasts
-- UNION ALL SELECT 'tickets', COUNT(*) FROM tickets
-- UNION ALL SELECT 'canned_responses', COUNT(*) FROM canned_responses
-- UNION ALL SELECT 'macros', COUNT(*) FROM macros
-- UNION ALL SELECT 'sla_policies', COUNT(*) FROM sla_policies
-- UNION ALL SELECT 'automation_rules', COUNT(*) FROM automation_rules
-- UNION ALL SELECT 'channel_gateway_connections', COUNT(*) FROM channel_gateway_connections
-- UNION ALL SELECT 'webhook_endpoints', COUNT(*) FROM webhook_endpoints
-- UNION ALL SELECT 'services', COUNT(*) FROM services
-- UNION ALL SELECT 'ai_connections', COUNT(*) FROM ai_connections;
