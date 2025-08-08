import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import GridOverlay from './components/GridOverlay'
import AreaOverlay from './components/AreaOverlay'
import PredictionOverlay from './components/PredictionOverlay'
import PermissionCheck from './components/PermissionCheck'
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
}

function App() {
  const [overlayType, setOverlayType] = useState<'none' | 'grid' | 'area' | 'prediction'>('none')
  const [gridData, setGridData] = useState<GridData | null>(null)
  const [predictionTargets, setPredictionTargets] = useState<PredictionTarget[]>([])
  const [hasPermissions, setHasPermissions] = useState<boolean>(true)
  const [showActivationIndicator, setShowActivationIndicator] = useState<boolean>(false)
  const [keyFeedback, setKeyFeedback] = useState<{ sequence: string; timestamp: number } | null>(null)

  useEffect(() => {
    // Check accessibility permissions on startup
    checkPermissions()

    // Listen for overlay configuration events from Rust backend
    const unlistenGrid = listen('configure-grid', (event) => {
      const data = event.payload as GridData
      setGridData(data)
      setOverlayType('grid')
    })

    const unlistenArea = listen('configure-area', () => {
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
    }
  }, [])

  const checkPermissions = async () => {
    try {
      const hasPerms = await invoke<boolean>('check_accessibility_permissions')
      setHasPermissions(hasPerms)
    } catch (error) {
      console.error('Failed to check permissions:', error)
      setHasPermissions(false)
    }
  }

  const requestPermissions = async () => {
    try {
      await invoke('request_accessibility_permissions')
      // Recheck permissions after a delay
      setTimeout(checkPermissions, 1000)
    } catch (error) {
      console.error('Failed to request permissions:', error)
    }
  }

  // Show permission check if permissions are not granted
  if (!hasPermissions) {
    return (
      <PermissionCheck 
        onRequestPermissions={requestPermissions}
        onRecheck={checkPermissions}
      />
    )
  }

  return (
    <div className="overlay-window">
      {overlayType === 'grid' && gridData && (
        <GridOverlay gridData={gridData} />
      )}
      
      {overlayType === 'area' && (
        <AreaOverlay />
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

export default App