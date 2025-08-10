import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface HotkeyConfig {
  gridMode: string
  areaMode: string
  predictionMode: string
  exitKey: string
}

interface GridSettings {
  rows: number
  columns: number
  opacity: number
  showLabels: boolean
  cellPadding: number
  borderWidth: number
}

interface AppSettings {
  hotkeys: HotkeyConfig
  gridSettings: GridSettings
  autoStart: boolean
  theme: 'light' | 'dark'
}

const defaultSettings: AppSettings = {
  hotkeys: {
    gridMode: 'Cmd+G',
    areaMode: 'Cmd+A',
    predictionMode: 'Cmd+P',
    exitKey: 'Escape'
  },
  gridSettings: {
    rows: 3,
    columns: 3,
    opacity: 0.8,
    showLabels: true,
    cellPadding: 2,
    borderWidth: 1
  },
  autoStart: false,
  theme: 'dark'
}

function SettingsApp() {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings)
  const [hasPermissions, setHasPermissions] = useState<boolean>(true)
  const [isRecordingHotkey, setIsRecordingHotkey] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'general' | 'hotkeys' | 'grid' | 'about'>('general')

  useEffect(() => {
    checkPermissions()
    loadSettings()
  }, [])

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
      setTimeout(checkPermissions, 1000)
    } catch (error) {
      console.error('Failed to request permissions:', error)
    }
  }

  const loadSettings = async () => {
    try {
      //TODO: 从后端加载设置
      //TODO: 实现设置加载错误处理
      //TODO: 显示加载状态指示器
      // const savedSettings = await invoke<AppSettings>('load_settings')
      // setSettings(savedSettings)
    } catch (error) {
      console.error('Failed to load settings:', error)
    }
  }

  const saveSettings = async () => {
    try {
      //TODO: 保存设置到后端
      //TODO: 显示保存成功/失败的用户反馈
      //TODO: 实现设置验证
      //TODO: 添加保存状态指示器
      // await invoke('save_settings', { settings })
      console.log('Settings saved:', settings)
    } catch (error) {
      console.error('Failed to save settings:', error)
    }
  }

  const testGridMode = async () => {
    try {
      await invoke('show_grid_overlay', {
        rows: settings.gridSettings.rows,
        columns: settings.gridSettings.columns,
        show_labels: settings.gridSettings.showLabels,
        cell_padding: settings.gridSettings.cellPadding,
        border_width: settings.gridSettings.borderWidth,
        opacity: settings.gridSettings.opacity
      })
    } catch (error) {
      console.error('Failed to test grid mode:', error)
    }
  }

  const testAreaMode = async () => {
    try {
      await invoke('show_area_overlay')
    } catch (error) {
      console.error('Failed to test area mode:', error)
    }
  }

  const hideAllOverlays = async () => {
    try {
      await invoke('hide_all_overlays')
    } catch (error) {
      console.error('Failed to hide overlays:', error)
    }
  }

  // const updateHotkey = (key: keyof HotkeyConfig, value: string) => {
  //   setSettings(prev => ({
  //     ...prev,
  //     hotkeys: {
  //       ...prev.hotkeys,
  //       [key]: value
  //     }
  //   }))
  // }

  const updateGridSettings = (key: keyof GridSettings, value: number | boolean) => {
    setSettings(prev => ({
      ...prev,
      gridSettings: {
        ...prev.gridSettings,
        [key]: value
      }
    }))
  }

  if (!hasPermissions) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="bg-gray-800 p-8 rounded-lg shadow-lg max-w-md w-full">
          <div className="text-center mb-6">
            <div className="text-6xl mb-4">🔒</div>
            <h2 className="text-2xl font-bold mb-2">需要辅助功能权限</h2>
            <p className="text-gray-400">
              Mouseless 需要辅助功能权限才能控制鼠标移动
            </p>
          </div>
          <div className="space-y-3">
            <button
              onClick={requestPermissions}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
            >
              打开系统偏好设置
            </button>
            <button
              onClick={checkPermissions}
              className="w-full bg-gray-700 hover:bg-gray-600 text-white font-medium py-3 px-4 rounded-lg transition-colors"
            >
              重新检查权限
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="text-2xl">🎯</div>
            <h1 className="text-xl font-bold">Mouseless</h1>
            <span className="text-sm text-gray-400">v1.0.0</span>
          </div>
          <div className="flex items-center space-x-3">
            <button
              onClick={saveSettings}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              保存设置
            </button>
          </div>
        </div>
      </header>

      <div className="flex">
        {/* Sidebar */}
        <nav className="w-64 bg-gray-800 border-r border-gray-700 min-h-screen">
          <div className="p-4">
            <div className="space-y-2">
              {[
                { id: 'general', label: '常规设置', icon: '⚙️' },
                { id: 'hotkeys', label: '快捷键', icon: '⌨️' },
                { id: 'grid', label: '网格设置', icon: '🎯' },
                { id: 'about', label: '关于', icon: 'ℹ️' }
              ].map(tab => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`w-full flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors ${
                    activeTab === tab.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-300 hover:bg-gray-700'
                  }`}
                >
                  <span>{tab.icon}</span>
                  <span>{tab.label}</span>
                </button>
              ))}
            </div>
          </div>
        </nav>

        {/* Main Content */}
        <main className="flex-1 p-6">
          {activeTab === 'general' && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold mb-6">常规设置</h2>
              
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">应用设置</h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <label className="text-sm font-medium">开机自启动</label>
                      <p className="text-xs text-gray-400">系统启动时自动运行 Mouseless</p>
                    </div>
                    <input
                      type="checkbox"
                      checked={settings.autoStart}
                      onChange={(e) => setSettings(prev => ({ ...prev, autoStart: e.target.checked }))}
                      className="w-4 h-4"
                    />
                    {/* TODO: Implement actual auto-start functionality for macOS */}
                    {/* TODO: Add system service/daemon registration */}
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div>
                      <label className="text-sm font-medium">主题</label>
                      <p className="text-xs text-gray-400">选择应用主题</p>
                    </div>
                    <select
                      value={settings.theme}
                      onChange={(e) => setSettings(prev => ({ ...prev, theme: e.target.value as 'light' | 'dark' }))}
                      className="bg-gray-700 text-white px-3 py-1 rounded"
                    >
                      <option value="dark">深色</option>
                      <option value="light">浅色</option>
                    </select>
                  </div>
                </div>
              </div>

              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">快速测试</h3>
                <div className="grid grid-cols-2 gap-4">
                  <button
                    onClick={testGridMode}
                    className="bg-blue-600 hover:bg-blue-700 text-white py-3 px-4 rounded-lg transition-colors"
                  >
                    🎯 测试网格模式
                  </button>
                  <button
                    onClick={testAreaMode}
                    className="bg-purple-600 hover:bg-purple-700 text-white py-3 px-4 rounded-lg transition-colors"
                  >
                    📍 测试区域模式
                  </button>
                  <button
                    onClick={hideAllOverlays}
                    className="bg-red-600 hover:bg-red-700 text-white py-3 px-4 rounded-lg transition-colors col-span-2"
                  >
                    ❌ 隐藏所有覆盖层
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'hotkeys' && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold mb-6">快捷键设置</h2>
              
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">模式激活快捷键</h3>
                <div className="space-y-4">
                  {[
                    { key: 'gridMode', label: '网格模式', desc: '激活网格覆盖层进行精确定位' },
                    { key: 'areaMode', label: '区域模式', desc: '激活9区域快速定位' },
                    { key: 'predictionMode', label: '预测模式', desc: '基于AI预测的目标定位' },
                    { key: 'exitKey', label: '退出键', desc: '退出当前模式' }
                  ].map(item => (
                    <div key={item.key} className="flex items-center justify-between p-4 bg-gray-700 rounded-lg">
                      <div>
                        <label className="text-sm font-medium">{item.label}</label>
                        <p className="text-xs text-gray-400">{item.desc}</p>
                      </div>
                      <button
                        onClick={() => setIsRecordingHotkey(item.key)}
                        className={`px-4 py-2 rounded-lg font-mono text-sm transition-colors ${
                          isRecordingHotkey === item.key
                            ? 'bg-blue-600 text-white'
                            : 'bg-gray-600 hover:bg-gray-500 text-gray-200'
                        }`}
                      >
                        {isRecordingHotkey === item.key 
                          ? '按下新快捷键...' 
                          : settings.hotkeys[item.key as keyof HotkeyConfig]
                        }
                      </button>
                      {/* TODO: Implement actual hotkey recording functionality */}
                      {/* TODO: Add hotkey conflict detection */}
                      {/* TODO: Validate hotkey combinations */}
                    </div>
                  ))}
                </div>
              </div>

              <div className="bg-yellow-900/20 border border-yellow-600/30 p-4 rounded-lg">
                <div className="flex items-start space-x-3">
                  <span className="text-yellow-500">⚠️</span>
                  <div>
                    <h4 className="font-medium text-yellow-200">快捷键说明</h4>
                    <p className="text-sm text-yellow-300 mt-1">
                      快捷键将在全局范围内生效。请确保不与其他应用的快捷键冲突。
                      建议使用 Cmd/Ctrl + 字母 的组合。
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'grid' && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold mb-6">网格设置</h2>
              
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">网格布局</h3>
                <div className="grid grid-cols-2 gap-6">
                  <div>
                    <label className="block text-sm font-medium mb-2">行数</label>
                    <input
                      type="range"
                      min="2"
                      max="6"
                      value={settings.gridSettings.rows}
                      onChange={(e) => updateGridSettings('rows', parseInt(e.target.value))}
                      className="w-full"
                    />
                    <div className="text-center text-sm text-gray-400 mt-1">
                      {settings.gridSettings.rows} 行
                    </div>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium mb-2">列数</label>
                    <input
                      type="range"
                      min="2"
                      max="6"
                      value={settings.gridSettings.columns}
                      onChange={(e) => updateGridSettings('columns', parseInt(e.target.value))}
                      className="w-full"
                    />
                    <div className="text-center text-sm text-gray-400 mt-1">
                      {settings.gridSettings.columns} 列
                    </div>
                  </div>
                </div>
              </div>

              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">外观设置</h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">透明度</label>
                    <input
                      type="range"
                      min="0.1"
                      max="1"
                      step="0.1"
                      value={settings.gridSettings.opacity}
                      onChange={(e) => updateGridSettings('opacity', parseFloat(e.target.value))}
                      className="w-full"
                    />
                    <div className="text-center text-sm text-gray-400 mt-1">
                      {Math.round(settings.gridSettings.opacity * 100)}%
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <div>
                      <label className="text-sm font-medium">显示标签</label>
                      <p className="text-xs text-gray-400">在网格单元格中显示快捷键标签</p>
                    </div>
                    <input
                      type="checkbox"
                      checked={settings.gridSettings.showLabels}
                      onChange={(e) => updateGridSettings('showLabels', e.target.checked)}
                      className="w-4 h-4"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">单元格间距</label>
                    <input
                      type="range"
                      min="0"
                      max="10"
                      value={settings.gridSettings.cellPadding}
                      onChange={(e) => updateGridSettings('cellPadding', parseInt(e.target.value))}
                      className="w-full"
                    />
                    <div className="text-center text-sm text-gray-400 mt-1">
                      {settings.gridSettings.cellPadding}px
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">边框宽度</label>
                    <input
                      type="range"
                      min="1"
                      max="5"
                      value={settings.gridSettings.borderWidth}
                      onChange={(e) => updateGridSettings('borderWidth', parseInt(e.target.value))}
                      className="w-full"
                    />
                    <div className="text-center text-sm text-gray-400 mt-1">
                      {settings.gridSettings.borderWidth}px
                    </div>
                  </div>
                </div>
              </div>

              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">预览</h3>
                <div className="flex justify-center">
                  <div 
                    className="grid gap-1 p-4 bg-gray-700 rounded-lg"
                    style={{
                      gridTemplateColumns: `repeat(${settings.gridSettings.columns}, 1fr)`,
                      opacity: settings.gridSettings.opacity
                    }}
                  >
                    {Array.from({ length: settings.gridSettings.rows * settings.gridSettings.columns }).map((_, i) => (
                      <div
                        key={i}
                        className="w-12 h-12 bg-blue-600 border border-blue-400 rounded flex items-center justify-center text-xs font-mono"
                        style={{
                          padding: `${settings.gridSettings.cellPadding}px`,
                          borderWidth: `${settings.gridSettings.borderWidth}px`
                        }}
                      >
                        {settings.gridSettings.showLabels && `${i + 1}`}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'about' && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold mb-6">关于 Mouseless</h2>
              
              <div className="bg-gray-800 p-6 rounded-lg text-center">
                <div className="text-6xl mb-4">🎯</div>
                <h3 className="text-2xl font-bold mb-2">Mouseless</h3>
                <p className="text-gray-400 mb-4">无鼠标精确定位工具</p>
                <p className="text-sm text-gray-500">版本 1.0.0</p>
              </div>

              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">功能特性</h3>
                <ul className="space-y-2 text-sm text-gray-300">
                  <li className="flex items-center space-x-2">
                    <span className="text-green-500">✓</span>
                    <span>网格模式 - 精确的屏幕区域定位</span>
                  </li>
                  <li className="flex items-center space-x-2">
                    <span className="text-green-500">✓</span>
                    <span>区域模式 - 快速的9区域定位</span>
                  </li>
                  <li className="flex items-center space-x-2">
                    <span className="text-yellow-500">⚠</span>
                    <span>预测模式 - AI驱动的智能定位 (开发中)</span>
                  </li>
                  <li className="flex items-center space-x-2">
                    <span className="text-green-500">✓</span>
                    <span>全局快捷键支持</span>
                  </li>
                  <li className="flex items-center space-x-2">
                    <span className="text-green-500">✓</span>
                    <span>可自定义的网格布局</span>
                  </li>
                </ul>
              </div>

              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-lg font-semibold mb-4">使用说明</h3>
                <div className="space-y-3 text-sm text-gray-300">
                  <div>
                    <h4 className="font-medium text-white mb-1">网格模式:</h4>
                    <p>1. 按下网格模式快捷键激活网格</p>
                    <p>2. 使用双键组合定位到目标位置</p>
                    <p>3. 按 ESC 或空格键退出</p>
                  </div>
                  <div>
                    <h4 className="font-medium text-white mb-1">区域模式:</h4>
                    <p>1. 按下区域模式快捷键激活9区域</p>
                    <p>2. 按对应字母键快速定位</p>
                    <p>3. 按 ESC 退出</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  )
}

export default SettingsApp