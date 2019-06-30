'use strict'

const { DefinePlugin } = require('webpack')
const { exec: _exec } = require('child_process')
const { promisify } = require('util')
const path = require('path')

const exec = promisify(_exec)

module.exports = async () => {
  const { stdout } = await exec('git rev-parse --short HEAD')
  const release = String(stdout).trim() + '-' + new Date().toISOString()

  return {
    entry: './lib/index.js',
    target: 'node',
    output: {
      library: '',
      libraryTarget: 'commonjs',
      filename: '[name].js',
      path: path.resolve('dist')
    },
    mode: 'production',
    optimization: {
      minimize: false
    },
    module: {
      rules: [{
        test (filename) {
          return filename.endsWith('.js')
        },
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {}
        }
      }]
    },
    plugins: [
      new DefinePlugin({
        'process.env.RELEASE': JSON.stringify(release)
      })
    ]
  }
}
