import * as React from 'react'
import {
  Box,
  theme,
  Input,
  Row,
  stylish,
  Button,
} from '~/web-platform'
import { clickOnEnter } from '~/web-platform/utils'
import { ArhivContext } from '../arhiv-context'
import { useWorkspaceURLManager } from './useWorkspaceURLManager'
import { createDocument } from './document-types'

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
}

export function Header(props: IProps) {
  const {
    showCatalog,
    setShowCatalog,
  } = props

  const ws = useWorkspaceURLManager()
  const arhiv = ArhivContext.use()

  function onChange(newFilter: string) {
    ws.updateFilter(newFilter)
    setShowCatalog(newFilter.length > 0)
  }

  async function addNote() {
    const document = await createDocument('note', arhiv)
    ws.openId(document.id)
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
        mr="large"
      >
        <Input
          light
          name="filter"
          placeholder="Filter documents"
          value={ws.filter}
          onChange={onChange}
          onClear={() => onChange('')}
          onKeyDown={(e) => {
            if (e.key === 'Escape') {
              onChange('')
            }
          }}
        />
      </Box>

      <Button variant="primary" onClick={addNote}>
        Add
      </Button>
    </Row>
  )
}
