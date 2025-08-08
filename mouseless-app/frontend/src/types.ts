export interface Position {
  x: number
  y: number
  screen_id?: number
}

export interface GridConfig {
  rows: number
  columns: number
  show_labels: boolean
  cell_size: [number, number]
  animation_style: AnimationStyle
}

export interface PredictionTarget {
  position: Position
  confidence: number
  target_type: TargetType
  label?: string
}

export type AnimationStyle = 'None' | 'Smooth' | 'Bounce' | 'Fade'

export type TargetType = 'Button' | 'Link' | 'Input' | 'Menu' | 'Icon' | 'Text'

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