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
  keySequence?: string
}

const GridOverlay: React.FC<GridOverlayProps> = ({ gridData, keySequence = '' }) => {
  const { config, cells, animation } = gridData

  const generateCells = () => {
    return cells.map((cell, index) => {
      const cellId = `${cell.row}-${cell.column}`
      const label = config.show_labels ? cell.key_combination.toUpperCase() : ''
      
      // Check if this cell matches the current key sequence
      const isHighlighted = keySequence && cell.key_combination.startsWith(keySequence.toLowerCase())
      const isExactMatch = keySequence && cell.key_combination === keySequence.toLowerCase()
      
      // Determine cell styling based on state
      const getCellStyle = () => {
        const baseStyle = {
          left: cell.bounds.x,
          top: cell.bounds.y,
          width: cell.bounds.width,
          height: cell.bounds.height,
          position: 'absolute' as const,
          padding: `${config.cell_padding || 2}px`,
          borderWidth: `${config.border_width || 1}px`,
        }
        
        if (isExactMatch) {
          return {
            ...baseStyle,
            opacity: 1.0,
            background: 'linear-gradient(135deg, rgba(0, 255, 0, 0.3), rgba(0, 255, 0, 0.1))',
            borderColor: 'rgba(0, 255, 0, 0.8)',
            boxShadow: '0 0 20px rgba(0, 255, 0, 0.5), inset 0 0 20px rgba(0, 255, 0, 0.2)',
          }
        } else if (isHighlighted) {
          return {
            ...baseStyle,
            opacity: 0.9,
            background: 'linear-gradient(135deg, rgba(0, 122, 255, 0.2), rgba(0, 122, 255, 0.05))',
            borderColor: 'rgba(0, 122, 255, 0.6)',
            boxShadow: '0 0 15px rgba(0, 122, 255, 0.4)',
          }
        } else {
          return {
            ...baseStyle,
            opacity: keySequence ? 0.3 : (config.opacity || 0.8),
          }
        }
      }

      return (
        <motion.div
          key={cellId}
          className={`grid-cell relative ripple-effect ${isHighlighted ? 'highlighted' : ''} ${isExactMatch ? 'exact-match' : ''}`}
          style={getCellStyle()}
          initial={{
            opacity: 0,
            scale: 0.8,
            rotateX: -90
          }}
          animate={{
            opacity: getCellStyle().opacity,
            scale: isHighlighted ? 1.05 : 1,
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
            scale: isHighlighted ? 1.08 : 1.02,
            opacity: Math.min((getCellStyle().opacity || 0.8) + 0.2, 1.0),
            transition: { duration: 0.15 }
          }}
        >
          {config.show_labels && (
            <motion.div
              className={`grid-label ${isHighlighted ? 'highlighted-label' : ''} ${isExactMatch ? 'exact-match-label' : ''}`}
              style={{
                background: isExactMatch 
                  ? 'rgba(0, 255, 0, 0.4)' 
                  : isHighlighted 
                    ? 'rgba(0, 122, 255, 0.4)' 
                    : 'rgba(0, 122, 255, 0.2)',
                borderColor: isExactMatch 
                  ? 'rgba(0, 255, 0, 0.6)' 
                  : isHighlighted 
                    ? 'rgba(0, 122, 255, 0.6)' 
                    : 'rgba(0, 122, 255, 0.3)',
                fontSize: isHighlighted ? '16px' : '14px',
                fontWeight: isHighlighted ? 'bold' : 'normal',
              }}
              initial={{ opacity: 0, y: -10 }}
              animate={{ 
                opacity: 1, 
                y: 0,
                scale: isHighlighted ? 1.1 : 1
              }}
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
            className="absolute rounded-full"
            style={{
              left: '50%',
              top: '50%',
              transform: 'translate(-50%, -50%)',
              width: isHighlighted ? '12px' : '8px',
              height: isHighlighted ? '12px' : '8px',
              background: isExactMatch 
                ? 'rgba(0, 255, 0, 0.9)' 
                : isHighlighted 
                  ? 'rgba(0, 122, 255, 0.9)' 
                  : 'rgba(255, 255, 255, 0.6)',
              boxShadow: isHighlighted 
                ? `0 0 10px ${isExactMatch ? 'rgba(0, 255, 0, 0.8)' : 'rgba(0, 122, 255, 0.8)'}` 
                : 'none',
            }}
            initial={{ scale: 0 }}
            animate={{ 
              scale: 1,
              opacity: isHighlighted ? 1 : 0.6
            }}
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