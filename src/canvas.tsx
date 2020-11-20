import React, { useContext, useEffect, useRef } from 'react'
import 'canvas.css'
import { Painter } from 'rust/paint/pkg'
import PainterContext, { PainterContextProps } from 'PainterContext'

async function loadPainter(target: HTMLDivElement): Promise<Painter> {
  let mod = await import('rust/paint/pkg')
  let painter = new mod.Painter(target)
  return painter
}

type CanvasProps = {
  onPainterInit: (p: Painter) => void
  onPainterFree: () => void
}

function Canvas({ onPainterInit, onPainterFree }: CanvasProps) {
  const painter = useContext(PainterContext)
  const target = useRef<HTMLDivElement>(null)
  useEffect(() => {
    loadPainter(target.current!)
      .then(newPainter => {
        console.log('Initialized Painter')
        onPainterInit(newPainter)
      })
      .catch(console.error)
    return () => {
      painter.current?.free()
      onPainterFree()
    }
  }, [])

  return <div id="canvasTarget" ref={target} />
}

export default Canvas
