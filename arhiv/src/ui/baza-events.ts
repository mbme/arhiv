import { BazaEvent } from 'dto';
import { useEffect, useRef } from 'react';
import { Callback } from 'utils';

const bazaEvents = new EventSource(`${window.BASE_PATH}/events`);

function subscribeToBazaEvents(cb: (event: BazaEvent) => void): Callback {
  const rpcEventHandler = (e: MessageEvent<string>) => {
    const event = JSON.parse(e.data) as BazaEvent;
    cb(event);
  };

  bazaEvents.addEventListener('message', rpcEventHandler);

  return () => {
    bazaEvents.removeEventListener('message', rpcEventHandler);
  };
}

subscribeToBazaEvents((event) => {
  console.debug('Baza Event:', event);
});

export function useBazaEvent(cb: (event: BazaEvent) => void) {
  const cbRef = useRef(cb);
  cbRef.current = cb;

  useEffect(
    () =>
      subscribeToBazaEvents((event) => {
        cbRef.current(event);
      }),
    [],
  );
}
