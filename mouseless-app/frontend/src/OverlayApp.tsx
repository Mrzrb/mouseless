import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import GridOverlay from './components/GridOverlay'
import AreaOverlay from './components/AreaOverlay'
import PredictionOverlay from './components/PredictionOverlay'
import ActivationIndicator from './components/ActivationIndicator'
import KeyFeedback from './components/KeyFeedback'
import { GridConfig, PredictionTarget } from './types'

interface GridData {
  config: GridConfig
  cells: Array<{
    row: number
    column: number
    bounds: {
      x: number
      y: number
      width: number
      height: number
    }
    key_combination: string
    center_position: {
      x: number
      y: number
    }
  }>
  screen_bounds: {
    width: number
    height: number
  }
  animation?: {
    type: 'appear' | 'disappear'
    duration: number
    easing: string
  }
}

interface AreaData {
  areas: Array<{
    key: string
    bounds: {
      x: number
      y: number
      width: number
      height: number
    }
    center: {
      x: number
      y: number
    }
    label: string
  }>
  screen_bounds: {
    width: number
    height: number
  }
}

function OverlayApp() {
  const [overlayType, setOverlayType] = useState<'none' | 'grid' | 'area' | 'prediction'>('none')
  const [gridData, setGridData] = useState<GridData | null>(null)
  const [areaData, setAreaData] = useState<AreaData | null>(null)
  const [predictionTargets, setPredictionTargets] = useState<PredictionTarget[]>([])
  const [showActivationIndicator, setShowActivationIndicator] = useState<boolean>(false)
  const [keyFeedback, setKeyFeedback] = useState<{ sequence: string; timestamp: number } | null>(null)
  const [keySequence, setKeySequence] = useState<string>('')
  const [lastKeyTime, setLastKeyTime] = useState<number>(0)
  const [highlightedArea, setHighlightedArea] = useState<string | null>(null)

  useEffect(() => {
    console.log('OverlayApp: Setting up event listeners')

    // Try to focus the overlay window when it mounts
    setTimeout(() => {
      const overlay = document.querySelector('.overlay-window') as HTMLElement
      if (overlay) {
        overlay.focus()
        console.log('OverlayApp: Focused overlay window on mount')
      }
    }, 100)

    // Listen for overlay configuration events from Rust backend
    const unlistenGrid = listen('configure-grid', (event) => {
      console.log('🎉 OverlayApp: Received configure-grid event!', event.payload)
      const data = event.payload as GridData
      setGridData(data)
      setOverlayType('grid')
      console.log('✅ OverlayApp: Grid data set, overlayType set to grid, cells:', data.cells?.length)
    })

    const unlistenArea = listen('configure-area', (event) => {
      console.log('OverlayApp: Received configure-area event', event.payload)
      const data = event.payload as AreaData
      setAreaData(data)
      setOverlayType('area')
    })

    const unlistenPrediction = listen('configure-prediction', (event) => {
      const targets = event.payload as PredictionTarget[]
      setPredictionTargets(targets)
      setOverlayType('prediction')
    })

    const unlistenHide = listen('hide-overlays', () => {
      setOverlayType('none')
      setGridData(null)
      setAreaData(null)
      setPredictionTargets([])
    })

    const unlistenActivation = listen('show-activation-indicator', () => {
      setShowActivationIndicator(true)
    })

    const unlistenDeactivation = listen('hide-activation-indicator', () => {
      setShowActivationIndicator(false)
    })

    const unlistenKeyFeedback = listen('show-key-feedback', (event) => {
      const data = event.payload as { sequence: string; timestamp: number }
      setKeyFeedback(data)
    })

    const unlistenGridDisappear = listen('animate-grid-disappear', () => {
      // Trigger grid disappear animation
      setOverlayType('none')
      setGridData(null)
    })

    const unlistenHighlightArea = listen('highlight-area', (event) => {
      console.log('OverlayApp: Received highlight-area event', event.payload)
      const data = event.payload as { highlightedArea: string | null; timestamp: number }
      setHighlightedArea(data.highlightedArea)
    })

    // Test event listener to see if overlay can receive any events
    const unlistenTest = listen('test-event', (event) => {
      console.log('🧪 OverlayApp: Received test event!', event.payload)
    })

    // Cleanup listeners
    return () => {
      unlistenGrid.then(fn => fn())
      unlistenArea.then(fn => fn())
      unlistenPrediction.then(fn => fn())
      unlistenHide.then(fn => fn())
      unlistenActivation.then(fn => fn())
      unlistenDeactivation.then(fn => fn())
      unlistenKeyFeedback.then(fn => fn())
      unlistenGridDisappear.then(fn => fn())
      unlistenHighlightArea.then(fn => fn())
      unlistenTest.then(fn => fn())
    }
  }, [])

  // Keyboard event handling for grid mode
  useEffect(() => {
    const handleKeyDown = async (event: KeyboardEvent) => {
      console.log('OverlayApp: Key pressed:', event.key, 'overlayType:', overlayType, 'hasGridData:', !!gridData)

      if (overlayType !== 'grid' || !gridData) {
        console.log('OverlayApp: Not in grid mode or no grid data, overlayType:', overlayType, 'gridData:', !!gridData)
        return
      }

      const key = event.key.toLowerCase()
      const now = Date.now()

      // Exit keys
      if (key === ' ' || key === 'escape') {
        console.log('OverlayApp: Exit key pressed, hiding overlays')
        try {
          await invoke('hide_all_overlays')
          console.log('OverlayApp: Overlays hidden successfully')
        } catch (error) {
          console.error('OverlayApp: Failed to hide overlays:', error)
        }
        return
      }

      // Valid grid keys
      const firstKeys = ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']
      const secondKeys = ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p']

      // Reset sequence if timeout (1 second)
      if (now - lastKeyTime > 1000) {
        setKeySequence('')
      }

      if (keySequence === '') {
        // First key
        if (firstKeys.includes(key)) {
          setKeySequence(key)
          setLastKeyTime(now)
          console.log('OverlayApp: First key:', key.toUpperCase())
        }
      } else {
        // Second key
        if (secondKeys.includes(key)) {
          const combination = keySequence + key
          console.log('OverlayApp: Key combination:', combination.toUpperCase())

          try {
            // Call Tauri command to move mouse to grid cell
            await invoke('move_mouse_to_grid_cell', {
              key_combination: combination
            })
            console.log('OverlayApp: Mouse moved successfully to grid cell:', combination.toUpperCase())
          } catch (error) {
            console.error('OverlayApp: Failed to move mouse to grid cell:', error)
          }

          // Reset sequence
          setKeySequence('')
          setLastKeyTime(0)
        } else {
          // Invalid second key, reset
          console.log('OverlayApp: Invalid second key, resetting')
          setKeySequence('')
          setLastKeyTime(0)
        }
      }
    }

    console.log('OverlayApp: Adding keyboard event listener')
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      console.log('OverlayApp: Removing keyboard event listener')
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [overlayType, gridData, keySequence, lastKeyTime])

  console.log('OverlayApp: Rendering with overlayType:', overlayType, 'gridData:', gridData)

  return (
    <div
      className="overlay-window"
      style={{
        width: '100vw',
        height: '100vh',
        background: 'transparent',
        pointerEvents: overlayType === 'grid' ? 'auto' : 'none'
      }}
      tabIndex={0}
      onClick={() => {
        console.log('Overlay clicked, focusing...')
        const overlay = document.querySelector('.overlay-window') as HTMLElement
        overlay?.focus()
      }}
      onKeyDown={(e) => {
        console.log('React onKeyDown:', e.key)
        e.preventDefault()
      }}
    >
      {overlayType === 'grid' && gridData && (
        <GridOverlay gridData={gridData} keySequence={keySequence} />
      )}

      {overlayType === 'area' && (
        <AreaOverlay
          areas={areaData?.areas}
          highlightedArea={highlightedArea || undefined}
        />
      )}

      {overlayType === 'prediction' && predictionTargets.length > 0 && (
        <PredictionOverlay targets={predictionTargets} />
      )}

      {showActivationIndicator && (
        <ActivationIndicator />
      )}

      {keyFeedback && (
        <KeyFeedback
          sequence={keyFeedback.sequence}
          timestamp={keyFeedback.timestamp}
        />
      )}

      {/* Key sequence indicator - only show when typing */}
      {keySequence && (
        <div style={{
          position: 'fixed',
          top: '20px',
          right: '20px',
          background: 'rgba(0, 0, 0, 0.8)',
          color: 'white',
          padding: '8px 16px',
          borderRadius: '8px',
          fontSize: '18px',
          fontWeight: 'bold',
          fontFamily: 'monospace',
          zIndex: 10000,
          pointerEvents: 'none'
        }}>
          {keySequence.toUpperCase()}_
        </div>
      )}
    </div>
  )
}

export default OverlayApp