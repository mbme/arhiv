import { useState } from 'react';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { ErrorMessage } from 'components/ErrorMessage';
import { Icon } from 'components/Icon';
import { Form } from 'components/Form/Form';
import { LoginContainer } from './LoginContainer';
import { ImportArhivKey } from './ImportArhivKey';

export function UnlockArhiv() {
  const [importMode, setImportMode] = useState(false);
  const [initialized, setInitialized] = useState(false);

  const [password, setPassword] = useState<string>();

  const { error, inProgress, triggerRefresh, result } = useQuery(async (abortSignal) => {
    try {
      await RPC.UnlockArhiv({ password, $secret: true }, abortSignal);
      location.reload();
    } catch (e) {
      setInitialized(true);
      throw e;
    }
  });

  const isUnlocking = !initialized || inProgress || result;

  if (importMode) {
    return (
      <ImportArhivKey
        onCancel={() => {
          setImportMode(false);
        }}
      />
    );
  }

  return (
    <LoginContainer heading="Unlock Arhiv">
      {isUnlocking ? (
        <div className="px-4 py-6 flex flex-col items-center justify-center gap-8">
          <Icon variant="spinner" className="h-10 w-10 opacity-50" />
          <span className="text-sm text-gray-400">Unlocking...</span>
        </div>
      ) : (
        <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={triggerRefresh}>
          {error && <ErrorMessage className="pl-1 my-2">{String(error)}</ErrorMessage>}

          <label>
            Password:
            <input
              type="password"
              name="password"
              required
              minLength={window.CONFIG.minPasswordLength}
              autoComplete="off"
              autoFocus
              onChange={(e) => {
                setPassword(e.currentTarget.value);
              }}
            />
          </label>

          <Button variant="primary" type="submit" busy={inProgress} className="mb-4">
            Unlock
          </Button>

          <Button
            variant="simple"
            onClick={() => {
              setImportMode(true);
            }}
          >
            Import key
          </Button>
        </Form>
      )}
    </LoginContainer>
  );
}
