import React from 'react'

interface Area {
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
}

interface AreaOverlayProps {
  areas?: Area[]
  showLabels?: boolean
  opacity?: number
  highlightedArea?: string
  combinationKeys?: string[]
}

const AreaOverlay: React.FC<AreaOverlayProps> = ({ 
  areas = [], 
  showLabels = true, 
  opacity = 0.8,
  highlightedArea,
  combinationKeys = []
}) => {
  // Default 9 areas if none provided (for testing)
  const defaultAreas: Area[] = [
    // Top row
    { key: 'q', bounds: { x: 0, y: 0, width: 640, height: 360 }, center: { x: 320, y: 180 }, label: 'Q' },
    { key: 'w', bounds: { x: 640, y: 0, width: 640, height: 360 }, center: { x: 960, y: 180 }, label: 'W' },
    { key: 'e', bounds: { x: 1280, y: 0, width: 640, height: 360 }, center: { x: 1600, y: 180 }, label: 'E' },
    // Middle row
    { key: 'a', bounds: { x: 0, y: 360, width: 640, height: 360 }, center: { x: 320, y: 540 }, label: 'A' },
    { key: 's', bounds: { x: 640, y: 360, width: 640, height: 360 }, center: { x: 960, y: 540 }, label: 'S' },
    { key: 'd', bounds: { x: 1280, y: 360, width: 640, height: 360 }, center: { x: 1600, y: 540 }, label: 'D' },
    // Bottom row
    { key: 'z', bounds: { x: 0, y: 720, width: 640, height: 360 }, center: { x: 320, y: 900 }, label: 'Z' },
    { key: 'x', bounds: { x: 640, y: 720, width: 640, height: 360 }, center: { x: 960, y: 900 }, label: 'X' },
    { key: 'c', bounds: { x: 1280, y: 720, width: 640, height: 360 }, center: { x: 1600, y: 900 }, label: 'C' },
  ]

  const displayAreas = areas.length > 0 ? areas : defaultAreas

  return (
    <div className="area-overlay">
      {displayAreas.map((area) => {
        const isHighlighted = highlightedArea === area.key.toLowerCase()
        const isInCombination = combinationKeys.includes(area.key.toLowerCase())
        const isActive = isHighlighted || isInCombination
        
        return (
          <div
            key={area.key}
            className="area-cell"
            style={{
              position: 'absolute',
              left: `${area.bounds.x}px`,
              top: `${area.bounds.y}px`,
              width: `${area.bounds.width}px`,
              height: `${area.bounds.height}px`,
              border: isActive 
                ? '3px solid rgba(255, 215, 0, 0.9)' 
                : '2px solid rgba(0, 150, 255, 0.8)',
              backgroundColor: isActive 
                ? `rgba(255, 215, 0, ${opacity * 0.2})` 
                : `rgba(0, 150, 255, ${opacity * 0.1})`,
              backdropFilter: 'blur(10px)',
              borderRadius: '8px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s ease',
              pointerEvents: 'none',
              zIndex: isActive ? 1001 : 1000,
              transform: isActive ? 'scale(1.05)' : 'scale(1)',
              boxShadow: isActive 
                ? '0 0 20px rgba(255, 215, 0, 0.5)' 
                : '0 0 10px rgba(0, 150, 255, 0.3)',
            }}
          >
          {showLabels && (
            <div
              className="area-label"
              style={{
                fontSize: isActive ? '56px' : '48px',
                fontWeight: 'bold',
                color: isActive 
                  ? 'rgba(255, 255, 255, 1.0)' 
                  : 'rgba(255, 255, 255, 0.9)',
                textShadow: isActive 
                  ? '3px 3px 6px rgba(0, 0, 0, 0.7)' 
                  : '2px 2px 4px rgba(0, 0, 0, 0.5)',
                fontFamily: 'system-ui, -apple-system, sans-serif',
                userSelect: 'none',
                transition: 'all 0.2s ease',
              }}
            >
              {area.label}
            </div>
          )}
          
          {/* Center point indicator */}
          <div
            style={{
              position: 'absolute',
              left: '50%',
              top: '50%',
              transform: 'translate(-50%, -50%)',
              width: isActive ? '12px' : '8px',
              height: isActive ? '12px' : '8px',
              backgroundColor: isActive 
                ? 'rgba(255, 215, 0, 0.9)' 
                : 'rgba(255, 255, 255, 0.8)',
              borderRadius: '50%',
              boxShadow: isActive 
                ? '0 0 15px rgba(255, 215, 0, 0.7)' 
                : '0 0 10px rgba(255, 255, 255, 0.5)',
              transition: 'all 0.2s ease',
            }}
          />
        </div>
        )
      })}
      
      {/* Area mode indicator */}
      <div
        style={{
          position: 'fixed',
          top: '20px',
          right: '20px',
          padding: '12px 20px',
          backgroundColor: 'rgba(0, 150, 255, 0.9)',
          color: 'white',
          borderRadius: '8px',
          fontSize: '16px',
          fontWeight: 'bold',
          fontFamily: 'system-ui, -apple-system, sans-serif',
          backdropFilter: 'blur(10px)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
          zIndex: 1001,
          userSelect: 'none',
        }}
      >
        🎯 Area Mode Active
      </div>
      
      {/* Instructions */}
      <div
        style={{
          position: 'fixed',
          bottom: '20px',
          left: '50%',
          transform: 'translateX(-50%)',
          padding: '16px 24px',
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          color: 'white',
          borderRadius: '12px',
          fontSize: '14px',
          fontFamily: 'system-ui, -apple-system, sans-serif',
          backdropFilter: 'blur(10px)',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.5)',
          zIndex: 1001,
          userSelect: 'none',
          textAlign: 'center',
        }}
      >
        <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>
          Press area keys to navigate: Q W E / A S D / Z X C
        </div>
        <div style={{ fontSize: '12px', opacity: 0.8 }}>
          Press Space or Esc to exit • Combine keys for intersections (e.g., Q+E)
        </div>
        {highlightedArea && (
          <div style={{ 
            marginTop: '8px', 
            fontSize: '12px', 
            color: 'rgba(255, 215, 0, 0.9)',
            fontWeight: 'bold'
          }}>
            Area {highlightedArea.toUpperCase()} selected - press another key for combination
          </div>
        )}
      </div>
    </div>
  )
}

export default AreaOverlay