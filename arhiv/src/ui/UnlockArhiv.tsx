import { useState } from 'react';
import { JSONObj } from 'utils';
import { unlockArhiv } from 'utils/network';
import { LoginContainer } from 'components/LoginContainer';
import { Form } from 'components/Form/Form';
import { Button } from 'components/Button';

export function UnlockArhiv() {
  const [error, setError] = useState('');
  const [inProgress, setInProgress] = useState(false);

  const onSubmit = async ({ password }: JSONObj) => {
    setInProgress(true);

    try {
      await unlockArhiv(password as string);

      location.reload();
    } catch (err) {
      console.error('Failed to unlock Arhiv:', err);
      setError(`Failed to unlock Arhiv: ${String(err)}`);
    } finally {
      setInProgress(false);
    }
  };

  return (
    <LoginContainer>
      <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={onSubmit}>
        {error && <div className="text-red-500 text-xl pl-1 my-2">{error}</div>}

        <label>
          Password:
          <input
            type="password"
            name="password"
            required
            minLength={window.MIN_PASSWORD_LENGTH}
            autoComplete="off"
          />
        </label>

        <Button variant="primary" type="submit" busy={inProgress}>
          Unlock
        </Button>
      </Form>
    </LoginContainer>
  );
}
