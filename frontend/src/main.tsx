import React from 'react'
import ReactDOM from 'react-dom/client'
import '@fontsource/geist-sans'
import '@fontsource/jetbrains-mono'
import App from './App'
import { ThemeProvider } from '@/components/theme-provider'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="reticle-theme">
      <App />
    </ThemeProvider>
  </React.StrictMode>
)
