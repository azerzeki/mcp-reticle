import { useState, useEffect } from 'react'
import { Circle, StopCircle, Save, Trash2, Download, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface RecordingStatus {
  is_recording: boolean
  session_id: string | null
  message_count: number
  duration_seconds: number
}

interface SessionListItem {
  id: string
  name: string
  started_at: number
  ended_at: number | null
  message_count: number
  duration_ms: number | null
  transport: string
}

export function RecordingControls() {
  const [isRecording, setIsRecording] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [status, setStatus] = useState<RecordingStatus | null>(null)
  const [sessions, setSessions] = useState<SessionListItem[]>([])
  const [showSessions, setShowSessions] = useState(false)

  // Load recording status on mount
  useEffect(() => {
    checkStatus()
    loadSessions()
  }, [])

  // Listen for recording events
  useEffect(() => {
    let unsubscribeStarted: (() => void) | null = null
    let unsubscribeStopped: (() => void) | null = null

    const setupListeners = async () => {
      unsubscribeStarted = await listen<{ session_id: string; session_name: string }>(
        'recording-started',
        (event) => {
          console.log('Recording started:', event.payload)
          setIsRecording(true)
          toast.success('Recording started', {
            description: `Session: ${event.payload.session_name}`,
            duration: 3000,
          })
          checkStatus()
        }
      )

      unsubscribeStopped = await listen<{ session_id: string; message_count: number }>(
        'recording-stopped',
        (event) => {
          console.log('Recording stopped:', event.payload)
          setIsRecording(false)
          toast.success('Recording saved', {
            description: `Captured ${event.payload.message_count} messages`,
            duration: 3000,
          })
          loadSessions()
        }
      )
    }

    setupListeners()

    return () => {
      if (unsubscribeStarted) unsubscribeStarted()
      if (unsubscribeStopped) unsubscribeStopped()
    }
  }, [])

  const checkStatus = async () => {
    try {
      const result = await invoke<RecordingStatus>('get_recording_status')
      setStatus(result)
      setIsRecording(result.is_recording)
    } catch (error) {
      console.error('Failed to get recording status:', error)
    }
  }

  const loadSessions = async () => {
    try {
      const result = await invoke<SessionListItem[]>('list_recorded_sessions')
      setSessions(result)
    } catch (error) {
      console.error('Failed to load sessions:', error)
    }
  }

  const startRecording = async () => {
    setIsLoading(true)
    try {
      const sessionName = `Session ${new Date().toLocaleString()}`
      await invoke('start_recording', { sessionName })
      setIsRecording(true)
      checkStatus()
    } catch (error) {
      console.error('Failed to start recording:', error)
      toast.error('Failed to start recording', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
        duration: 4000,
      })
    } finally {
      setIsLoading(false)
    }
  }

  const stopRecording = async () => {
    setIsLoading(true)
    try {
      await invoke('stop_recording')
      setIsRecording(false)
      await loadSessions()
    } catch (error) {
      console.error('Failed to stop recording:', error)
      toast.error('Failed to stop recording', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
        duration: 4000,
      })
    } finally {
      setIsLoading(false)
    }
  }

  const deleteSession = async (sessionId: string) => {
    try {
      await invoke('delete_recorded_session', { sessionId })
      toast.success('Session deleted', { duration: 2000 })
      await loadSessions()
    } catch (error) {
      console.error('Failed to delete session:', error)
      toast.error('Failed to delete session', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
      })
    }
  }

  const exportSession = async (sessionId: string) => {
    try {
      // In a real app, we'd open a file dialog. For now, use a default path
      const exportPath = `/tmp/mcp-session-${sessionId}.json`
      await invoke('export_session', { sessionId, exportPath })
      toast.success('Session exported', {
        description: `Saved to ${exportPath}`,
        duration: 3000,
      })
    } catch (error) {
      console.error('Failed to export session:', error)
      toast.error('Failed to export session', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
      })
    }
  }

  const formatDuration = (ms: number | null) => {
    if (!ms) return '0s'
    const seconds = Math.floor(ms / 1000)
    if (seconds < 60) return `${seconds}s`
    const minutes = Math.floor(seconds / 60)
    return `${minutes}m ${seconds % 60}s`
  }

  return (
    <div className="glass-strong border-b border-white/5">
      <div className="flex items-center gap-2 px-4 py-2.5">
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-semibold text-zinc-500 uppercase tracking-wider">
            Recording
          </span>

          {!isRecording && (
            <Button
              variant="default"
              size="sm"
              onClick={startRecording}
              disabled={isLoading}
              className="h-9 px-4 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/50 text-xs font-medium"
            >
              {isLoading ? (
                <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" />
              ) : (
                <Circle className="w-3.5 h-3.5 mr-2" />
              )}
              {isLoading ? 'Starting...' : 'Start Recording'}
            </Button>
          )}

          {isRecording && status && (
            <>
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-red-500/10 border border-red-500/30">
                <span className="animate-pulse w-1.5 h-1.5 bg-red-400 rounded-full glow-error" />
                <span className="text-[11px] font-medium text-red-400">
                  Recording
                </span>
                <span className="text-[11px] text-zinc-400">
                  {status.message_count} msgs • {status.duration_seconds}s
                </span>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={stopRecording}
                disabled={isLoading}
                className="h-9 px-4 border-white/10 hover:bg-zinc-800 text-xs text-zinc-300"
              >
                {isLoading ? (
                  <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" />
                ) : (
                  <StopCircle className="w-3.5 h-3.5 mr-2" />
                )}
                {isLoading ? 'Stopping...' : 'Stop & Save'}
              </Button>
            </>
          )}

          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowSessions(!showSessions)}
            className="h-9 px-3 hover:bg-zinc-800 text-zinc-400 text-xs"
          >
            <Save className="w-3.5 h-3.5 mr-2" />
            Sessions ({sessions.length})
          </Button>
        </div>

        <div className="flex-1" />
      </div>

      {showSessions && sessions.length > 0 && (
        <div className="px-4 pb-3 pt-2 border-t border-white/5 bg-zinc-950/50">
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {sessions.map((session) => (
              <div
                key={session.id}
                className="flex items-center gap-3 p-2 rounded-md bg-zinc-900/50 border border-white/5 hover:border-white/10 transition-colors"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-xs font-medium text-zinc-300 truncate">
                    {session.name}
                  </p>
                  <p className="text-xs text-zinc-500">
                    {session.message_count} messages • {formatDuration(session.duration_ms)} • {session.transport}
                  </p>
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => exportSession(session.id)}
                    className="h-7 px-2 hover:bg-zinc-800 text-zinc-400"
                    title="Export session"
                  >
                    <Download className="w-3.5 h-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => deleteSession(session.id)}
                    className="h-7 px-2 hover:bg-zinc-800 text-red-400 hover:text-red-300"
                    title="Delete session"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {showSessions && sessions.length === 0 && (
        <div className="px-4 pb-3 pt-2 border-t border-white/5 bg-zinc-950/50">
          <p className="text-xs text-zinc-400 text-center py-3">
            No recorded sessions yet. Start recording to capture MCP messages.
          </p>
        </div>
      )}

      {!isRecording && !showSessions && (
        <div className="px-4 pb-3 pt-2 border-t border-white/5 bg-zinc-950/50">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 w-1 h-12 bg-red-500/30 rounded-full" />
            <div className="flex-1">
              <p className="text-xs text-zinc-400 leading-relaxed">
                <span className="font-medium text-zinc-300">Session Recording:</span> Capture all MCP message traffic for later analysis and replay.
                Click <span className="font-mono text-red-400">Start Recording</span> to begin capturing messages.
              </p>
              <p className="text-xs text-zinc-500 mt-1.5">
                Recordings are saved to your local database and can be exported as JSON files.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
