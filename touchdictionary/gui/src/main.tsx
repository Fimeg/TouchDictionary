import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import './styles/index.css'

const domRoot = document.getElementById('root')
const root = createRoot(domRoot!)
root.render(
  <StrictMode>
    <App />
  </StrictMode>
)
