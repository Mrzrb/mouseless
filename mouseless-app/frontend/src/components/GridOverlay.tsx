import React from 'react'
import { motion } from 'framer-motion'
import { GridConfig } from '../types'

interface GridCell {
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
}

interface GridData {
  config: GridConfig
  cells: GridCell[]
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

interface GridOverlayProps {
  gridData: GridData
}

const GridOverlay: React.FC<GridOverlayProps> = ({ gridData }) => {
  const { config, cells, animation } = gridData

  const generateCells = () => {
    return cells.map((cell, index) => {
      const cellId = `${cell.row}-${cell.column}`
      const label = config.show_labels ? cell.key_combination.toUpperCase() : ''

      return (
        <motion.div
          key={cellId}
          className="grid-cell relative ripple-effect"
          style={{
            left: cell.bounds.x,
            top: cell.bounds.y,
            width: cell.bounds.width,
            height: cell.bounds.height,
            position: 'absolute',
            opacity: config.opacity || 0.8,
            padding: `${config.cell_padding || 2}px`,
            borderWidth: `${config.border_width || 1}px`,
          }}
          initial={{
            opacity: 0,
            scale: 0.8,
            rotateX: -90
          }}
          animate={{
            opacity: config.opacity || 0.8,
            scale: 1,
            rotateX: 0
          }}
          transition={{
            duration: (animation?.duration || 400) / 1000,
            delay: index * 0.03,
            ease: animation?.easing || 'easeOut',
            type: 'spring',
            stiffness: 100
          }}
          whileHover={{
            scale: 1.02,
            opacity: Math.min((config.opacity || 0.8) + 0.2, 1.0),
            transition: { duration: 0.15 }
          }}
        >
          {config.show_labels && (
            <motion.div
              className="grid-label"
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{
                duration: 0.3,
                delay: index * 0.03 + 0.2
              }}
            >
              {label}
            </motion.div>
          )}

          {/* Center point indicator */}
          <motion.div
            className="absolute w-2 h-2 bg-white rounded-full opacity-60"
            style={{
              left: '50%',
              top: '50%',
              transform: 'translate(-50%, -50%)',
            }}
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{
              duration: 0.2,
              delay: index * 0.03 + 0.4
            }}
          />
        </motion.div>
      )
    })
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