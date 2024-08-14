import { useEffect, useState } from 'react';
import { DocumentId, DocumentLockKey } from 'dto';
import { useSessionState } from 'utils/hooks';
import { RPC } from 'utils/network';

export function useLockDocument(
  id: DocumentId,
  lock: boolean,
): { lockKey: DocumentLockKey | null; error: unknown } {
  const [lockKey, setLockKey] = useSessionState<DocumentLockKey | null>(
    `document-lock-key-${id}`,
    null,
  );
  const [error, setError] = useState<unknown>();

  useEffect(() => {
    if (!lock) {
      return;
    }

    let mounted = true;

    if (!lockKey) {
      void RPC.LockDocument({ id }).then(
        ({ lockKey }) => {
          if (mounted) {
            setLockKey(lockKey);
            setError(undefined);
          } else {
            return RPC.UnlockDocument({ id, lockKey }).catch((e: unknown) => {
              console.error(`Failed to unlock document ${id}`, e);
            });
          }
        },
        (e) => {
          console.error(`Failed to lock document ${id}`, e);
          setError(e);
        },
      );

      return;
    }

    return () => {
      mounted = false;

      setLockKey(null);

      void RPC.UnlockDocument({ id, lockKey }).then(
        () => {
          console.debug(`Unlocked document ${id}`);
        },
        (e) => {
          console.error(`Failed to unlock document ${id}`, e);
        },
      );
    };
  }, [id, lock, lockKey, setLockKey]);

  return {
    lockKey,
    error,
  };
}
