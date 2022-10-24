import { createRPCProxy } from './utils/rpc';
import { APIRequest, APIResponse } from '../dto';

export const RPC = createRPCProxy<APIRequest, APIResponse>('/api');
