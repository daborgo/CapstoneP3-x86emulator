import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import './index.css'
import App from './App.tsx'
import Login from './Login.tsx'
import Auth from './Auth.tsx'
import Home from './Home.tsx'
import Sidebar from './Sidebar.tsx'
import LabPage from './LabPage.tsx'
import InstructorSubmissions from './InstructorSubmissions.tsx'

function LabLayout() {
  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        <Outlet />
      </div>
    </div>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/login" replace />} />
        <Route path="/login" element={<Login />} />
        <Route path="/auth" element={<Auth />} />
        <Route path="/home" element={<Home />} />
        <Route path="/emulator" element={<App />} />
        <Route path="/submissions" element={<InstructorSubmissions />} />
        <Route element={<LabLayout />}>
          <Route path="/lab1" element={<LabPage key="lab1" />} />
          <Route path="/lab2" element={<LabPage key="lab2" />} />
          <Route path="/lab3" element={<LabPage key="lab3" />} />
          <Route path="/lab4" element={<LabPage key="lab4" />} />
          <Route path="/lab5" element={<LabPage key="lab5" />} />
        </Route>
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
