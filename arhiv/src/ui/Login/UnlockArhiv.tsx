import { useState } from 'react';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { ErrorMessage } from 'components/ErrorMessage';
import { Form } from 'components/Form/Form';
import { LoginContainer } from './LoginContainer';
import { ImportArhivKey } from './ImportArhivKey';

export function UnlockArhiv() {
  const [importMode, setImportMode] = useState(false);

  const [password, setPassword] = useState('');

  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.UnlockArhiv({ password }, abortSignal),
    {
      refreshOnMount: false,
      onSuccess() {
        location.reload();
      },
    },
  );

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
    </LoginContainer>
  );
}
