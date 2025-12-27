import { cn } from '@/lib/utils'
import logoImage from '@/assets/logo.png'

interface LogoProps {
  variant?: 'full' | 'icon' | 'compact'
  className?: string
  showGlow?: boolean
}

/**
 * Reticle Logo
 * Custom brand logo
 */
export function Logo({ variant = 'full', className, showGlow = false }: LogoProps) {
  if (variant === 'icon') {
    return (
      <div className={cn('relative inline-flex', className)}>
        <img
          src={logoImage}
          alt="Reticle"
          className={cn(
            'w-8 h-8 object-contain',
            showGlow && 'glow-primary'
          )}
        />
      </div>
    )
  }

  if (variant === 'compact') {
    return (
      <div className={cn('inline-flex items-center gap-2.5', className)}>
        <img
          src={logoImage}
          alt="Reticle"
          className={cn(
            'w-7 h-7 object-contain',
            showGlow && 'glow-primary'
          )}
        />
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
        <img
          src={logoImage}
          alt="Reticle"
          className={cn(
            'w-9 h-9 object-contain',
            showGlow && 'glow-primary'
          )}
        />
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
