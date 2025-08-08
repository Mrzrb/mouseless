import React, { useEffect, useState } from 'react'
import { motion } from 'framer-motion'

const AreaOverlay: React.FC = () => {
  const [screenDimensions, setScreenDimensions] = useState({ width: 1920, height: 1080 })

  useEffect(() => {
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

  const regionWidth = screenDimensions.width / 3
  const regionHeight = screenDimensions.height / 3

  const generateRegions = () => {
    const regions = []
    const labels = ['1', '2', '3', '4', '5', '6', '7', '8', '9']
    
    for (let row = 0; row < 3; row++) {
      for (let col = 0; col < 3; col++) {
        const regionId = row * 3 + col
        const label = labels[regionId]

        regions.push(
          <motion.div
            key={regionId}
            className="area-region relative flex items-center justify-center"
            style={{
              left: col * regionWidth,
              top: row * regionHeight,
              width: regionWidth,
              height: regionHeight,
              position: 'absolute',
            }}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{
              duration: 0.4,
              delay: regionId * 0.05,
              ease: 'easeOut'
            }}
            whileHover={{
              backgroundColor: 'rgba(255, 255, 255, 0.15)',
              scale: 1.02,
              transition: { duration: 0.2 }
            }}
          >
            <motion.div
              className="area-number"
              initial={{ scale: 0, rotate: -180 }}
              animate={{ scale: 1, rotate: 0 }}
              transition={{
                duration: 0.5,
                delay: regionId * 0.05 + 0.2,
                type: 'spring',
                stiffness: 200,
                damping: 15
              }}
              whileHover={{
                scale: 1.1,
                transition: { duration: 0.2 }
              }}
            >
              {label}
            </motion.div>
          </motion.div>
        )
      }
    }
    
    return regions
  }

  return (
    <motion.div
      className="absolute inset-0 pointer-events-auto"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.3 }}
    >
      {generateRegions()}
    </motion.div>
  )
}

export default AreaOverlay