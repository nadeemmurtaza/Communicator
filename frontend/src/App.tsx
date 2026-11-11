import { Navigate, Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { ProtectedRoute } from './components/ProtectedRoute'
import { Login } from './pages/Login'
import { NidCallback } from './pages/NidCallback'
import { Organizations } from './pages/Organizations'
import { Automation } from './pages/Automation'
import { Broadcasts } from './pages/Broadcasts'
import { ChannelsHub } from './pages/ChannelsHub'
import { Connections } from './pages/Connections'
import { Contacts } from './pages/Contacts'
import { ContactProfile } from './pages/ContactProfile'
import { ConversationDetail } from './pages/ConversationDetail'
import { Conversations } from './pages/Conversations'
import { Dashboard } from './pages/Dashboard'
import { Drafts } from './pages/Drafts'
import { Reports } from './pages/Reports'
import { KnowledgeBase } from './pages/KnowledgeBase'
import { Sequences } from './pages/Sequences'
import { Services } from './pages/Services'
import { CommerceDashboard } from './pages/CommerceDashboard'
import { CatalogManagement } from './pages/CatalogManagement'
import { ProductDetail } from './pages/ProductDetail'
import { ProductOptions } from './pages/ProductOptions'
import { CartManagement } from './pages/CartManagement'
import { OrdersList } from './pages/OrdersList'
import { OrderDetail } from './pages/OrderDetail'
import { SyncSettings } from './pages/SyncSettings'
import { AnalyticsDashboard } from './pages/AnalyticsDashboard'
import { CouponsManagement } from './pages/CouponsManagement'
import { ShippingSettings } from './pages/ShippingSettings'
import { InventoryManagement } from './pages/InventoryManagement'
import { StorefrontSettings } from './pages/StorefrontSettings'
import { SelfImproving } from './pages/SelfImproving'
import { Simulations } from './pages/Simulations'
import { TopicCategorization } from './pages/TopicCategorization'
import { CXQAScoring } from './pages/CXQAScoring'
import MonitoringDashboard from './pages/MonitoringDashboard'
import { SuggestionsDashboard } from './pages/SuggestionsDashboard'
import { CSATDashboard } from './pages/CSATDashboard'
import { FinOperatorDashboard } from './pages/FinOperatorDashboard'
import { Team } from './pages/Team'
import { AccessManagement } from './pages/AccessManagement'
import { Tickets } from './pages/Tickets'
import { Webhooks } from './pages/Webhooks'
import { WhatsAppIntegration } from './pages/WhatsAppIntegration'
import WhatsAppCampaigns from './pages/WhatsAppCampaigns'
import { LiveChatIntegration } from './pages/LiveChatIntegration'
import { SMSIntegration } from './pages/SMSIntegration'
import { FacebookIntegration } from './pages/FacebookIntegration'
import { InstagramIntegration } from './pages/InstagramIntegration'
import { TelegramIntegration } from './pages/TelegramIntegration'
import { LineIntegration } from './pages/LineIntegration'
import { VoiceIntegration } from './pages/VoiceIntegration'
import { EmailIntegration } from './pages/EmailIntegration'
import { TikTokIntegration } from './pages/TikTokIntegration'
import { DiscordIntegration } from './pages/DiscordIntegration'
import { SlackIntegration } from './pages/SlackIntegration'
import { AuditLogs } from './pages/AuditLogs'
import { CustomAttributes } from './pages/CustomAttributes'
import { SlaMonitoring } from './pages/SlaMonitoring'
import { RoutingCapacity } from './pages/RoutingCapacity'
import { MacrosManagement } from './pages/MacrosManagement'
import { CannedResponsesManagement } from './pages/CannedResponsesManagement'
import { ApprovalsQueue } from './pages/ApprovalsQueue'
import { OrganizationProfile } from './pages/OrganizationProfile'
import { OrganizationKeys } from './pages/OrganizationKeys'
import { OrganizationAiAgent } from './pages/OrganizationAiAgent'
import { BrandProfile } from './pages/BrandProfile'
import { ClientProfiles } from './pages/ClientProfiles'
import { BrandsList } from './pages/BrandsList'
import { ProceduresDashboard } from './pages/ProceduresDashboard'
import { UnifiedInbox } from './pages/UnifiedInbox'
import WorkflowBuilder from './pages/WorkflowBuilder'
import { HelpCenterDashboard } from './pages/HelpCenterDashboard'
import CRMSyncDashboard from './pages/CRMSyncDashboard'
import FinVisionDashboard from './pages/FinVisionDashboard'
import AudienceKBDashboard from './pages/AudienceKBDashboard'
import { BrandOverridesDashboard } from './pages/BrandOverridesDashboard'
import MCPDashboard from './pages/MCPDashboard'
import AutonomousResolutionDashboard from './pages/AutonomousResolutionDashboard'
import KBImportDashboard from './pages/KBImportDashboard'
import { AgentPerformanceDashboard } from './pages/AgentPerformanceDashboard'
import NotificationsDashboard from './pages/NotificationsDashboard'
import InAppMessagingDashboard from './pages/InAppMessagingDashboard'
import ToneOfVoiceDashboard from './pages/ToneOfVoiceDashboard'
import TranslationDashboard from './pages/TranslationDashboard'
import EmbedWidgetDashboard from './pages/EmbedWidgetDashboard'
import { UnifiedAnalyticsDashboard } from './pages/UnifiedAnalyticsDashboard'
import ReplyIntegrationDashboard from './pages/ReplyIntegrationDashboard'
import { ABTestingDashboard } from './pages/ABTestingDashboard'
import FulfillmentDashboard from './pages/FulfillmentDashboard'
import ScheduledTasksDashboard from './pages/ScheduledTasksDashboard'
import ComplianceDashboard from './pages/ComplianceDashboard'
import ConversationSearchDashboard from './pages/ConversationSearchDashboard'
import { Launchpad } from './pages/Launchpad'
import { PersonalProfile } from './pages/PersonalProfile'
import { WorkProfile } from './pages/WorkProfile'
import { SystemHealthDashboard } from './pages/SystemHealthDashboard'
import { EmailTemplatesDashboard } from './pages/EmailTemplatesDashboard'
import { BroadcastDashboard } from './pages/BroadcastDashboard'
import { TenantDashboard } from './pages/TenantDashboard'
import { OrganizationDashboard } from './pages/OrganizationDashboard'

function App() {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route path="/auth/nid/callback" element={<NidCallback />} />
      <Route path="/organizations" element={<Organizations />} />
      <Route
        element={
          <ProtectedRoute>
            <Layout />
          </ProtectedRoute>
        }
      >
        {/* Dashboard */}
        <Route path="/" element={<Dashboard />} />

        {/* Workspace — Organization + Brand hierarchy */}
        <Route path="/workspace/organization" element={<OrganizationProfile />} />
        <Route path="/workspace/organization/keys" element={<OrganizationKeys />} />
        <Route path="/workspace/brands" element={<BrandsList />} />
        <Route path="/workspace/brand/profile" element={<BrandProfile tab="profile" />} />
        <Route path="/workspace/brand/keys" element={<BrandProfile tab="keys" />} />
        <Route path="/workspace/brand/ai-config" element={<BrandProfile tab="ai-config" />} />
        <Route path="/workspace/brand/knowledge" element={<BrandProfile tab="knowledge" />} />

        {/* Inbox */}
        <Route path="/conversations" element={<Conversations />} />
        <Route path="/conversations/:id" element={<ConversationDetail />} />
        <Route path="/drafts" element={<Drafts />} />
        <Route path="/tickets" element={<Tickets />} />
        <Route path="/sla" element={<SlaMonitoring />} />
        <Route path="/approvals" element={<ApprovalsQueue />} />

        {/* Audience */}
        <Route path="/contacts" element={<Contacts />} />
        <Route path="/contacts/:id" element={<ContactProfile />} />
        <Route path="/clients" element={<ClientProfiles />} />
        <Route path="/clients/:id" element={<ContactProfile />} />
        <Route path="/broadcasts" element={<Broadcasts />} />
        <Route path="/sequences" element={<Sequences />} />
        <Route path="/automation" element={<Automation />} />

        {/* Channels */}
        <Route path="/channels" element={<ChannelsHub />} />
        <Route path="/channels/whatsapp" element={<WhatsAppIntegration />} />
        <Route path="/channels/whatsapp/campaigns" element={<WhatsAppCampaigns />} />
        <Route path="/channels/live-chat" element={<LiveChatIntegration />} />
        <Route path="/channels/sms" element={<SMSIntegration />} />
        <Route path="/channels/facebook" element={<FacebookIntegration />} />
        <Route path="/channels/instagram" element={<InstagramIntegration />} />
        <Route path="/channels/telegram" element={<TelegramIntegration />} />
        <Route path="/channels/line" element={<LineIntegration />} />
        <Route path="/channels/voice" element={<VoiceIntegration />} />
        <Route path="/channels/email" element={<EmailIntegration />} />
        <Route path="/channels/tiktok" element={<TikTokIntegration />} />
        <Route path="/channels/discord" element={<DiscordIntegration />} />
        <Route path="/channels/slack" element={<SlackIntegration />} />

        {/* Intelligence */}
        <Route path="/knowledge-base" element={<KnowledgeBase />} />
        <Route path="/services" element={<Services />} />
        <Route path="/ai-agent" element={<OrganizationAiAgent />} />
        <Route path="/simulations" element={<Simulations />} />

        {/* Commerce */}
        <Route path="/commerce" element={<CommerceDashboard />} />
        <Route path="/commerce/catalog" element={<CatalogManagement />} />
        <Route path="/commerce/catalog/:id" element={<ProductDetail />} />
        <Route path="/commerce/catalog/:id/options" element={<ProductOptions />} />
        <Route path="/commerce/carts" element={<CartManagement />} />
        <Route path="/commerce/orders" element={<OrdersList />} />
        <Route path="/commerce/orders/:id" element={<OrderDetail />} />
        <Route path="/commerce/sync" element={<SyncSettings />} />
        <Route path="/commerce/analytics" element={<AnalyticsDashboard />} />
        <Route path="/commerce/coupons" element={<CouponsManagement />} />
        <Route path="/commerce/shipping" element={<ShippingSettings />} />
        <Route path="/commerce/inventory" element={<InventoryManagement />} />
        <Route path="/commerce/storefront" element={<StorefrontSettings />} />
        <Route path="/self-improving" element={<SelfImproving />} />
        <Route path="/topic-categorization" element={<TopicCategorization />} />
        <Route path="/cx-qa" element={<CXQAScoring />} />
        <Route path="/monitoring" element={<MonitoringDashboard />} />
        <Route path="/suggestions" element={<SuggestionsDashboard />} />
        <Route path="/csat" element={<CSATDashboard />} />
        <Route path="/operator" element={<FinOperatorDashboard />} />
        <Route path="/procedures" element={<ProceduresDashboard />} />
        <Route path="/unified-inbox" element={<UnifiedInbox />} />
        <Route path="/workflows" element={<WorkflowBuilder />} />
        <Route path="/help-center" element={<HelpCenterDashboard />} />
        <Route path="/crm-sync" element={<CRMSyncDashboard />} />
        <Route path="/fin-vision" element={<FinVisionDashboard />} />
        <Route path="/audience-kb" element={<AudienceKBDashboard />} />
        <Route path="/brand-overrides" element={<BrandOverridesDashboard />} />
        <Route path="/mcp" element={<MCPDashboard />} />
        <Route path="/autonomous-resolution" element={<AutonomousResolutionDashboard />} />
        <Route path="/kb-import" element={<KBImportDashboard />} />
        <Route path="/agent-performance" element={<AgentPerformanceDashboard />} />
        <Route path="/notifications" element={<NotificationsDashboard />} />
        <Route path="/inapp-messaging" element={<InAppMessagingDashboard />} />
        <Route path="/tone-of-voice" element={<ToneOfVoiceDashboard />} />
        <Route path="/translation" element={<TranslationDashboard />} />
        <Route path="/embed-widget" element={<EmbedWidgetDashboard />} />
        <Route path="/unified-analytics" element={<UnifiedAnalyticsDashboard />} />
        <Route path="/reply-integration" element={<ReplyIntegrationDashboard />} />
        <Route path="/ab-testing" element={<ABTestingDashboard />} />
        <Route path="/fulfillment" element={<FulfillmentDashboard />} />
        <Route path="/scheduled-tasks" element={<ScheduledTasksDashboard />} />
        <Route path="/compliance" element={<ComplianceDashboard />} />
        <Route path="/conversation-search" element={<ConversationSearchDashboard />} />
        <Route path="/system-health" element={<SystemHealthDashboard />} />
        <Route path="/email-templates" element={<EmailTemplatesDashboard />} />
        <Route path="/broadcast" element={<BroadcastDashboard />} />

        {/* nID Sync — Launchpad & Profiles */}
        <Route path="/launchpad" element={<Launchpad />} />
        <Route path="/personal-profile/:nidAccountId" element={<PersonalProfile />} />
        <Route path="/work-profile/:orgId" element={<WorkProfile />} />
        <Route path="/tenants" element={<TenantDashboard />} />
        <Route path="/organizations" element={<OrganizationDashboard />} />

        {/* Reports */}
        <Route path="/reports" element={<Reports />} />
        <Route path="/reports/analytics" element={<Reports />} />

        {/* Settings */}
        <Route path="/settings/team" element={<Team />} />
        <Route path="/settings/access" element={<AccessManagement />} />
        <Route path="/settings/connections" element={<Connections />} />
        <Route path="/settings/webhooks" element={<Webhooks />} />
        <Route path="/settings/audit-logs" element={<AuditLogs />} />
        <Route path="/settings/custom-attributes" element={<CustomAttributes />} />
        <Route path="/settings/routing-capacity" element={<RoutingCapacity />} />
        <Route path="/settings/macros" element={<MacrosManagement />} />
        <Route path="/settings/canned-responses" element={<CannedResponsesManagement />} />

        {/* Backward compatibility — old routes redirect to new locations */}
        <Route path="/organization" element={<Navigate to="/workspace/organization" replace />} />
        <Route path="/organization/keys" element={<Navigate to="/workspace/organization/keys" replace />} />
        <Route path="/organization/ai-agent" element={<Navigate to="/ai-agent" replace />} />
        <Route path="/brand/profile" element={<Navigate to="/workspace/brand/profile" replace />} />
        <Route path="/brand/keys" element={<Navigate to="/workspace/brand/keys" replace />} />
        <Route path="/brand/ai-config" element={<Navigate to="/workspace/brand/ai-config" replace />} />
        <Route path="/brand/knowledge" element={<Navigate to="/workspace/brand/knowledge" replace />} />
        <Route path="/team" element={<Navigate to="/settings/team" replace />} />
        <Route path="/connections" element={<Navigate to="/settings/connections" replace />} />
        <Route path="/webhooks" element={<Navigate to="/settings/webhooks" replace />} />
        <Route path="/audit-logs" element={<Navigate to="/settings/audit-logs" replace />} />
        <Route path="/custom-attributes" element={<Navigate to="/settings/custom-attributes" replace />} />
        <Route path="/routing-capacity" element={<Navigate to="/settings/routing-capacity" replace />} />
        <Route path="/macros" element={<Navigate to="/settings/macros" replace />} />
        <Route path="/canned-responses" element={<Navigate to="/settings/canned-responses" replace />} />
        <Route path="/business-profile" element={<Navigate to="/workspace/brand/profile" replace />} />
      </Route>
    </Routes>
  )
}

export default App
