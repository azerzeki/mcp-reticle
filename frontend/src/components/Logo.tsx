import { Activity } from 'lucide-react'
import { cn } from '@/lib/utils'

interface LogoProps {
  variant?: 'full' | 'icon' | 'compact'
  className?: string
  showGlow?: boolean
}

/**
 * Reticle Logo
 * Modern minimalist design with activity monitoring icon
 */
export function Logo({ variant = 'full', className, showGlow = false }: LogoProps) {
  if (variant === 'icon') {
    return (
      <div className={cn('relative inline-flex', className)}>
        <div
          className={cn(
            'relative flex items-center justify-center w-8 h-8 rounded-lg',
            showGlow && 'glow-primary'
          )}
          style={{ background: 'linear-gradient(135deg, #00F0FF 0%, #00B8C4 100%)' }}
        >
          <Activity className="w-4 h-4 text-[#0D1117]" strokeWidth={2} />
        </div>
      </div>
    )
  }

  if (variant === 'compact') {
    return (
      <div className={cn('inline-flex items-center gap-2.5', className)}>
        <div
          className={cn(
            'relative flex items-center justify-center w-7 h-7 rounded-lg',
            showGlow && 'glow-primary'
          )}
          style={{ background: 'linear-gradient(135deg, #00F0FF 0%, #00B8C4 100%)' }}
        >
          <Activity className="w-3.5 h-3.5 text-[#0D1117]" strokeWidth={2} />
        </div>
        <span className="text-sm font-bold text-foreground tracking-tight">
          Reticle
        </span>
      </div>
    )
  }

  // Full logo with tagline
  return (
    <div className={cn('inline-flex flex-col gap-1', className)}>
      <div className="flex items-center gap-2.5">
        <div
          className={cn(
            'relative flex items-center justify-center w-9 h-9 rounded-lg',
            showGlow && 'glow-primary'
          )}
          style={{ background: 'linear-gradient(135deg, #00F0FF 0%, #00B8C4 100%)' }}
        >
          <Activity className="w-5 h-5 text-[#0D1117]" strokeWidth={2} />
        </div>
        <div>
          <h1 className="text-lg font-bold text-foreground tracking-tight leading-none">
            Reticle
          </h1>
          <p className="text-[10px] text-muted-foreground font-medium tracking-wide uppercase mt-0.5">
            Protocol Inspector
          </p>
        </div>
      </div>
    </div>
  )
}
