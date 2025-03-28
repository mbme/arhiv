import { useRef, useState } from 'react';
import { JSONObj } from 'utils';
import { RPC } from 'utils/network';
import { Form } from 'components/Form/Form';
import { Button } from 'components/Button';
import { LoginContainer } from 'login/LoginContainer';
import { ErrorMessage } from 'components/ErrorMessage';

export function CreateArhiv() {
  const [error, setError] = useState('');
  const [inProgress, setInProgress] = useState(false);
  const passwordRepeatInputRef = useRef<HTMLInputElement>(null);

  const onSubmit = async (values: JSONObj) => {
    const { password, passwordRepeat } = values;

    if (password !== passwordRepeat) {
      passwordRepeatInputRef.current?.setCustomValidity(
        'Password confirmation is not the same as password',
      );
      passwordRepeatInputRef.current?.reportValidity();
      return;
    }

    setInProgress(true);

    try {
      await RPC.CreateArhiv({ password: password as string });

      location.reload();
    } catch (err) {
      console.error('Failed to create Arhiv:', err);
      setError(`Failed to create Arhiv: ${String(err)}`);
    } finally {
      setInProgress(false);
    }
  };

  return (
    <LoginContainer heading="Create Arhiv">
      <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={onSubmit}>
        {error && <ErrorMessage className="pl-1 my-2">{error}</ErrorMessage>}

        <label>
          Password:
          <input
            type="password"
            name="password"
            required
            minLength={window.CONFIG.minPasswordLength}
            autoComplete="off"
            autoFocus
          />
        </label>

        <label>
          Repeat password:
          <input
            type="password"
            name="passwordRepeat"
            ref={passwordRepeatInputRef}
            required
            minLength={window.CONFIG.minPasswordLength}
            autoComplete="off"
            onChange={(e) => {
              e.currentTarget.setCustomValidity('');
            }}
          />
        </label>

        <Button variant="primary" type="submit" busy={inProgress}>
          Create
        </Button>
      </Form>
    </LoginContainer>
  );
}
