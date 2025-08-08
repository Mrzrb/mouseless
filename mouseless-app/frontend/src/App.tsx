import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import GridOverlay from './components/GridOverlay'
import AreaOverlay from './components/AreaOverlay'
import PredictionOverlay from './components/PredictionOverlay'
import PermissionCheck from './components/PermissionCheck'
import { GridConfig, PredictionTarget } from './types'

function App() {
  const [overlayType, setOverlayType] = useState<'none' | 'grid' | 'area' | 'prediction'>('none')
  const [gridConfig, setGridConfig] = useState<GridConfig | null>(null)
  const [predictionTargets, setPredictionTargets] = useState<PredictionTarget[]>([])
  const [hasPermissions, setHasPermissions] = useState<boolean>(true)

  useEffect(() => {
    // Check accessibility permissions on startup
    checkPermissions()

    // Listen for overlay configuration events from Rust backend
    const unlistenGrid = listen('configure-grid', (event) => {
      const config = event.payload as GridConfig
      setGridConfig(config)
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
      setGridConfig(null)
      setPredictionTargets([])
    })

    // Cleanup listeners
    return () => {
      unlistenGrid.then(fn => fn())
      unlistenArea.then(fn => fn())
      unlistenPrediction.then(fn => fn())
      unlistenHide.then(fn => fn())
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
      {overlayType === 'grid' && gridConfig && (
        <GridOverlay config={gridConfig} />
      )}
      
      {overlayType === 'area' && (
        <AreaOverlay />
      )}
      
      {overlayType === 'prediction' && predictionTargets.length > 0 && (
        <PredictionOverlay targets={predictionTargets} />
      )}
    </div>
  )
}

export default App