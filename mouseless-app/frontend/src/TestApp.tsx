import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { GridConfig } from './types'

// GridOverlay 现在在 OverlayApp 中使用

interface GridData {
  config: GridConfig
  cells: Array<{
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
  }>
  screen_bounds: {
    width: number
    height: number
  }
}

function TestApp() {
  const [overlayType, setOverlayType] = useState<'none' | 'grid' | 'area' | 'prediction'>('none')
  const [gridData, setGridData] = useState<GridData | null>(null)
  const [hasPermissions, setHasPermissions] = useState<boolean>(true)
  const [keySequence, setKeySequence] = useState<string>('')
  const [lastKeyTime, setLastKeyTime] = useState<number>(0)

  useEffect(() => {
    // Check accessibility permissions on startup
    checkPermissions()

    // Listen for overlay configuration events from Rust backend
    const unlistenGrid = listen('configure-grid', (event) => {
      console.log('🎉 收到 configure-grid 事件:', event.payload)
      const data = event.payload as GridData
      setGridData(data)
      setOverlayType('grid')
      console.log('✅ 网格模式已激活, 网格数据:', data)
    })

    const unlistenArea = listen('configure-area', () => {
      setOverlayType('area')
    })

    const unlistenPrediction = listen('configure-prediction', () => {
      setOverlayType('prediction')
    })

    const unlistenHide = listen('hide-overlays', () => {
      setOverlayType('none')
      setGridData(null)
    })

    const unlistenActivation = listen('show-activation-indicator', () => {
      console.log('Activation indicator shown')
    })

    const unlistenDeactivation = listen('hide-activation-indicator', () => {
      console.log('Activation indicator hidden')
    })

    const unlistenKeyFeedback = listen('show-key-feedback', (event) => {
      console.log('Key feedback:', event.payload)
    })

    const unlistenGridDisappear = listen('animate-grid-disappear', () => {
      // Trigger grid disappear animation
      setOverlayType('none')
      setGridData(null)
    })

    // Cleanup listeners
    return () => {
      unlistenGrid.then(fn => fn())
      unlistenArea.then(fn => fn())
      unlistenPrediction.then(fn => fn())
      unlistenHide.then(fn => fn())
      unlistenActivation.then(fn => fn())
      unlistenDeactivation.then(fn => fn())
      unlistenKeyFeedback.then(fn => fn())
      unlistenGridDisappear.then(fn => fn())
    }
  }, [])

  // Separate useEffect for keyboard handling
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (overlayType !== 'grid' || !gridData) return

      const key = event.key.toLowerCase()
      const now = Date.now()

      // Exit keys
      if (key === ' ' || key === 'escape') {
        hideAllOverlays()
        return
      }

      // Valid grid keys
      const firstKeys = ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']
      const secondKeys = ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p']

      // Reset sequence if timeout (1 second)
      if (now - lastKeyTime > 1000) {
        setKeySequence('')
      }

      if (keySequence === '') {
        // First key
        if (firstKeys.includes(key)) {
          setKeySequence(key)
          setLastKeyTime(now)
          console.log('First key:', key.toUpperCase())
        }
      } else {
        // Second key
        if (secondKeys.includes(key)) {
          const combination = keySequence + key
          console.log('Key combination:', combination.toUpperCase())

          // Find the grid cell for this combination
          const cell = gridData.cells.find(c => c.key_combination === combination)
          if (cell) {
            console.log('Moving to position:', cell.center_position)
            // Here we would normally move the mouse, but for now just log
            // In a real implementation, this would call a Tauri command to move the mouse
          } else {
            console.log('No cell found for combination:', combination.toUpperCase())
          }

          // Reset sequence
          setKeySequence('')
          setLastKeyTime(0)
        } else {
          // Invalid second key, reset
          setKeySequence('')
          setLastKeyTime(0)
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [overlayType, gridData, keySequence, lastKeyTime])

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

  // Simplified permission check
  if (!hasPermissions) {
    return (
      <div style={{ padding: '20px', background: '#f0f0f0', color: '#333' }}>
        <h2>需要辅助功能权限</h2>
        <button onClick={requestPermissions} style={{ padding: '10px 20px', margin: '10px' }}>
          打开系统偏好设置
        </button>
        <button onClick={checkPermissions} style={{ padding: '10px 20px', margin: '10px' }}>
          重新检查
        </button>
      </div>
    )
  }

  // Test functions for grid mode
  const showDefaultGrid = async () => {
    console.log('🎯 点击了显示默认网格按钮')
    try {
      console.log('📞 正在调用 show_grid_overlay 命令...')
      const result = await invoke('show_grid_overlay', {
        rows: 3,
        columns: 3,
        showLabels: true,
        opacity: 0.8,
        cellPadding: 2,
        borderWidth: 1
      })
      console.log('✅ show_grid_overlay 命令执行成功:', result)
    } catch (error) {
      console.error('❌ Failed to show grid:', error)
      alert('显示网格失败: ' + error)
    }
  }

  const showLargeGrid = async () => {
    try {
      await invoke('show_grid_overlay', {
        rows: 4,
        columns: 5,
        showLabels: true,
        opacity: 0.8,
        cellPadding: 2,
        borderWidth: 1
      })
    } catch (error) {
      console.error('Failed to show large grid:', error)
    }
  }

  const hideAllOverlays = async () => {
    try {
      await invoke('hide_all_overlays')
    } catch (error) {
      console.error('Failed to hide overlays:', error)
    }
  }

  const testConnection = async () => {
    console.log('🧪 测试 Tauri 连接...')
    try {
      const result = await invoke<string>('test_show_grid')
      console.log('✅ 测试成功:', result)
      alert('Tauri 连接正常: ' + result)
    } catch (error) {
      console.error('❌ 测试失败:', error)
      alert('Tauri 连接失败: ' + error)
    }
  }

  // Test functions for area mode
  const showAreaOverlay = async () => {
    console.log('🎯 点击了显示区域覆盖层按钮')
    try {
      console.log('📞 正在调用 show_area_overlay 命令...')
      const result = await invoke('show_area_overlay')
      console.log('✅ show_area_overlay 命令执行成功:', result)
    } catch (error) {
      console.error('❌ Failed to show area overlay:', error)
      alert('显示区域覆盖层失败: ' + error)
    }
  }

  return (
    <div style={{
      height: '100vh',
      width: '100vw',
      background: '#1a1a1a',
      color: 'white',
      padding: '20px',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      {/* Control Panel */}
      <div style={{ marginBottom: '20px' }}>
        <h1 style={{ textAlign: 'center', marginBottom: '20px' }}>🎯 Mouseless 网格模式测试</h1>
        <div style={{ display: 'flex', gap: '10px', justifyContent: 'center', marginBottom: '20px', flexWrap: 'wrap' }}>
          <button
            onClick={testConnection}
            style={{
              padding: '12px 24px',
              background: '#FF9500',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            🧪 测试连接
          </button>
          <button
            onClick={showDefaultGrid}
            style={{
              padding: '12px 24px',
              background: '#007AFF',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            显示默认网格 (3x3)
          </button>
          <button
            onClick={showLargeGrid}
            style={{
              padding: '12px 24px',
              background: '#34C759',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            显示大网格 (4x5)
          </button>
          <button
            onClick={showAreaOverlay}
            style={{
              padding: '12px 24px',
              background: '#AF52DE',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            显示区域覆盖层 (9区域)
          </button>
          <button
            onClick={hideAllOverlays}
            style={{
              padding: '12px 24px',
              background: '#FF3B30',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            隐藏所有覆盖层
          </button>
        </div>

        <div style={{
          background: '#2a2a2a',
          padding: '20px',
          borderRadius: '8px',
          maxWidth: '800px',
          margin: '0 auto'
        }}>
          <h3 style={{ color: '#4CAF50', marginBottom: '15px' }}>📋 使用说明</h3>
          <p style={{ marginBottom: '10px' }}><strong>1. 激活网格:</strong> 点击上面的按钮显示网格</p>
          <p style={{ marginBottom: '10px' }}><strong>2. 使用双键组合:</strong> 网格出现后，按对应的双键组合移动鼠标</p>
          <p style={{ marginBottom: '15px' }}><strong>3. 退出网格:</strong> 按空格键或ESC键，或点击"隐藏所有覆盖层"</p>

          <h4 style={{ color: '#007AFF', marginBottom: '15px' }}>🎯 默认 3x3 网格键位:</h4>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: '8px',
            maxWidth: '300px',
            margin: '0 auto'
          }}>
            {['AQ\n左上', 'AW\n上中', 'AE\n右上', 'SQ\n左中', 'SW\n中心', 'SE\n右中', 'DQ\n左下', 'DW\n下中', 'DE\n右下'].map((text, i) => (
              <div key={i} style={{
                background: '#444',
                padding: '12px',
                textAlign: 'center',
                borderRadius: '4px',
                border: '2px solid #666',
                fontFamily: 'monospace',
                fontSize: '14px',
                fontWeight: 'bold',
                whiteSpace: 'pre-line'
              }}>
                {text}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Status */}
      <div style={{
        textAlign: 'center',
        padding: '10px',
        background: '#333',
        borderRadius: '4px',
        marginTop: '20px'
      }}>
        <strong>状态:</strong> {overlayType === 'none' ? '无活动覆盖层' : `当前模式: ${overlayType}`}
        {gridData && <span> | 网格: {gridData.config.rows}x{gridData.config.columns}</span>}
        {keySequence && <span> | 当前输入: {keySequence.toUpperCase()}_</span>}
      </div>

      {/* 网格覆盖层现在通过独立的 Tauri 窗口显示 */}
    </div>
  )
}

export default TestApp