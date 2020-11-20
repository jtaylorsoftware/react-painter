import React, { useState } from 'react'
import './App.css'

import PainterContext, { PainterContextProps } from 'PainterContext'
import Canvas from 'canvas'
import type { Painter } from 'rust/paint/pkg'

function App() {
  const [painterCtx, setPainterCtx] = useState<PainterContextProps>({
    current: null
  })
  const handlePainterInit = (painter: Painter) => {
    setPainterCtx({ current: painter })
  }
  const handlePainterFree = () => {
    setPainterCtx({ current: null })
  }
  const changeColor = () => {
    painterCtx.current?.changeColor(new Float32Array([1.0, 0, 0]))
  }
  return (
    <>
      <div className="container">
        <PainterContext.Provider value={painterCtx}>
          <div className="ui">
            <h1 className="unselectable">WebGl2 Painter</h1>
            <button onClick={changeColor} className="unselectable">
              Click to change color
            </button>
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
