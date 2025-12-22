# MCP-Sentinel Frontend

A high-performance desktop debugger for the Model Context Protocol (MCP) built with React, TypeScript, and Tauri v2.

## 🎨 Tech Stack

- **Framework**: React 18 + Vite + TypeScript
- **UI Library**: Shadcn UI (Radix Primitives + Tailwind CSS)
- **Icons**: Lucide React
- **State Management**: Zustand
- **Virtualization**: react-virtuoso (for 10k+ log rows)
- **Code Editor**: Monaco Editor (@monaco-editor/react)
- **Charts**: Recharts
- **Desktop**: Tauri v2
- **Layout**: react-resizable-panels

## 📁 Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── ui/                    # Shadcn UI components
│   │   │   ├── button.tsx
│   │   │   └── scroll-area.tsx
│   │   ├── LogStream.tsx          # Virtualized message list
│   │   ├── Inspector.tsx          # JSON viewer with Monaco
│   │   └── Sidebar.tsx            # Metrics & session management
│   ├── store/
│   │   └── index.ts               # Zustand global store
│   ├── types/
│   │   └── index.ts               # TypeScript definitions
│   ├── lib/
│   │   └── utils.ts               # Helper functions
│   ├── styles/
│   │   └── globals.css            # Tailwind + custom styles
│   ├── App.tsx                    # Main layout + Tauri integration
│   └── main.tsx                   # React entry point
├── index.html
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── package.json
```

## 🚀 Getting Started

### Prerequisites

- Node.js 18+ and npm/pnpm/yarn
- Rust (for Tauri)

### Installation

```bash
cd frontend
npm install
```

### Development

```bash
# Run Vite dev server (for web development)
npm run dev

# Run with Tauri (desktop app)
npm run tauri dev
```

### Build

```bash
# Build for production
npm run build

# Build Tauri app
npm run tauri build
```

## 🎯 Features

### LogStream Component
- **Virtualization**: Handles 10,000+ logs without performance degradation
- **Auto-Scroll**: Sticks to bottom on new logs, pauses when user scrolls up
- **Smart Filtering**: By method, direction, session, or search text
- **Compact Design**: Timestamp, direction icon, method, summary, duration
- **Color Coding**:
  - 🔵 Blue: Requests
  - 🟢 Green: Responses
  - 🔴 Red: Errors

### Inspector Component
- **Monaco Editor**: Full-featured JSON editor with syntax highlighting
- **Read-Only Mode**: Prevents accidental edits
- **Metadata Panel**: Shows timestamp, direction, session ID, duration
- **Copy Button**: One-click copy to clipboard
- **Code Folding**: Collapse/expand JSON structures

### Sidebar Component
- **Live Metrics**:
  - Total message count
  - Messages per second (1s window)
  - Activity chart (10s history)
- **Filters**:
  - Direction (Incoming/Outgoing)
  - Method name
- **Session Management**: Time-travel through historical sessions
- **Actions**: Clear logs button

## 🎨 Theme & Styling

The app uses a **cyberpunk-inspired dark theme** with:
- **Base**: Zinc color palette (Shadcn default)
- **Accents**:
  - Neon Blue (`#00d4ff`)
  - Neon Cyan (`#00fff2`)
  - Neon Pink (`#ff007a`)
  - Neon Purple (`#b400ff`)
- **Font**: Geist Mono (monospace)
- **Glow Effects**: Subtle neon glows on interactive elements

## 🔌 Tauri Integration

The app listens for events from the Rust backend:

### Events

#### `log-event`
Payload:
```typescript
{
  id: string
  session_id: string
  timestamp: number  // microseconds
  direction: "in" | "out"
  content: string    // Raw JSON-RPC
  method?: string
  duration_micros?: number
}
```

#### `session-start`
Payload:
```typescript
{
  id: string
  started_at: number
}
```

## 🧪 Performance Optimizations

1. **Virtualization**: Only renders visible log rows (~20-30 items)
2. **Memoization**: `React.memo()` on LogRow to prevent unnecessary re-renders
3. **Computed Selectors**: Zustand getters for filtered logs
4. **Circular Buffer**: Keeps max 10,000 logs in memory (FIFO)
5. **Incremental Layout**: Monaco editor auto-adjusts to panel size

## 📊 State Management

The app uses **Zustand** for global state:

```typescript
interface SentinelStore {
  isConnected: boolean
  logs: LogEntry[]
  selectedLogId: string | null
  sessions: Session[]
  filters: FilterOptions

  // Actions
  addLog(log: LogEntry): void
  selectLog(id: string | null): void
  setFilters(filters: Partial<FilterOptions>): void
  clearLogs(): void

  // Computed
  getFilteredLogs(): LogEntry[]
  getSelectedLog(): LogEntry | null
}
```

## 🎨 Customization

### Change Theme

Edit `src/styles/globals.css`:

```css
:root {
  --background: 240 10% 3.9%;    /* Background color */
  --foreground: 0 0% 98%;        /* Text color */
  --primary: 0 0% 98%;           /* Primary accent */
  /* ... */
}
```

### Add New Filters

1. Update `FilterOptions` type in `src/types/index.ts`
2. Add filter logic in `getFilteredLogs()` in `src/store/index.ts`
3. Add UI controls in `src/components/Sidebar.tsx`

## 📝 License

MIT
