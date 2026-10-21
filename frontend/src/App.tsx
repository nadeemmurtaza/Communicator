import { Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Automation } from './pages/Automation'
import { Broadcasts } from './pages/Broadcasts'
import { ChannelsHub } from './pages/ChannelsHub'
import { Connections } from './pages/Connections'
import { Contacts } from './pages/Contacts'
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

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/reports" element={<Reports />} />
        <Route path="/drafts" element={<Drafts />} />
        <Route path="/conversations" element={<Conversations />} />
        <Route path="/conversations/:id" element={<ConversationDetail />} />
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
        <Route path="/contacts" element={<Contacts />} />
        <Route path="/broadcasts" element={<Broadcasts />} />
        <Route path="/automation" element={<Automation />} />
        <Route path="/knowledge-base" element={<KnowledgeBase />} />
        <Route path="/services" element={<Services />} />
        <Route path="/sequences" element={<Sequences />} />
        <Route path="/tickets" element={<Tickets />} />
        <Route path="/simulations" element={<Simulations />} />
        <Route path="/self-improving" element={<SelfImproving />} />
        <Route path="/team" element={<Team />} />
        <Route path="/connections" element={<Connections />} />
      </Route>
    </Routes>
  )
}

export default App
