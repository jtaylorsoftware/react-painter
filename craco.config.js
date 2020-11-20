const path = require('path')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')
const webpack = require('webpack')
const { ESLINT_MODES } = require('@craco/craco')

module.exports = {
  webpack: {
    configure: webpackConfig => {
      webpackConfig.resolve.extensions.push('.wasm')

      webpackConfig.module.rules.forEach(rule => {
        ;(rule.oneOf || []).forEach(oneOf => {
          if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
            oneOf.exclude.push(/\.wasm$/)
          }
        })
      })

      webpackConfig.plugins = (webpackConfig.plugins || []).concat([
        new WasmPackPlugin({
          crateDirectory: path.resolve(__dirname, 'src/rust/paint')
        })
      ])

      return webpackConfig
    }
  }
}
