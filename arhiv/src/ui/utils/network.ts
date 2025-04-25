import { APIRequest, APIResponse, DocumentId } from 'dto';
import { formatBytes, Obj } from './index';

const SECRET_PLACEHOLDER = '{ SECRET }';

export type RPCResponse<Request extends APIRequest> = Extract<
  APIResponse,
  { typeName: Request['typeName'] }
>;

export async function doRPC<Request extends APIRequest>(
  url: string,
  request: Request,
  signal?: AbortSignal,
): Promise<RPCResponse<Request>> {
  const containsSecret = '$secret' in request;

  console.debug('RPC: %s', request.typeName, containsSecret ? SECRET_PLACEHOLDER : request);

  const onAbort = () => {
    console.debug(
      'RPC: aborted %s',
      request.typeName,
      containsSecret ? SECRET_PLACEHOLDER : request,
    );
  };
  signal?.addEventListener('abort', onAbort);

  try {
    if (containsSecret) {
      // @ts-expect-error remove marker prop
      delete request.$secret;
    }

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

    const parsedResponse = JSON.parse(message) as RPCResponse<Request>;

    return parsedResponse;
  } catch (e) {
    const isAbortError = e instanceof Error && e.name === 'AbortError';

    // we log the abort error in the signal event handler above
    if (!isAbortError) {
      console.error(e);
    }
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
    signal?: AbortSignal,
  ) => Promise<Extract<Response, { typeName: key }>>;
};

function createRPCProxy<Request extends SerdeEnum, Response extends SerdeEnum>(
  url: string,
): ProxyHandlers<Request, Response> {
  return new Proxy(
    {},
    {
      get(_, prop: string) {
        return (params: Obj, signal?: AbortSignal) =>
          doRPC(url, { typeName: prop, ...params } as APIRequest, signal);
      },
    },
  ) as ProxyHandlers<Request, Response>;
}

export const API_ENDPOINT = `${window.CONFIG.basePath}/api`;

export const RPC = createRPCProxy<APIRequest, APIResponse>(API_ENDPOINT);

export async function uploadFile(file: File, signal?: AbortSignal): Promise<DocumentId> {
  console.debug('File upload: %s %s', file.name, formatBytes(file.size));

  const onAbort = () => {
    console.debug('File upload: aborted %s', file.name);
  };
  signal?.addEventListener('abort', onAbort);

  try {
    const response = await fetch(`${window.CONFIG.basePath}/assets`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/octet-stream',
        'X-File-Name': file.name,
      },
      body: file,
    });

    console.debug('File upload: %s finished', file.name);
    const message = await response.text();

    if (!response.ok) {
      throw new Error(`File upload failed: ${response.status}\n${message}`);
    }

    return message as DocumentId;
  } catch (e) {
    const isAbortError = e instanceof Error && e.name === 'AbortError';

    // we log the abort error in the signal event handler above
    if (!isAbortError) {
      console.error(e);
    }
    throw e;
  } finally {
    signal?.removeEventListener('abort', onAbort);
  }
}

export function getDocumentUrl(documentId: DocumentId): string {
  return `${window.location.origin}${window.CONFIG.basePath}?id=${documentId}`;
}

export function getAssetUrl(assetId: DocumentId): string {
  return `${window.CONFIG.basePath}/assets/${assetId}`;
}

export function getScaledImageUrl(
  assetId: DocumentId,
  maxWidth: number,
  maxHeight: number,
): string {
  return `${window.CONFIG.basePath}/assets/images/${assetId}?max_w=${maxWidth}&max_h=${maxHeight}`;
}
