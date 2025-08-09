import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import OverlayApp from './OverlayApp'
import './index.css'

// Determine which app to render based on the current window
const isOverlayWindow = window.location.pathname.includes('overlay.html')

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isOverlayWindow ? <OverlayApp /> : <App />}
  </React.StrictMode>,
)