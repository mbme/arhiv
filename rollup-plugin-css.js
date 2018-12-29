import path from 'path'
import fs from 'fs'

export default function rollupPluginCss() {
  const styles = new Map()
  let changes = 0

  return {
    name: 'css',

    transform(code, id) {
      if (!id.endsWith('.css')) return

      const existingCode = styles.get(id)
      if (existingCode !== code) {
        styles.set(id, code)
        changes += 1
      }

      return ''
    },

    generateBundle(opts, bundle) {
      if (!changes) return

      changes = 0

      const entries = Array.from(styles.entries())
      if (bundle.modules) { // sort by module occurrence order
        const fileList = Object.keys(bundle.modules)
        entries.sort((a, b) => fileList.indexOf(a[0]) - fileList.indexOf(b[0]))
      }

      const css = entries.map(entry => entry[1]).join('')

      const basename = path.basename(opts.file, path.extname(opts.file))
      const outputFile = path.join(path.dirname(opts.file), basename + '.css')

      return fs.promises.writeFile(outputFile, css)
    },
  }
}
