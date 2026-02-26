import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import './App.css'
import Sidebar from './Sidebar'
import LabPage from './LabPage'

export default function App() {
  return (
    <BrowserRouter>
      <div className="app-container">
        <Sidebar />
        <div className="main-content">
          <Routes>
            <Route path="/" element={<Navigate to="/lab1" replace />} />
            <Route path="/lab1" element={<LabPage key="lab1" />} />
            <Route path="/lab2" element={<LabPage key="lab2" />} />
            <Route path="/lab3" element={<LabPage key="lab3" />} />
            <Route path="/lab4" element={<LabPage key="lab4" />} />
            <Route path="/lab5" element={<LabPage key="lab5" />} />
            <Route path="*" element={<Navigate to="/lab1" replace />} />
          </Routes>
        </div>
      </div>
    </BrowserRouter>
  )
}
