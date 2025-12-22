/**
 * Core types for Reticle frontend
 */

export type Direction = 'in' | 'out'

/** Type of message content */
export type MessageType = 'jsonrpc' | 'raw' | 'stderr'

export interface LogEntry {
  id: string
  session_id: string
  timestamp: number // microseconds since epoch
  direction: Direction
  content: string // Raw JSON-RPC message or raw text
  method?: string // Extracted method name for quick filtering
  duration_micros?: number // For responses, time since request
  message_type?: MessageType // Type of content (jsonrpc, raw, stderr)
}

export interface ParsedMessage {
  jsonrpc: string
  id?: string | number
  method?: string
  params?: unknown
  result?: unknown
  error?: {
    code: number
    message: string
    data?: unknown
  }
}

export interface Session {
  id: string
  started_at: number
  message_count: number
  last_activity: number
}

export interface MetricsData {
  timestamp: number
  messages_per_second: number
  avg_latency_micros: number
}

export interface FilterOptions {
  method?: string
  direction?: Direction
  searchText?: string
  sessionId?: string
}
