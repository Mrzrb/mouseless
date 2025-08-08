import React from 'react'
import { motion } from 'framer-motion'

const ActivationIndicator: React.FC = () => {
  return (
    <motion.div
      className="fixed top-4 right-4 z-50"
      initial={{ opacity: 0, scale: 0, x: 50 }}
      animate={{ opacity: 1, scale: 1, x: 0 }}
      exit={{ opacity: 0, scale: 0, x: 50 }}
      transition={{ 
        duration: 0.3,
        type: 'spring',
        stiffness: 200,
        damping: 15
      }}
    >
      <motion.div
        className="glass rounded-full p-3 flex items-center space-x-2"
        animate={{
          boxShadow: [
            '0 0 20px rgba(0, 122, 255, 0.3)',
            '0 0 30px rgba(0, 122, 255, 0.5)',
            '0 0 20px rgba(0, 122, 255, 0.3)'
          ]
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: 'easeInOut'
        }}
      >
        {/* Active indicator dot */}
        <motion.div
          className="w-3 h-3 bg-green-400 rounded-full"
          animate={{
            scale: [1, 1.2, 1],
            opacity: [0.8, 1, 0.8]
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: 'easeInOut'
          }}
        />
        
        {/* Status text */}
        <motion.span
          className="text-white text-sm font-medium"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.2 }}
        >
          Mouseless Active
        </motion.span>
      </motion.div>
    </motion.div>
  )
}

export default ActivationIndicator