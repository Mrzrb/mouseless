import React, { useEffect, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'

interface KeyFeedbackProps {
  sequence: string
  timestamp: number
}

const KeyFeedback: React.FC<KeyFeedbackProps> = ({ sequence, timestamp }) => {
  const [isVisible, setIsVisible] = useState(true)

  useEffect(() => {
    // Auto-hide after 2 seconds
    const timer = setTimeout(() => {
      setIsVisible(false)
    }, 2000)

    return () => clearTimeout(timer)
  }, [timestamp])

  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          className="fixed top-8 left-1/2 transform -translate-x-1/2 z-50"
          initial={{ opacity: 0, y: -20, scale: 0.8 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: -20, scale: 0.8 }}
          transition={{
            duration: 0.3,
            ease: 'easeOut',
            type: 'spring',
            stiffness: 200
          }}
        >
          <div className="glass px-6 py-3 rounded-lg shadow-lg">
            <div className="flex items-center space-x-2">
              <div className="text-sm text-gray-300 font-medium">
                Key Sequence:
              </div>
              <div className="text-lg font-bold text-white tracking-wider">
                {sequence.toUpperCase()}
              </div>
              {sequence.endsWith('_') && (
                <motion.div
                  className="w-2 h-6 bg-blue-400 rounded-sm"
                  animate={{ opacity: [1, 0, 1] }}
                  transition={{
                    duration: 1,
                    repeat: Infinity,
                    ease: 'easeInOut'
                  }}
                />
              )}
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

export default KeyFeedback