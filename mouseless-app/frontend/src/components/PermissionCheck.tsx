import React from 'react'
import { motion } from 'framer-motion'
import { Shield, Settings, RefreshCw } from 'lucide-react'

interface PermissionCheckProps {
  onRequestPermissions: () => void
  onRecheck: () => void
}

const PermissionCheck: React.FC<PermissionCheckProps> = ({ 
  onRequestPermissions, 
  onRecheck 
}) => {
  return (
    <motion.div
      className="fixed inset-0 bg-black bg-opacity-80 flex items-center justify-center"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
    >
      <motion.div
        className="bg-white rounded-lg p-8 max-w-md mx-4 text-center"
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.3, delay: 0.1 }}
      >
        <motion.div
          className="mb-6"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          transition={{ duration: 0.3, delay: 0.2, type: 'spring' }}
        >
          <Shield className="w-16 h-16 mx-auto text-red-500" />
        </motion.div>
        
        <motion.h2
          className="text-2xl font-bold text-gray-800 mb-4"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.3 }}
        >
          Accessibility Permissions Required
        </motion.h2>
        
        <motion.p
          className="text-gray-600 mb-6 leading-relaxed"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.4 }}
        >
          Mouseless needs accessibility permissions to control your mouse cursor. 
          This allows the app to move the cursor and perform clicks based on your keyboard input.
        </motion.p>
        
        <motion.div
          className="space-y-3"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.5 }}
        >
          <button
            onClick={onRequestPermissions}
            className="w-full bg-blue-500 hover:bg-blue-600 text-white font-semibold 
                       py-3 px-6 rounded-lg transition-colors duration-200 
                       flex items-center justify-center space-x-2"
          >
            <Settings className="w-5 h-5" />
            <span>Open System Preferences</span>
          </button>
          
          <button
            onClick={onRecheck}
            className="w-full bg-gray-200 hover:bg-gray-300 text-gray-800 font-semibold 
                       py-3 px-6 rounded-lg transition-colors duration-200 
                       flex items-center justify-center space-x-2"
          >
            <RefreshCw className="w-5 h-5" />
            <span>Check Again</span>
          </button>
        </motion.div>
        
        <motion.div
          className="mt-6 text-sm text-gray-500"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.3, delay: 0.6 }}
        >
          <p>
            After granting permissions in System Preferences, click "Check Again" to continue.
          </p>
        </motion.div>
      </motion.div>
    </motion.div>
  )
}

export default PermissionCheck