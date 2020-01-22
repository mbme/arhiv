import * as React from 'react'
import {
  Box,
  theme,
  Input,
  Row,
  stylish,
} from '~/web-platform'
import { clickOnEnter } from '~/web-platform/utils'

const $menuItem = stylish(
  {
    textTransform: 'uppercase',
    mx: 'medium',
    cursor: 'pointer',
    borderBottom: '1px solid transparent',
  },
  props => props.active && {
    borderBottom: `1px solid ${theme.color.text}`,
  },
)

interface IProps {
  showCatalog: boolean
  setShowCatalog(showCatalog: boolean): void
  filter: string
  updateFilter(filter: string | undefined): void
}

export function Header(props: IProps) {
  const {
    showCatalog,
    setShowCatalog,
    filter,
    updateFilter,
  } = props

  function onChange(newFilter: string) {
    updateFilter(newFilter)
    setShowCatalog(newFilter.length > 0)
  }

  return (
    <Row
      as="nav"
      alignX="center"
      boxShadow={theme.boxShadow}
      py="fine"
      width="100%"
      bgColor="bg1"
      zIndex="1"
    >
      <div
        onClick={() => setShowCatalog(false)}
        className={$menuItem.with({ active: !showCatalog }).className}
        role="tab"
        tabIndex={0}
        onKeyPress={clickOnEnter}
      >
        Workspace
      </div>

      <div
        onClick={() => setShowCatalog(true)}
        className={$menuItem.with({ active: showCatalog }).className}
        role="tab"
        tabIndex={0}
        onKeyPress={clickOnEnter}
      >
        Catalog
      </div>

      <Box
        width="11rem"
      >
        <Input
          light
          name="filter"
          placeholder="Filter documents"
          value={filter}
          onChange={onChange}
          onClear={() => onChange('')}
          onKeyDown={(e) => {
            if (e.key === 'Escape') {
              onChange('')
            }
          }}
        />
      </Box>
    </Row>
  )
}
