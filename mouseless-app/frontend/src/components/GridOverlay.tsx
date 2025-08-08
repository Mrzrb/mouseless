import React, { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { GridConfig } from '../types'

interface GridOverlayProps {
  config: GridConfig
}

const GridOverlay: React.FC<GridOverlayProps> = ({ config }) => {
  const [screenDimensions, setScreenDimensions] = useState({ width: 1920, height: 1080 })

  useEffect(() => {
    // Get actual screen dimensions
    const updateDimensions = () => {
      setScreenDimensions({
        width: window.screen.width,
        height: window.screen.height
      })
    }

    updateDimensions()
    window.addEventListener('resize', updateDimensions)
    
    return () => window.removeEventListener('resize', updateDimensions)
  }, [])

  const cellWidth = screenDimensions.width / config.columns
  const cellHeight = screenDimensions.height / config.rows

  const generateCells = () => {
    const cells = []
    
    for (let row = 0; row < config.rows; row++) {
      for (let col = 0; col < config.columns; col++) {
        const cellId = `${row}-${col}`
        const label = config.show_labels ? 
          String.fromCharCode(65 + row) + (col + 1) : // A1, A2, B1, B2, etc.
          ''

        cells.push(
          <motion.div
            key={cellId}
            className="grid-cell relative"
            style={{
              left: col * cellWidth,
              top: row * cellHeight,
              width: cellWidth,
              height: cellHeight,
              position: 'absolute',
            }}
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{
              duration: 0.3,
              delay: (row * config.columns + col) * 0.02,
              ease: 'easeOut'
            }}
            whileHover={{
              backgroundColor: 'rgba(0, 122, 255, 0.2)',
              scale: 1.02,
              transition: { duration: 0.1 }
            }}
          >
            {config.show_labels && (
              <div className="grid-label">
                {label}
              </div>
            )}
          </motion.div>
        )
      }
    }
    
    return cells
  }

  return (
    <motion.div
      className="absolute inset-0 pointer-events-auto"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      {generateCells()}
    </motion.div>
  )
}

export default GridOverlay