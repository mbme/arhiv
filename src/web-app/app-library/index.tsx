import * as React from 'react'

import { Library } from '~/web-platform/Library'
import { IApp } from '../chrome'

export const LibraryApp: IApp = {
  name: 'Component Library',
  route: '/library',
  render: () => <Library />,
}
