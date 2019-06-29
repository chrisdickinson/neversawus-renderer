#!/usr/bin/env node
'use strict'

// zip up

const archiver = require('archiver')
const fs = require('fs')

const archive = archiver('zip', {
  zlib: { level: 9 }
})
archive.pipe(fs.createWriteStream('renderer.zip'))
archive.file('dist/main.js', {name: 'lib/index.js'})
archive.finalize()
