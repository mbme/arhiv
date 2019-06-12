import { test } from '~/tester'
import {
  StyleNode,
  hash2class,
} from './StyleNode'

test('it works', (assert) => {
  const style = new StyleNode({
    fontSize: 1,
  })

  assert.equal(
    style.intoCss()[0],
    `${hash2class(style.hash)} { font-size: 1 }`,
  )
})

test('it generates consistent hashes', (assert) => {
  const style1 = new StyleNode({
    fontSize: 1,
    '@media': {
      fontSize: 1,
      margin: 0,
    },
    '&:hover': {
      fontSize: 1,
      margin: 0,
    },
    margin: 0,
  })

  const style2 = new StyleNode({
    margin: 0,
    '@media': {
      margin: 0,
      fontSize: 1,
    },
    fontSize: 1,
    '&:hover': {
      margin: 0,
      fontSize: 1,
    },
  })

  assert.equal(style1.hash, style2.hash)
})

test('it supports with nested selectors', (assert) => {
  const style = new StyleNode({
    fontSize: 1,
    '.test &:hover, &': {
      fontSize: 2,
    },
  })

  assert.matchSnapshot(style.intoCss())
})

test('it supports media query', (assert) => {
  const style = new StyleNode({
    fontSize: 1,
    '@media screen and (min-width: 800px)': {
      fontSize: 2,
      '&:hover': {
        margin: 0,
      },
    },
  })

  assert.matchSnapshot(style.intoCss())
})

test('it renders @keyframes', (assert) => {
  const style = new StyleNode({
    from: {
      width: '300%',
      marginLeft: '0',
    },
    to: {
      marginLeft: '100px',
      width: '100%',
    },
  })

  assert.matchSnapshot(style.asKeyframes())
})
