import { createContext } from 'react'
import type { Painter } from 'rust/paint/pkg'

export type PainterContextProps = {
  current: Painter | null
}

export default createContext<PainterContextProps>({ current: null })
