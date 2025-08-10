import React from 'react'
import { motion } from 'framer-motion'
import { PredictionTarget } from '../types'

interface PredictionOverlayProps {
  targets: PredictionTarget[]
}

const PredictionOverlay: React.FC<PredictionOverlayProps> = ({ targets }) => {
  //TODO: Implement keyboard shortcuts for prediction target selection
  //TODO: Add numbered/lettered labels for quick selection
  //TODO: Implement confidence-based animations (pulsing, breathing effects)
  //TODO: Add target type-specific styling (button, link, text field, etc.)
  //TODO: Implement target filtering based on user preferences
  
  const getConfidenceClass = (confidence: number) => {
    if (confidence >= 0.8) return 'prediction-high'
    if (confidence >= 0.5) return 'prediction-medium'
    return 'prediction-low'
  }

  const getTargetSize = (confidence: number) => {
    // Size based on confidence: higher confidence = larger target
    const baseSize = 20
    const maxSize = 40
    return baseSize + (confidence * (maxSize - baseSize))
  }

  return (
    <motion.div
      className="absolute inset-0 pointer-events-auto"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      {targets.map((target, index) => {
        const size = getTargetSize(target.confidence)
        
        return (
          <motion.div
            key={index}
            className={`prediction-target breathing ${getConfidenceClass(target.confidence)}`}
            style={{
              left: target.position.x,
              top: target.position.y,
              width: size,
              height: size,
            }}
            initial={{ 
              opacity: 0, 
              scale: 0,
              rotate: -180
            }}
            animate={{ 
              opacity: 1, 
              scale: 1,
              rotate: 0
            }}
            transition={{
              duration: 0.4,
              delay: index * 0.05,
              type: 'spring',
              stiffness: 200,
              damping: 15
            }}
            whileHover={{
              scale: 1.3,
              transition: { duration: 0.15 }
            }}
          >
            {/* Confidence indicator */}
            <motion.div
              className="absolute inset-0 rounded-full border-2 border-white"
              animate={{
                scale: [1, 1.2, 1],
                opacity: [0.6, 1, 0.6]
              }}
              transition={{
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut'
              }}
            />
            
            {/* Target label */}
            {target.description && (
              <motion.div
                className="absolute -top-8 left-1/2 transform -translate-x-1/2 
                           text-xs font-bold text-white bg-black bg-opacity-60 
                           px-2 py-1 rounded whitespace-nowrap"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.03 + 0.2 }}
              >
                {target.description}
              </motion.div>
            )}
            
            {/* Confidence percentage */}
            <motion.div
              className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2
                         text-xs font-bold text-white"
              style={{ textShadow: '0 0 4px rgba(0, 0, 0, 0.8)' }}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: index * 0.03 + 0.3 }}
            >
              {Math.round(target.confidence * 100)}%
            </motion.div>
          </motion.div>
        )
      })}
    </motion.div>
  )
}

export default PredictionOverlay