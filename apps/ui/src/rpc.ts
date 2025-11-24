import { invoke } from '@tauri-apps/api/tauri';

let requestId = 0;

export const sendRpc = async (method: string, params: any = {}) => {
    const request: any = {
        jsonrpc: "2.0",
        id: ++requestId,
        method,
    };

    if (params && Object.keys(params).length > 0) {
        request.params = params;
    }

    try {
        const response = await invoke('rpc_request', { request: JSON.stringify(request) });
        return JSON.parse(response as string);
    } catch (e) {
        console.error("RPC Error:", e);
        return null;
    }
};
