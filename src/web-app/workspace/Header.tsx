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
import { useWorkspaceStore } from './store'
import { NoteModule } from './document-types/note'

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

export function Header() {
  const store = useWorkspaceStore()

  function onChange(newFilter: string) {
    store.updateFilter(newFilter)
  }

  function addNote() {
    return store.createDocument(NoteModule)
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
        onClick={() => store.showCatalog(false)}
        className={$menuItem.with({ active: !store.state.showCatalog }).className}
        role="tab"
        tabIndex={0}
        onKeyPress={clickOnEnter}
      >
        Workspace
      </div>

      <div
        onClick={() => store.showCatalog(true)}
        className={$menuItem.with({ active: store.state.showCatalog }).className}
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
          value={store.state.filter}
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
