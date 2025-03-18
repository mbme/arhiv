import { useState } from 'react';
import { RPC } from 'utils/network';
import { useQuery } from 'utils/hooks';
import { LoginContainer } from 'components/LoginContainer';
import { Form } from 'components/Form/Form';
import { Button } from 'components/Button';

export function UnlockArhiv() {
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

  return (
    <LoginContainer>
      <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={triggerRefresh}>
        {error && <div className="text-red-500 text-xl pl-1 my-2">{String(error)}</div>}

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

        <Button variant="primary" type="submit" busy={inProgress}>
          Unlock
        </Button>
      </Form>
    </LoginContainer>
  );
}
