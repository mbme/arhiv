function App() {
  const router = RouterContext.use()

  const [location] = useObservable(() => router.location$.value$)

  if (!location) {
    return null
  }

  switch (location.path) {
    case '/': {
      return (
        <WorkspaceViewContainer />
      )
    }

    case '/library': {
      return (
        <Box
          maxWidth="50rem"
          mx="auto"
          p="medium"
        >
          <Library />
        </Box>
      )
    }

    default: {
      return NotFound
    }
  }
}
