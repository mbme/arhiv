import { useEffect, useRef, useState } from 'react';
import { JSONObj } from 'utils';
import { RPC } from 'utils/network';
import { Form } from 'components/Form/Form';
import { ErrorMessage } from 'components/ErrorMessage';
import { Button } from 'components/Button';

export interface ExportedKey {
  key: string;
  qrcodeSvgBase64: string;
  htmlPage: string;
}

interface Props {
  onSuccess: (key: ExportedKey) => void;
}
export function ExportKeyForm({ onSuccess }: Props) {
  const [error, setError] = useState('');
  const [inProgress, setInProgress] = useState(false);

  const passwordInputRef = useRef<HTMLInputElement>(null);
  const passwordRepeatInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    passwordInputRef.current?.focus();
  }, []);

  const onSubmit = async (values: JSONObj) => {
    const { password, exportPassword, exportPasswordRepeat } = values;

    if (exportPassword !== exportPasswordRepeat) {
      passwordRepeatInputRef.current?.setCustomValidity(
        'Password confirmation is not the same as password',
      );
      passwordRepeatInputRef.current?.reportValidity();
      return;
    }

    setInProgress(true);

    try {
      const { key, qrcodeSvgBase64, htmlPage } = await RPC.ExportKey({
        password: password as string,
        exportPassword: exportPassword as string,
        $secret: true,
      });

      onSuccess({ key, qrcodeSvgBase64, htmlPage });
    } catch (err) {
      console.error('Failed to export Arhiv key:', err);
      setError(`Failed to export Arhiv key: ${String(err)}`);
    } finally {
      setInProgress(false);
    }
  };

  return (
    <Form className="flex flex-col max-w-md items-center gap-4" onSubmit={onSubmit}>
      {error && <ErrorMessage className="pl-1 my-2">{error}</ErrorMessage>}

      <label className="mb-8">
        Password:
        <input
          type="password"
          name="password"
          ref={passwordInputRef}
          required
          minLength={window.CONFIG.minPasswordLength}
          autoComplete="off"
          autoFocus
        />
      </label>

      <label>
        Password for exported key:
        <input
          type="password"
          name="exportPassword"
          required
          minLength={window.CONFIG.minPasswordLength}
          autoComplete="off"
        />
      </label>

      <label className="mb-8">
        Repeat password for exported key:
        <input
          type="password"
          name="exportPasswordRepeat"
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
        Export key
      </Button>
    </Form>
  );
}
