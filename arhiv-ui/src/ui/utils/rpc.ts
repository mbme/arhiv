import { APIRequest, APIResponse } from 'dto';
import { Obj } from './index';

export type RPCResponse<Request extends APIRequest> = Extract<
  APIResponse,
  { typeName: Request['typeName'] }
>;

export async function doRPC<Request extends APIRequest>(
  url: string,
  request: Request,
  signal?: AbortSignal
): Promise<RPCResponse<Request>> {
  console.debug('RPC: %s', request.typeName, request);

  const onAbort = () => {
    console.debug('RPC: aborted %s', request.typeName, request);
  };
  signal?.addEventListener('abort', onAbort);

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
      signal,
    });

    const message = await response.text();

    if (!response.ok) {
      throw new Error(`API call failed: ${response.status}\n${message}`);
    }

    return JSON.parse(message) as RPCResponse<Request>;
  } catch (e) {
    console.error(e);
    throw e;
  } finally {
    signal?.removeEventListener('abort', onAbort);
  }
}

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
      get(_, prop: string) {
        return (params: Obj, signal?: AbortSignal) =>
          doRPC(url, { typeName: prop, ...params } as APIRequest, signal);
      },
    }
  ) as ProxyHandlers<Request, Response>;
}

export const RPC = createRPCProxy<APIRequest, APIResponse>(window.API_ENDPOINT);
