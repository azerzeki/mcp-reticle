import { useState } from 'react'
import { Play, Pause, RotateCcw, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { useReticleStore } from '@/store'
import { invoke } from '@tauri-apps/api/core'
import { ThemeToggle } from '@/components/ThemeToggle'

export function DemoControls() {
  const [isRunning, setIsRunning] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const { clearLogs } = useReticleStore()

  const startDemo = async () => {
    setIsLoading(true)
    try {
      await invoke('start_proxy', {
        command: 'demo',
        args: [],
      })
      setIsRunning(true)
      toast.success('Demo started successfully', {
        duration: 2000,
      })
    } catch (error) {
      console.error('Failed to start demo:', error)
      toast.error('Failed to start demo', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
        duration: 4000,
      })
    } finally {
      setIsLoading(false)
    }
  }

  const stopDemo = async () => {
    setIsLoading(true)
    try {
      await invoke('stop_proxy')
      setIsRunning(false)
      toast.success('Demo stopped', {
        duration: 2000,
      })
    } catch (error) {
      console.error('Failed to stop demo:', error)
      toast.error('Failed to stop demo', {
        description: error instanceof Error ? error.message : 'Unknown error occurred',
        duration: 4000,
      })
    } finally {
      setIsLoading(false)
    }
  }

  const resetDemo = () => {
    clearLogs()
  }

  return (
    <div className="glass-strong flex items-center gap-2 px-4 py-2.5 border-b border-white/5">
      <div className="flex items-center gap-2">
        <span className="text-[11px] font-semibold text-zinc-500 uppercase tracking-wider">
          Demo Mode
        </span>
        {!isRunning ? (
          <Button
            variant="default"
            size="sm"
            onClick={startDemo}
            disabled={isLoading}
            className="h-9 px-4 bg-indigo-500/20 hover:bg-indigo-500/30 text-indigo-400 border border-indigo-500/50 text-xs font-medium"
          >
            {isLoading ? (
              <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" />
            ) : (
              <Play className="w-3.5 h-3.5 mr-2" />
            )}
            {isLoading ? 'Starting...' : 'Start Demo'}
          </Button>
        ) : (
          <Button
            variant="outline"
            size="sm"
            onClick={stopDemo}
            disabled={isLoading}
            className="h-9 px-4 border-white/10 hover:bg-zinc-800 text-xs text-zinc-300"
          >
            {isLoading ? (
              <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" />
            ) : (
              <Pause className="w-3.5 h-3.5 mr-2" />
            )}
            {isLoading ? 'Stopping...' : 'Stop Demo'}
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          onClick={resetDemo}
          className="h-9 px-3 hover:bg-zinc-800 text-zinc-400 text-xs"
          title="Clear all logs"
        >
          <RotateCcw className="w-3.5 h-3.5 mr-2" />
          Reset
        </Button>
      </div>
      <div className="flex-1" />
      <div className="flex items-center gap-4">
        <div className="text-[11px] text-zinc-400">
          {isRunning && (
            <span className="flex items-center gap-2">
              <span className="animate-pulse w-1.5 h-1.5 bg-indigo-400 rounded-full glow-primary" />
              Loading demo data...
            </span>
          )}
        </div>
        <ThemeToggle />
      </div>
    </div>
  )
}
