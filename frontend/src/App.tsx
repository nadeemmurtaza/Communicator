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
import { SelfImproving } from './pages/SelfImproving'
import { Simulations } from './pages/Simulations'
import { Team } from './pages/Team'
import { Tickets } from './pages/Tickets'
import { Webhooks } from './pages/Webhooks'
import { WhatsAppIntegration } from './pages/WhatsAppIntegration'
import { LiveChatIntegration } from './pages/LiveChatIntegration'
import { SMSIntegration } from './pages/SMSIntegration'
import { FacebookIntegration } from './pages/FacebookIntegration'
import { InstagramIntegration } from './pages/InstagramIntegration'
import { TelegramIntegration } from './pages/TelegramIntegration'
import { LineIntegration } from './pages/LineIntegration'
import { VoiceIntegration } from './pages/VoiceIntegration'
import { EmailIntegration } from './pages/EmailIntegration'
import { TikTokIntegration } from './pages/TikTokIntegration'
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
        <Route path="/channels/live-chat" element={<LiveChatIntegration />} />
        <Route path="/channels/sms" element={<SMSIntegration />} />
        <Route path="/channels/facebook" element={<FacebookIntegration />} />
        <Route path="/channels/instagram" element={<InstagramIntegration />} />
        <Route path="/channels/telegram" element={<TelegramIntegration />} />
        <Route path="/channels/line" element={<LineIntegration />} />
        <Route path="/channels/voice" element={<VoiceIntegration />} />
        <Route path="/channels/email" element={<EmailIntegration />} />
        <Route path="/channels/tiktok" element={<TikTokIntegration />} />

        {/* Intelligence */}
        <Route path="/knowledge-base" element={<KnowledgeBase />} />
        <Route path="/services" element={<Services />} />
        <Route path="/ai-agent" element={<OrganizationAiAgent />} />
        <Route path="/simulations" element={<Simulations />} />
        <Route path="/self-improving" element={<SelfImproving />} />

        {/* Reports */}
        <Route path="/reports" element={<Reports />} />
        <Route path="/reports/analytics" element={<Reports />} />

        {/* Settings */}
        <Route path="/settings/team" element={<Team />} />
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
