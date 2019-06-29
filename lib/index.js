'use strict'

const hl = require('remark-highlight.js')
const mdhtml = require('remark-html')
const toml = require('@iarna/toml')
const remark = require('remark')
const AWS = require('aws-sdk')

const FRONTMATTER = '\n---\n\n'

async function main (event, context) {
  const s3Client = new AWS.S3()
  for (const { s3 = null } of event.Records) {
    if (!s3 || !s3.object || !s3.object.key || !s3.object.bucket || !s3.object.bucket.name) {
      continue
    }

    let data = null
    try {
      data = String(await s3Client.getObject({
        Key: s3.object.key,
        Bucket: s3.object.bucket.name
      }).promise())
    } catch (err) {
      console.error(`Caught error ${err.message} while fetching bucket=${s3.object.bucket} key=${s3.object.key}`)
      continue
    }

    let [first, ...rest] = data = data.split(FRONTMATTER, 1)
    if (!rest.length) {
      rest = [first]
      first = ''
    }

    let frontmatter = null
    try {
      frontmatter = toml.parse(first)
    } catch (err) {
      console.error(`Caught error ${err.message} while parsing toml front matter for bucket=${s3.object.bucket} key=${s3.object.key}`)
      continue
    }

    const { title, slug = s3.object.key.replace(/.md$/, ''), date, metadata, framing = 'default.html' } = frontmatter

    const markdown = rest.join(FRONTMATTER)
    const html = remark().use(hl).use(mdhtml).process(markdown)

    await s3Client.putObject({
      Key: slug ? `${slug}/index.html` : s3.object.key.replace(/\.md/, '/index.html'),
      Bucket: process.env.S3_DESTINATION_BUCKET,
      Body: String(html),
      ACL: 'public-read',
      Metadata: {
        title: title || 'Untitled',
        date: date || new Date(),
        framing,
        ...metadata
      }
    }).promise()
  }
}

exports.handlers = (event, context, ready) => {
  return main(event, context).then(
    xs => ready(),
    xs => {
      console.error(xs)
      return ready()
    }
  )
}
