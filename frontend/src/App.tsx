import { Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Connections } from './pages/Connections'
import { ConversationDetail } from './pages/ConversationDetail'
import { Conversations } from './pages/Conversations'
import { Dashboard } from './pages/Dashboard'
import { Drafts } from './pages/Drafts'
import { Sequences } from './pages/Sequences'
import { Services } from './pages/Services'

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/drafts" element={<Drafts />} />
        <Route path="/conversations" element={<Conversations />} />
        <Route path="/conversations/:id" element={<ConversationDetail />} />
        <Route path="/services" element={<Services />} />
        <Route path="/sequences" element={<Sequences />} />
        <Route path="/connections" element={<Connections />} />
      </Route>
    </Routes>
  )
}

export default App
