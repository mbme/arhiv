<script lang="ts">
  import { getContext } from 'svelte'
  import { Router, Route, Link } from 'svelte-routing'

  import type { IDataSchema } from '@v/arhiv-api'

  import Home from './views/Home.svelte'
  import Catalog from './views/Catalog.svelte'
  import Card from './views/Card.svelte'
  import CardEditor from './views/CardEditor.svelte'
  import NewCardEditor from './views/NewCardEditor.svelte'

  const schema: IDataSchema = getContext('schema')

  const documentTypes = schema.modules.map(item => item.documentType)
</script>

<Router>
  <div class="h-screen min-h-screen bg-gray-100 flex justify-center py-2">
    <nav class="border-r-2 border-light-blue-500 w-40 bg-yellow-300 mr-3 flex flex-col items-end pr-3 pt-20 text-xl">
      {#each documentTypes as documentType (documentType)}
        <Link to="/catalog/{documentType}">
           {documentType}
        </Link>
      {/each}
    </nav>

    <main class="w-100 bg-blue-100">
      <Route path="/" component="{Home}" />

      <Route path="/catalog/:documentType" component="{Catalog}" />
      <Route path="/catalog/:documentType/new" component="{NewCardEditor}" />

      <Route path="/card/:id" component="{Card}" />
      <Route path="/card/:id/edit" component="{CardEditor}" />

      <Route path="*">
        <h1 class="text-red-500 font-bold text-center">NOT FOUND</h1>
      </Route>
    </main>
  </div>
</Router>

<style global>
  html, body, #app {
    height: 100vh;
    min-height: 100vh;

    font-size: 18px;
    text-rendering: optimizeLegibility;
  }
</style>
