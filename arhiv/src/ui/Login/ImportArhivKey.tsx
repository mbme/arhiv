import { useEffect, useState } from 'react';
import { Callback, formatBytes } from 'utils';
import { RPC } from 'utils/network';
import { useQuery } from 'utils/hooks';
import { Form } from 'components/Form/Form';
import { Textarea } from 'components/Form/Textarea';
import { Button } from 'components/Button';
import { ErrorMessage } from 'components/ErrorMessage';
import { FileInput } from 'components/FileInput';
import { LoginContainer } from './LoginContainer';

interface Props {
  onCancel?: Callback;
}
export function ImportArhivKey({ onCancel }: Props) {
  const [encryptedKey, setEncryptedKey] = useState('');
  const [password, setPassword] = useState('');
  const [formError, setFormError] = useState('');

  const {
    error: importError,
    inProgress,
    triggerRefresh,
  } = useQuery(
    (abortSignal) => RPC.ImportKey({ encryptedKey, password, $secret: true }, abortSignal),
    {
      refreshOnMount: false,
      onSuccess() {
        location.reload();
      },
    },
  );

  // clear form error on input
  useEffect(() => {
    setFormError('');
  }, [encryptedKey, password]);

  return (
    <LoginContainer heading="Import Arhiv key">
      <Form className="flex flex-col items-center gap-4 self-stretch" onSubmit={triggerRefresh}>
        {importError && <ErrorMessage className="pl-1 my-2">{String(importError)}</ErrorMessage>}
        {formError && <ErrorMessage className="pl-1 my-2">{formError}</ErrorMessage>}

        <label className="self-stretch">
          Encrypted key:
          <Textarea
            name="encrypted-key"
            required
            autoGrow
            value={encryptedKey}
            onChange={setEncryptedKey}
          />
        </label>

        <FileInput
          label="Load key from file"
          variant="text"
          onFileSelected={(file) => {
            if (!file) {
              return;
            }

            if (file.size > 10 * 1024) {
              setFormError('Failed to load key from file: file too big ' + formatBytes(file.size));
              return;
            }

            file.text().then(
              (fileData) => {
                setEncryptedKey(fileData.trim());
              },
              (e: unknown) => {
                setFormError('Failed to load key from file: ' + String(e));
              },
            );
          }}
        />

        <label>
          Password:
          <input
            type="password"
            name="password"
            required
            minLength={window.CONFIG.minPasswordLength}
            autoComplete="off"
            onChange={(e) => {
              setPassword(e.currentTarget.value);
            }}
          />
        </label>

        <Button variant="primary" type="submit" busy={inProgress}>
          Import key
        </Button>

        {onCancel && (
          <Button variant="simple" onClick={onCancel}>
            Cancel
          </Button>
        )}
      </Form>
    </LoginContainer>
  );
}
