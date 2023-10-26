import { useEffect, useState } from 'react';
import { DocumentId, DocumentLockKey } from 'dto';
import { useSessionState } from 'utils/hooks';
import { RPC } from 'utils/rpc';

export function useLockDocument(id: DocumentId): DocumentLockKey | null {
  const [lockKey, setLockKey] = useSessionState<DocumentLockKey | null>(
    `document-lock-key-${id}`,
    null,
  );
  const [error, setError] = useState<unknown>();

  if (error) {
    console.error(`Failed to lock document ${id}`, error);

    throw new Error(`Failed to lock document ${id}`);
  }

  useEffect(() => {
    let mounted = true;

    if (!lockKey) {
      void RPC.LockDocument({ id })
        .then(
          ({ lockKey }) => {
            if (mounted) {
              setLockKey(lockKey);
            } else {
              return RPC.UnlockDocument({ id, lockKey });
            }
          },
          (e) => {
            console.error(`Failed to lock document ${id}`, e);
            setError(e);
          },
        )
        .catch((e) => {
          console.error(`Failed to unlock document ${id}`, e);
        });

      return;
    }

    return () => {
      mounted = false;

      void RPC.UnlockDocument({ id, lockKey }).then(
        () => {
          console.debug(`Unlocked document ${id}`);
          setLockKey(null);
        },
        (e) => {
          console.error(`Failed to unlock document ${id}`, e);
          setLockKey(null); // local lock key is most likely invalid anyway
        },
      );
    };
  }, [id, lockKey, setLockKey]);

  return lockKey;
}
