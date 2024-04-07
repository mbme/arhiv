import { useRef, useState } from 'react';
import { JSONObj } from 'utils';
import { Form } from 'components/Form/Form';
import { Button } from 'components/Button';

export function CreateArhiv() {
  const [error, setError] = useState('');
  const [inProgress, setInProgress] = useState(false);
  const passwordRepeatInputRef = useRef<HTMLInputElement>(null);

  const onSubmit = async (values: JSONObj) => {
    const { login, password, passwordRepeat } = values;
    console.error('ON SUBMIT', values);
    if (password !== passwordRepeat) {
      passwordRepeatInputRef.current?.setCustomValidity(
        'Password confirmation is not the same as password',
      );
      passwordRepeatInputRef.current?.reportValidity();
      return;
    }

    setInProgress(true);

    try {
      const response = await fetch(`${window.BASE_PATH}/create`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ login, password }),
      });

      const message = await response.text();

      if (response.ok) {
        location.reload();
      } else {
        console.error(`Failed to create arhiv: ${response.status}\n${message}`);
        setError(`Failed to create arhiv: ${message}`);
      }
    } catch (err) {
      console.error('Failed to create arhiv:', err);
    } finally {
      setInProgress(false);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center pt-32">
      <img
        src={`${window.BASE_PATH}/favicon.svg`}
        alt="Arhiv logo"
        className="size-24 rounded-md shadow-lg mb-8"
      />

      <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={onSubmit}>
        {error && <div className="text-red-500 text-xl pl-1 my-2">{error}</div>}

        <label>
          Login:
          <input
            type="text"
            name="login"
            className="field"
            required
            minLength={window.MIN_LOGIN_LENGTH}
            autoFocus
            autoComplete="off"
          />
        </label>

        <label>
          Password:
          <input
            type="password"
            name="password"
            className="field"
            required
            minLength={window.MIN_PASSWORD_LENGTH}
            autoComplete="off"
          />
        </label>

        <label>
          Repeat password:
          <input
            type="password"
            name="passwordRepeat"
            ref={passwordRepeatInputRef}
            className="field"
            required
            minLength={window.MIN_PASSWORD_LENGTH}
            autoComplete="off"
            onChange={(e) => {
              e.currentTarget.setCustomValidity('');
            }}
          />
        </label>

        <Button variant="primary" type="submit" busy={inProgress}>
          Create arhiv
        </Button>
      </Form>
    </div>
  );
}
