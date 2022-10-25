import { APIRequest, APIResponse } from '../../dto';
import { Obj } from './index';

type SerdeEnum = {
  typeName: string;
};

type ProxyHandlers<Request extends SerdeEnum, Response extends SerdeEnum> = {
  [key in Request['typeName']]: (
    request: Omit<Extract<Request, { typeName: key }>, 'typeName'>,
    signal?: AbortSignal
  ) => Promise<Extract<Response, { typeName: key }>>;
};

function createRPCProxy<Request extends SerdeEnum, Response extends SerdeEnum>(
  url: string
): ProxyHandlers<Request, Response> {
  return new Proxy(
    {},
    {
      get(_, prop) {
        return async (params: Obj, signal?: AbortSignal) => {
          console.debug('RPC: %s', prop, params);

          const onAbort = () => {
            console.debug('RPC: aborted %s', prop, params);
          };
          signal?.addEventListener('abort', onAbort);

          try {
            const response = await fetch(url, {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                typeName: prop as string,
                ...params,
              }),
              signal,
            });

            const message = await response.text();

            if (!response.ok) {
              throw new Error(`API call failed: ${response.status}\n${message}`);
            }

            return JSON.parse(message) as Obj;
          } catch (e) {
            console.error(e);
            throw e;
          } finally {
            signal?.removeEventListener('abort', onAbort);
          }
        };
      },
    }
  ) as ProxyHandlers<Request, Response>;
}

export const RPC = createRPCProxy<APIRequest, APIResponse>('/api');
