import * as React from 'react'

import { Library } from '~/web-platform/Library'

export default {
  name: 'Component Library',
  rootRoute: '/library',
  routes: {
    '/library': () => <Library />,
  },
}
