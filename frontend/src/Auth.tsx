import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import './App.css'

export default function Auth() {
  const navigate = useNavigate()

  useEffect(() => {
    const hasValidCookie = document.cookie
      .split(';')
      .map((c) => c.trim())
      .includes('canvasAuth=valid')

    if (hasValidCookie) {
      navigate('/home')
      return
    }
  }, [navigate])

  return (
    <div/>
  )
}