import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
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
    
    // Listen for overlay configuration events from Rust backend
    const unlistenGrid = listen('configure-grid', (event) => {
      console.log('OverlayApp: Received configure-grid event', event.payload)
      const data = event.payload as GridData
      setGridData(data)
      setOverlayType('grid')
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
    }
  }, [])

  // Keyboard event handling for grid mode
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // console.log('OverlayApp: Key pressed:', event.key)
      
      // if (overlayType !== 'grid' || !gridData) {
      //   console.log('OverlayApp: Not in grid mode or no grid data')
      //   return
      // }

      const key = event.key.toLowerCase()
      const now = Date.now()

      // Exit keys
      if (key === ' ' || key === 'escape') {
        console.log('OverlayApp: Exit key pressed')
        // We can't directly call hideAllOverlays here, so we'll emit an event
        // For now, just log
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
          
          // Find the grid cell for this combination
          const cell = gridData.cells.find(c => c.key_combination === combination)
          if (cell) {
            console.log('OverlayApp: Moving to position:', cell.center_position)
            // Here we would normally move the mouse, but for now just log
            // In a real implementation, this would call a Tauri command to move the mouse
          } else {
            console.log('OverlayApp: No cell found for combination:', combination.toUpperCase())
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
      style={{ width: '100vw', height: '100vh' }}
      tabIndex={0}
      onKeyDown={(e) => console.log('React onKeyDown:', e.key)}
      onClick={() => {
        console.log('Overlay clicked, focusing...')
        const overlay = document.querySelector('.overlay-window') as HTMLElement
        overlay?.focus()
      }}
    >
      {/* Debug info - always visible */}
      <div style={{ 
        position: 'fixed', 
        top: '10px', 
        left: '10px', 
        background: 'rgba(255,0,0,0.8)', 
        color: 'white', 
        padding: '10px',
        borderRadius: '4px',
        fontSize: '14px',
        zIndex: 9999,
        border: '2px solid yellow'
      }}>
        <div>🎯 OVERLAY WINDOW ACTIVE</div>
        <div>Overlay Type: {overlayType}</div>
        <div>Grid Data: {gridData ? 'Available' : 'None'}</div>
        <div>Cells: {gridData?.cells?.length || 0}</div>
        {keySequence && <div>Key Sequence: {keySequence.toUpperCase()}_</div>}
        <div>Time: {new Date().toLocaleTimeString()}</div>
      </div>
      
      {/* Test grid - always show a simple grid for testing */}
      <div style={{
        position: 'fixed',
        top: '100px',
        left: '100px',
        width: '300px',
        height: '300px',
        background: 'rgba(0,255,0,0.3)',
        border: '2px solid red',
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gridTemplateRows: 'repeat(3, 1fr)',
        gap: '2px'
      }}>
        {Array.from({length: 9}, (_, i) => (
          <div key={i} style={{
            background: 'rgba(255,255,255,0.5)',
            border: '1px solid black',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '20px',
            fontWeight: 'bold'
          }}>
            {String.fromCharCode(65 + Math.floor(i/3)) + String.fromCharCode(81 + (i%3))}
          </div>
        ))}
      </div>

      {/* Focus test button */}
      <button
        style={{
          position: 'fixed',
          top: '420px',
          left: '100px',
          padding: '10px 20px',
          background: 'rgba(255,255,0,0.8)',
          border: '2px solid black',
          borderRadius: '4px',
          fontSize: '16px',
          fontWeight: 'bold',
          cursor: 'pointer'
        }}
        onClick={() => {
          console.log('Focus button clicked!')
          const overlay = document.querySelector('.overlay-window') as HTMLElement
          if (overlay) {
            overlay.focus()
            console.log('Overlay focused')
          }
        }}
      >
        点击获取焦点
      </button>

      {overlayType === 'grid' && gridData && (
        <GridOverlay gridData={gridData} />
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
    </div>
  )
}

export default OverlayApp