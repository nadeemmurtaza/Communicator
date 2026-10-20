import { Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Automation } from './pages/Automation'
import { Broadcasts } from './pages/Broadcasts'
import { Connections } from './pages/Connections'
import { Contacts } from './pages/Contacts'
import { ConversationDetail } from './pages/ConversationDetail'
import { Conversations } from './pages/Conversations'
import { Dashboard } from './pages/Dashboard'
import { Drafts } from './pages/Drafts'
import { KnowledgeBase } from './pages/KnowledgeBase'
import { Sequences } from './pages/Sequences'
import { Services } from './pages/Services'
import { Team } from './pages/Team'

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/drafts" element={<Drafts />} />
        <Route path="/conversations" element={<Conversations />} />
        <Route path="/conversations/:id" element={<ConversationDetail />} />
        <Route path="/contacts" element={<Contacts />} />
        <Route path="/broadcasts" element={<Broadcasts />} />
        <Route path="/automation" element={<Automation />} />
        <Route path="/knowledge-base" element={<KnowledgeBase />} />
        <Route path="/services" element={<Services />} />
        <Route path="/sequences" element={<Sequences />} />
        <Route path="/team" element={<Team />} />
        <Route path="/connections" element={<Connections />} />
      </Route>
    </Routes>
  )
}

export default App
