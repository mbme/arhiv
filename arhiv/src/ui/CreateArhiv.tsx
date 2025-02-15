import { useRef, useState } from 'react';
import { JSONObj } from 'utils';
import { createArhiv } from 'utils/network';
import { Form } from 'components/Form/Form';
import { Button } from 'components/Button';

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
      await createArhiv(password as string);

      location.reload();
    } catch (err) {
      console.error('Failed to create arhiv:', err);
      setError(`Failed to create arhiv: ${String(err)}`);
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
          Password:
          <input
            type="password"
            name="password"
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
