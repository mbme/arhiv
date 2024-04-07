import { BazaEvent } from 'dto';
import { useEffect, useRef } from 'react';
import { Callback } from 'utils';

let bazaEvents: EventSource;

function subscribeToBazaEvents(cb: (event: BazaEvent) => void): Callback {
  if (!bazaEvents) {
    bazaEvents = new EventSource(`${window.BASE_PATH}/events`);
  }

  const rpcEventHandler = (e: MessageEvent<string>) => {
    const event = JSON.parse(e.data) as BazaEvent;
    cb(event);
  };

  bazaEvents.addEventListener('message', rpcEventHandler);

  return () => {
    bazaEvents.removeEventListener('message', rpcEventHandler);
  };
}

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
