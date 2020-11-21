import React, { useState } from 'react'
import './App.css'
import { ColorResult, RGBColor, SketchPicker } from 'react-color'

import PainterContext, { PainterContextProps } from 'PainterContext'
import Canvas from 'canvas'
import type { Painter } from 'rust/paint/pkg'
import { normalizeRgbaFloat32Array } from 'util/color'

function App() {
  const [painterCtx, setPainterCtx] = useState<PainterContextProps>({
    current: null
  })
  const [color, setColor] = useState<RGBColor>({
    r: 128,
    g: 128,
    b: 128,
    a: 1.0
  })
  const handlePainterInit = (painter: Painter) => {
    setPainterCtx({ current: painter })
  }
  const handlePainterFree = () => {
    setPainterCtx({ current: null })
  }
  const changeColor = (
    { rgb }: ColorResult,
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    event.stopPropagation()
    painterCtx.current?.changeColor(
      normalizeRgbaFloat32Array(new Float32Array([rgb.r, rgb.g, rgb.b, rgb.a!]))
    )
    setColor(rgb)
  }
  return (
    <>
      <div className="container">
        <PainterContext.Provider value={painterCtx}>
          <div className="ui">
            <div className="ui-drawer">
              <h1 className="unselectable">WebGl2 Painter</h1>

              <SketchPicker
                color={color}
                onChange={changeColor}
                // disableAlpha
              />
            </div>
          </div>
          <Canvas
            onPainterInit={handlePainterInit}
            onPainterFree={handlePainterFree}
          />
        </PainterContext.Provider>
      </div>
    </>
  )
}

export default App
