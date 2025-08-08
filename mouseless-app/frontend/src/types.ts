export interface Position {
  x: number
  y: number
  screen_id?: number
}

export interface GridConfig {
  rows: number
  columns: number
  show_labels: boolean
  animation_style: AnimationStyle
  cell_padding: number
  border_width: number
  opacity: number
}

export interface PredictionTarget {
  position: Position
  confidence: number
  target_type: TargetType
  shortcut_key: string
  description?: string
}

export type AnimationStyle = 'Instant' | 'Linear' | 'Smooth' | 'Bounce'

export type TargetType = 'Button' | 'Link' | 'TextField' | 'MenuItem' | 'Icon' | { Custom: string }

export interface ScreenBounds {
  id: number
  x: number
  y: number
  width: number
  height: number
  is_primary: boolean
}

export interface Theme {
  name: string
  overlay_opacity: number
  glassmorphism_enabled: boolean
  grid_color: string
  area_color: string
  prediction_colors: {
    high: string
    medium: string
    low: string
  }
}