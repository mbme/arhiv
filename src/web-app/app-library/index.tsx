import * as React from 'react'

import {
  Library,
} from '~/web-platform'

export default {
  name: 'Component Library',
  rootRoute: '/library',
  routes: {
    '/library': () => <Library />,
  },
}
