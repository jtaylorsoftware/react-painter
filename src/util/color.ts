function normalizeColor(color: number): number {
  return color / 255.0
}

/**
 * Normalizes an rgba 4-tuple, leaving alpha untouched
 * @param rgba rgba 4-tuple where each RGB value is in range [0, 255] and A is already in range [0.0,1.0]
 * @returns The rgba argument
 */
export function normalizeRgbaFloat32Array(rgba: Float32Array): Float32Array {
  rgba[0] = normalizeColor(rgba[0])
  rgba[1] = normalizeColor(rgba[1])
  rgba[2] = normalizeColor(rgba[2])
  return rgba
}
